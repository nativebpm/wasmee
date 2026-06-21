package main

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"sync"
	"time"

	"github.com/google/uuid"
	"github.com/nativebpm/jsonschema"
	"github.com/nativebpm/wasmee"
	"github.com/nativebpm/wasmee/olme"
)

// memoryStore implements olme.SnapshotStore in memory.
type memoryStore struct {
	mu        sync.Mutex
	snapshots map[string][]byte
	deltas    map[string]map[int][]byte
	oplogs    map[string][]olme.OplogEntry
	metadata  map[string]*olme.InstanceMeta
}

func newMemoryStore() *memoryStore {
	return &memoryStore{
		snapshots: make(map[string][]byte),
		deltas:    make(map[string]map[int][]byte),
		oplogs:    make(map[string][]olme.OplogEntry),
		metadata:  make(map[string]*olme.InstanceMeta),
	}
}

func (s *memoryStore) SaveSnapshot(ctx context.Context, id string, snapshot []byte) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.snapshots[id] = snapshot
	return nil
}

func (s *memoryStore) LoadSnapshot(ctx context.Context, id string) ([]byte, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	data, ok := s.snapshots[id]
	if !ok {
		return nil, fmt.Errorf("snapshot not found")
	}
	return data, nil
}

func (s *memoryStore) DeleteSnapshot(ctx context.Context, id string) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	delete(s.snapshots, id)
	return nil
}

func (s *memoryStore) SaveDeltas(ctx context.Context, id string, deltas map[int][]byte) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.deltas[id] == nil {
		s.deltas[id] = make(map[int][]byte)
	}
	for k, v := range deltas {
		s.deltas[id][k] = v
	}
	return nil
}

func (s *memoryStore) LoadDeltas(ctx context.Context, id string) (map[int][]byte, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.deltas[id], nil
}

func (s *memoryStore) TruncateDeltas(ctx context.Context, id string) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	delete(s.deltas, id)
	return nil
}

func (s *memoryStore) SaveOplog(ctx context.Context, id string, entry olme.OplogEntry) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.oplogs[id] = append(s.oplogs[id], entry)
	return nil
}

func (s *memoryStore) LoadOplog(ctx context.Context, id string) ([]olme.OplogEntry, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.oplogs[id], nil
}

func (s *memoryStore) TruncateOplog(ctx context.Context, id string, beforeCallIndex int) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	var filtered []olme.OplogEntry
	for _, entry := range s.oplogs[id] {
		if entry.CallIndex < beforeCallIndex {
			filtered = append(filtered, entry)
		}
	}
	s.oplogs[id] = filtered
	return nil
}

func (s *memoryStore) SaveMetadata(ctx context.Context, meta *olme.InstanceMeta) (bool, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.metadata[meta.InstanceID] = meta
	return true, nil
}

func (s *memoryStore) LoadMetadata(ctx context.Context, id string) (*olme.InstanceMeta, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	meta, ok := s.metadata[id]
	if !ok {
		return nil, nil
	}
	return meta, nil
}

type ProcessInstance struct {
	ID                       string                 `json:"id"`
	ProcessID                string                 `json:"process_id"` // "todo", "kanban", "incident"
	ActiveActivityInstances  []string               `json:"active_activity_instances"`
	WaitingActivityInstances []string               `json:"waiting_activity_instances"`
	CompletedNodes           []string               `json:"completed_nodes"`
	Variables                map[string]interface{} `json:"variables"`
	Completed                bool                   `json:"completed"`
}

var taskSchemas = map[string]string{
	"add_todo": `{
		"type": "object",
		"title": "Add Todo Item",
		"properties": {
			"todo_item": { "type": "string", "title": "Task Description", "ui:widget": "text" },
			"priority": { "type": "string", "title": "Priority", "enum": ["Low", "Medium", "High"], "ui:widget": "select", "default": "Medium" }
		},
		"required": ["todo_item"]
	}`,
	"complete_todo": `{
		"type": "object",
		"title": "Complete Todo Item",
		"properties": {
			"todo_item": { "type": "string", "title": "Task Description (Read Only)", "ui:widget": "text" },
			"priority": { "type": "string", "title": "Priority", "ui:widget": "text" },
			"todo_item_completed": { "type": "boolean", "title": "Mark as Completed", "ui:widget": "checkbox" }
		},
		"required": ["todo_item_completed"]
	}`,
	"create_task": `{
		"type": "object",
		"title": "Create Kanban Task",
		"properties": {
			"task_name": { "type": "string", "title": "Task Title", "ui:widget": "text" },
			"task_description": { "type": "string", "title": "Task Description", "ui:widget": "text" }
		},
		"required": ["task_name"]
	}`,
	"backlog": `{
		"type": "object",
		"title": "Assign Task (Start Progress)",
		"properties": {
			"task_name": { "type": "string", "title": "Task Title", "ui:widget": "text" },
			"assignee": { "type": "string", "title": "Assignee Name", "ui:widget": "text" }
		},
		"required": ["assignee"]
	}`,
	"in_progress": `{
		"type": "object",
		"title": "Submit Task for Review",
		"properties": {
			"task_name": { "type": "string", "title": "Task Title", "ui:widget": "text" },
			"assignee": { "type": "string", "title": "Assignee", "ui:widget": "text" }
		}
	}`,
	"in_review": `{
		"type": "object",
		"title": "Review Task Approval",
		"properties": {
			"task_name": { "type": "string", "title": "Task Title", "ui:widget": "text" },
			"approved": { "type": "boolean", "title": "Approve and Close Task", "ui:widget": "checkbox" }
		},
		"required": ["approved"]
	}`,
	"create_incident": `{
		"type": "object",
		"title": "Create ITIL Incident Ticket",
		"properties": {
			"title": { "type": "string", "title": "Incident Title", "ui:widget": "text" },
			"description": { "type": "string", "title": "Incident Description", "ui:widget": "text" },
			"priority": { "type": "string", "title": "Priority level", "enum": ["Low", "Medium", "High", "Critical"], "ui:widget": "select", "default": "Medium" }
		},
		"required": ["title", "description"]
	}`,
	"new_incident": `{
		"type": "object",
		"title": "Assign Incident Ticket",
		"properties": {
			"title": { "type": "string", "title": "Incident Title", "ui:widget": "text" },
			"assignee": { "type": "string", "title": "Support Engineer Name", "ui:widget": "text" }
		},
		"required": ["assignee"]
	}`,
	"investigating": `{
		"type": "object",
		"title": "Update Investigation Details",
		"properties": {
			"title": { "type": "string", "title": "Incident Title", "ui:widget": "text" },
			"comments": { "type": "string", "title": "Investigation Update / Log", "ui:widget": "text" }
		},
		"required": ["comments"]
	}`,
	"resolved": `{
		"type": "object",
		"title": "Resolve Incident Ticket",
		"properties": {
			"title": { "type": "string", "title": "Incident Title", "ui:widget": "text" },
			"resolution": { "type": "string", "title": "Resolution Notes", "ui:widget": "text" }
		},
		"required": ["resolution"]
	}`,
}

var (
	runner      *wasmee.Runner
	store       *memoryStore
	instances   = make(map[string]ProcessInstance)
	instancesMu sync.RWMutex
)

func main() {
	wasmPath := filepath.Join("target", "wasm32-wasip1", "release", "servicedesk_guest.wasm")
	if _, err := os.Stat(wasmPath); os.IsNotExist(err) {
		wasmPath = filepath.Join("wasmee", "target", "wasm32-wasip1", "release", "servicedesk_guest.wasm")
		if _, err := os.Stat(wasmPath); os.IsNotExist(err) {
			// Try workspace root path
			wasmPath = filepath.Join("..", "..", "target", "wasm32-wasip1", "release", "servicedesk_guest.wasm")
			if _, err := os.Stat(wasmPath); os.IsNotExist(err) {
				fmt.Println("Error: servicedesk_guest.wasm not found. Please compile the Rust guest first.")
				os.Exit(1)
			}
		}
	}

	wasmBytes, err := os.ReadFile(wasmPath)
	if err != nil {
		fmt.Printf("Failed to read guest WASM file: %v\n", err)
		os.Exit(1)
	}

	ctx := context.Background()
	store = newMemoryStore()

	var errRunner error
	runner, errRunner = wasmee.NewRunner(ctx, wasmBytes, "http://localhost:8081")
	if errRunner != nil {
		fmt.Printf("Failed to initialize wasmee runner: %v\n", errRunner)
		os.Exit(1)
	}

	// SLA Ticker Routine
	go startSLATicker(ctx)

	// API Routing
	http.HandleFunc("/", handleHome)
	http.HandleFunc("/api/instances", handleListInstances)
	http.HandleFunc("/api/instances/create", handleCreateInstance)
	http.HandleFunc("/api/instances/state", handleGetInstanceState)
	http.HandleFunc("/api/instances/submit", handleSubmitTask)

	port := "8086"
	fmt.Printf("Wasmee Multi-Domain ServiceDesk App is running on http://localhost:%s\n", port)
	if err := http.ListenAndServe(":"+port, nil); err != nil {
		fmt.Printf("HTTP server failed: %v\n", err)
	}
}

func startSLATicker(ctx context.Context) {
	ticker := time.NewTicker(2 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			instancesMu.Lock()
			for id, inst := range instances {
				if inst.ProcessID == "incident" && !inst.Completed {
					// Load State
					state := olme.NewSessionState(id, store)
					if err := state.Load(ctx); err != nil {
						continue
					}

					session := wasmee.NewSession(id, state)
					instBytes, _ := json.Marshal(inst)

					// Execute tick on Wasm
					crashed, respBytes, err := runner.Execute(ctx, session, "tick", instBytes, uint64(len(instBytes)))
					if err == nil && !crashed {
						var updatedInst ProcessInstance
						if err := json.Unmarshal(respBytes, &updatedInst); err == nil {
							instances[id] = updatedInst
						}
					}
				}
			}
			instancesMu.Unlock()
		}
	}
}

func handleHome(w http.ResponseWriter, r *http.Request) {
	if r.URL.Path != "/" {
		http.NotFound(w, r)
		return
	}
	// Serve examples/servicedesk/index.html
	filePath := filepath.Join("examples", "servicedesk", "index.html")
	if _, err := os.Stat(filePath); os.IsNotExist(err) {
		filePath = "index.html" // Fallback if ran inside folder
	}

	file, err := os.Open(filePath)
	if err != nil {
		http.Error(w, "index.html not found", http.StatusNotFound)
		return
	}
	defer file.Close()

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	io.Copy(w, file)
}

func handleListInstances(w http.ResponseWriter, r *http.Request) {
	instancesMu.RLock()
	defer instancesMu.RUnlock()

	w.Header().Set("Content-Type", "application/json")
	var list []ProcessInstance
	for _, inst := range instances {
		list = append(list, inst)
	}
	json.NewEncoder(w).Encode(list)
}

func handleCreateInstance(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Only POST allowed", http.StatusMethodNotAllowed)
		return
	}

	var req struct {
		ProcessID string                 `json:"process_id"`
		Variables map[string]interface{} `json:"variables"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid request body", http.StatusBadRequest)
		return
	}

	id := uuid.New().String()
	ctx := r.Context()

	// Initialize OLME storage for this session
	meta := &olme.InstanceMeta{
		InstanceID: id,
		WasmHash:   "servicedesk_process_hash",
		Version:    0,
	}
	_, _ = store.SaveMetadata(ctx, meta)

	state := olme.NewSessionState(id, store)
	if err := state.Load(ctx); err != nil {
		http.Error(w, "Failed to load state: "+err.Error(), http.StatusInternalServerError)
		return
	}

	session := wasmee.NewSession(id, state)
	if req.Variables == nil {
		req.Variables = make(map[string]interface{})
	}

	// Prepare exchange buffer: type + variables
	typeBytes := []byte(req.ProcessID)
	varsBytes, _ := json.Marshal(req.Variables)

	exchangeBuffer := make([]byte, len(typeBytes)+len(varsBytes))
	copy(exchangeBuffer[0:len(typeBytes)], typeBytes)
	copy(exchangeBuffer[len(typeBytes):], varsBytes)

	crashed, respBytes, err := runner.Execute(ctx, session, "execute", exchangeBuffer, uint64(len(typeBytes)), uint64(len(varsBytes)))
	if err != nil {
		http.Error(w, "Execution failed: "+err.Error(), http.StatusInternalServerError)
		return
	}
	if crashed {
		http.Error(w, "Execution crashed inside WASM sandbox", http.StatusInternalServerError)
		return
	}

	var inst ProcessInstance
	if err := json.Unmarshal(respBytes, &inst); err != nil {
		http.Error(w, "Failed to decode process output: "+err.Error(), http.StatusInternalServerError)
		return
	}
	inst.ID = id

	instancesMu.Lock()
	instances[id] = inst
	instancesMu.Unlock()

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(inst)
}

func handleGetInstanceState(w http.ResponseWriter, r *http.Request) {
	id := r.URL.Query().Get("id")
	if id == "" {
		http.Error(w, "Missing id parameter", http.StatusBadRequest)
		return
	}

	instancesMu.RLock()
	inst, exists := instances[id]
	instancesMu.RUnlock()

	if !exists {
		http.Error(w, "Instance not found", http.StatusNotFound)
		return
	}

	// Render dynamic json schema widgets
	var widgets []*jsonschema.UIWidgetSpec
	if len(inst.WaitingActivityInstances) > 0 {
		activeTask := inst.WaitingActivityInstances[0]
		if schema, ok := taskSchemas[activeTask]; ok {
			var err error
			widgets, err = jsonschema.ParseSchema(schema, inst.Variables)
			if err != nil {
				http.Error(w, "Failed to parse schema: "+err.Error(), http.StatusInternalServerError)
				return
			}
		}
	}

	response := map[string]interface{}{
		"id":            inst.ID,
		"process_id":    inst.ProcessID,
		"completed":     inst.Completed,
		"active_nodes":  inst.ActiveActivityInstances,
		"waiting_nodes": inst.WaitingActivityInstances,
		"variables":     inst.Variables,
		"widgets":       widgets,
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(response)
}

func handleSubmitTask(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Only POST allowed", http.StatusMethodNotAllowed)
		return
	}

	var req struct {
		ID   string                 `json:"id"`
		Vars map[string]interface{} `json:"variables"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid body", http.StatusBadRequest)
		return
	}

	instancesMu.Lock()
	inst, exists := instances[req.ID]
	instancesMu.Unlock()

	if !exists {
		http.Error(w, "Instance not found", http.StatusNotFound)
		return
	}

	if len(inst.WaitingActivityInstances) == 0 {
		http.Error(w, "No waiting activity to submit", http.StatusBadRequest)
		return
	}
	activeTask := inst.WaitingActivityInstances[0]

	// Extract role header
	role := r.Header.Get("X-Role")
	if role == "" {
		role = "Customer"
	}

	ctx := r.Context()
	state := olme.NewSessionState(req.ID, store)
	if err := state.Load(ctx); err != nil {
		http.Error(w, "Failed to load state: "+err.Error(), http.StatusInternalServerError)
		return
	}

	session := wasmee.NewSession(req.ID, state)

	// Merge form variables with existing
	if inst.Variables == nil {
		inst.Variables = make(map[string]interface{})
	}

	instBytes, _ := json.Marshal(inst)
	activeTaskBytes := []byte(activeTask)
	inputBytes, _ := json.Marshal(req.Vars)
	roleBytes := []byte(role)

	// Construct exchange buffer back-to-back
	instanceOffset := len(instBytes)
	activeTaskOffset := instanceOffset + len(activeTaskBytes)
	inputOffset := activeTaskOffset + len(inputBytes)

	exchangeBuffer := make([]byte, inputOffset+len(roleBytes))
	copy(exchangeBuffer[0:instanceOffset], instBytes)
	copy(exchangeBuffer[instanceOffset:activeTaskOffset], activeTaskBytes)
	copy(exchangeBuffer[activeTaskOffset:inputOffset], inputBytes)
	copy(exchangeBuffer[inputOffset:], roleBytes)

	crashed, respBytes, err := runner.Execute(ctx, session, "resume", exchangeBuffer,
		uint64(len(instBytes)),
		uint64(len(activeTaskBytes)),
		uint64(len(inputBytes)),
		uint64(len(roleBytes)),
	)

	if err != nil {
		// Detect role authorization error from Wasm return code
		if err.Error() == "WASM core execution failed with error code: -403" {
			http.Error(w, "Access Denied: You do not have permissions for this action.", http.StatusForbidden)
			return
		}
		http.Error(w, "Execution failed: "+err.Error(), http.StatusInternalServerError)
		return
	}
	if crashed {
		http.Error(w, "Execution crashed inside WASM sandbox", http.StatusInternalServerError)
		return
	}

	var updatedInst ProcessInstance
	if err := json.Unmarshal(respBytes, &updatedInst); err != nil {
		http.Error(w, "Failed to decode execution output: "+err.Error(), http.StatusInternalServerError)
		return
	}
	updatedInst.ID = req.ID

	instancesMu.Lock()
	instances[req.ID] = updatedInst
	instancesMu.Unlock()

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(updatedInst)
}
