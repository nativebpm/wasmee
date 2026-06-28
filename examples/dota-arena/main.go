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

	"github.com/nativebpm/wasmee"

)

// memoryStore implements wasmee.SnapshotStore in memory.
type memoryStore struct {
	mu        sync.Mutex
	snapshots map[string][]byte
	deltas    map[string]map[int][]byte
	oplogs    map[string][]wasmee.OplogEntry
	metadata  map[string]*wasmee.InstanceMeta
}

func newMemoryStore() *memoryStore {
	return &memoryStore{
		snapshots: make(map[string][]byte),
		deltas:    make(map[string]map[int][]byte),
		oplogs:    make(map[string][]wasmee.OplogEntry),
		metadata:  make(map[string]*wasmee.InstanceMeta),
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

func (s *memoryStore) SaveOplog(ctx context.Context, id string, entry wasmee.OplogEntry) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.oplogs[id] = append(s.oplogs[id], entry)
	return nil
}

func (s *memoryStore) LoadOplog(ctx context.Context, id string) ([]wasmee.OplogEntry, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.oplogs[id], nil
}

func (s *memoryStore) TruncateOplog(ctx context.Context, id string, beforeCallIndex int) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	var filtered []wasmee.OplogEntry
	for _, entry := range s.oplogs[id] {
		if entry.CallIndex < beforeCallIndex {
			filtered = append(filtered, entry)
		}
	}
	s.oplogs[id] = filtered
	return nil
}

func (s *memoryStore) SaveMetadata(ctx context.Context, meta *wasmee.InstanceMeta) (bool, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.metadata[meta.InstanceID] = meta
	return true, nil
}

func (s *memoryStore) LoadMetadata(ctx context.Context, id string) (*wasmee.InstanceMeta, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	meta, ok := s.metadata[id]
	if !ok {
		return nil, nil
	}
	return meta, nil
}

type Cooldowns struct {
	StormHammer int `json:"storm_hammer"`
	Warcry      int `json:"warcry"`
	DragonSlave int `json:"dragon_slave"`
	LagunaBlade int `json:"laguna_blade"`
}

type Hero struct {
	ID           int       `json:"id"`
	Name         string    `json:"name"`
	X            int       `json:"x"`
	Y            int       `json:"y"`
	HP           int       `json:"hp"`
	MaxHP        int       `json:"max_hp"`
	Damage       int       `json:"damage"`
	Range        int       `json:"range"`
	StunnedTurns int       `json:"stunned_turns"`
	WarcryTurns  int       `json:"warcry_turns"`
	Cooldowns    Cooldowns `json:"cooldowns"`
	DodgeChance  int       `json:"dodge_chance"`
	IsRadiant    bool      `json:"is_radiant"`
}

type GameState struct {
	Turn            int      `json:"turn"`
	Status          string   `json:"status"`
	Radiant         []Hero   `json:"radiant"`
	Dire            []Hero   `json:"dire"`
	Log             []string `json:"log"`
	CheckpointCount int      `json:"checkpoint_count"`
	PausedBy        string   `json:"paused_by,omitempty"`
	PauseRemaining  int      `json:"pause_remaining,omitempty"`
}

var (
	runner          *wasmee.Runner
	store           *memoryStore
	instanceID      = "dota-arena-session"
	currentInstance GameState
	stateMu         sync.Mutex
	currentMode     = "pve_radiant"
	lastHeartbeat   = make(map[string]time.Time)
	pauseEndsAt     time.Time
	pausedBy        string
)

func main() {
	wasmPath := filepath.Join("target", "wasm32-wasip1", "release", "dota_guest.wasm")
	if _, err := os.Stat(wasmPath); os.IsNotExist(err) {
		wasmPath = filepath.Join("..", "..", "target", "wasm32-wasip1", "release", "dota_guest.wasm")
		if _, err := os.Stat(wasmPath); os.IsNotExist(err) {
			fmt.Println("Error: dota_guest.wasm not found. Please compile the Rust guest first.")
			os.Exit(1)
		}
	}

	wasmBytes, err := os.ReadFile(wasmPath)
	if err != nil {
		fmt.Printf("Failed to read guest WASM file: %v\n", err)
		os.Exit(1)
	}

	ctx := context.Background()
	store = newMemoryStore()

	// Initialize the wasmee HTTP runner (expects server listening on :8081)
	var errRunner error
	runner, errRunner = wasmee.NewRunner(ctx, wasmBytes, "http://localhost:8081")
	if errRunner != nil {
		fmt.Printf("Failed to initialize wasmee runner: %v\n", errRunner)
		os.Exit(1)
	}

	// Start or reset process initial state
	if err := startNewGameInstance(ctx); err != nil {
		fmt.Printf("Failed to start initial game: %v\n", err)
		os.Exit(1)
	}

	// Set up HTTP routes
	http.HandleFunc("/", handleHome)
	http.HandleFunc("/api/state", handleState)
	http.HandleFunc("/api/submit", handleSubmit)
	http.HandleFunc("/api/reset", handleReset)
	http.HandleFunc("/api/crash", handleCrash)
	http.HandleFunc("/api/checkpoints", handleCheckpoints)
	http.HandleFunc("/api/claim-victory", handleClaimVictory)

	port := "8085"
	fmt.Printf("Wasmee Dota Arena Web App is running on http://localhost:%s\n", port)
	if err := http.ListenAndServe(":"+port, nil); err != nil {
		fmt.Printf("HTTP server failed: %v\n", err)
	}
}

func startNewGameInstance(ctx context.Context) error {
	stateMu.Lock()
	defer stateMu.Unlock()

	lastHeartbeat = make(map[string]time.Time)
	pauseEndsAt = time.Time{}
	pausedBy = ""
	store.mu.Lock()
	delete(store.snapshots, instanceID)
	delete(store.deltas, instanceID)
	delete(store.oplogs, instanceID)
	delete(store.metadata, instanceID)
	store.mu.Unlock()

	meta := &wasmee.InstanceMeta{
		InstanceID: instanceID,
		WasmHash:   "dota_arena_hash",
		Version:    0,
	}
	_, _ = store.SaveMetadata(ctx, meta)

	state := wasmee.NewSessionState(instanceID, store)
	if err := state.Load(ctx); err != nil {
		return err
	}

	session := wasmee.NewSession(instanceID, state)

	initReq := map[string]interface{}{
		"action":          "reset",
		"game_mode":       "pvp",
		"radiant_actions": []interface{}{},
		"dire_actions":    []interface{}{},
	}
	exchangeBuffer, _ := json.Marshal(initReq)

	_, respBytes, err := runner.Execute(ctx, session, "play_turn", exchangeBuffer, uint64(len(exchangeBuffer)))
	if err != nil {
		return err
	}

	return json.Unmarshal(respBytes, &currentInstance)
}

func handleHome(w http.ResponseWriter, r *http.Request) {
	path := r.URL.Path
	if path == "/sven.png" || path == "/lina.png" {
		filePath := filepath.Join("examples", "dota-arena", "static", path)
		if _, err := os.Stat(filePath); os.IsNotExist(err) {
			filePath = filepath.Join("static", path)
		}
		http.ServeFile(w, r, filePath)
		return
	}

	// Serve static/index.html file
	filePath := filepath.Join("examples", "dota-arena", "static", "index.html")
	if _, err := os.Stat(filePath); os.IsNotExist(err) {
		filePath = filepath.Join("static", "index.html")
	}

	file, err := os.Open(filePath)
	if err != nil {
		http.Error(w, "index.html not found", http.StatusNotFound)
		return
	}
	defer file.Close()

	w.Header().Set("Content-Type", "text/html")
	io.Copy(w, file)
}

func handleState(w http.ResponseWriter, r *http.Request) {
	stateMu.Lock()
	defer stateMu.Unlock()

	player := r.URL.Query().Get("player")

	if currentInstance.Turn == 0 {
		ctx := r.Context()
		state := wasmee.NewSessionState(instanceID, store)
		if err := state.Load(ctx); err == nil {
			session := wasmee.NewSession(instanceID, state)
			queryReq := map[string]interface{}{
				"action":          "get_state",
				"game_mode":       "pvp",
				"radiant_actions": []interface{}{},
				"dire_actions":    []interface{}{},
			}
			exchangeBuffer, _ := json.Marshal(queryReq)
			_, respBytes, err := runner.Execute(ctx, session, "play_turn", exchangeBuffer, uint64(len(exchangeBuffer)))
			if err == nil {
				_ = json.Unmarshal(respBytes, &currentInstance)
			}
		}
	}

	// Update heartbeats and check pause in PvP mode
	if currentMode == "pvp" && (currentInstance.Status == "active" || currentInstance.Status == "paused") {
		now := time.Now()
		if player == "radiant" || player == "dire" {
			lastHeartbeat[player] = now
		}

		radHb, radOk := lastHeartbeat["radiant"]
		dirHb, dirOk := lastHeartbeat["dire"]

		radDisconnected := !radOk || now.Sub(radHb) > 10*time.Second
		dirDisconnected := !dirOk || now.Sub(dirHb) > 10*time.Second

		if currentInstance.Status == "active" && (radDisconnected || dirDisconnected) {
			currentInstance.Status = "paused"
			pauseEndsAt = now.Add(3 * time.Minute)
			if radDisconnected {
				pausedBy = "radiant"
			} else {
				pausedBy = "dire"
			}
		} else if currentInstance.Status == "paused" {
			stillDisconnected := false
			if pausedBy == "radiant" && (!radOk || now.Sub(radHb) > 10*time.Second) {
				stillDisconnected = true
			}
			if pausedBy == "dire" && (!dirOk || now.Sub(dirHb) > 10*time.Second) {
				stillDisconnected = true
			}

			if !stillDisconnected {
				currentInstance.Status = "active"
				pausedBy = ""
			}
		}
	}

	if currentInstance.Status == "paused" {
		remaining := int(time.Until(pauseEndsAt).Seconds())
		if remaining < 0 {
			remaining = 0
		}
		currentInstance.PausedBy = pausedBy
		currentInstance.PauseRemaining = remaining
	} else {
		currentInstance.PausedBy = ""
		currentInstance.PauseRemaining = 0
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(currentInstance)
}

func handleSubmit(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Only POST allowed", http.StatusMethodNotAllowed)
		return
	}

	var reqData map[string]interface{}
	if err := json.NewDecoder(r.Body).Decode(&reqData); err != nil {
		http.Error(w, "Invalid body: "+err.Error(), http.StatusBadRequest)
		return
	}

	stateMu.Lock()
	defer stateMu.Unlock()

	if currentInstance.Status == "paused" {
		http.Error(w, "Game is paused due to player disconnect", http.StatusForbidden)
		return
	}

	if mode, ok := reqData["game_mode"].(string); ok {
		currentMode = mode
	}

	ctx := r.Context()

	// Ensure action field is set
	reqData["action"] = "play"

	exchangeBuffer, _ := json.Marshal(reqData)

	state := wasmee.NewSessionState(instanceID, store)
	if err := state.Load(ctx); err != nil {
		http.Error(w, "Failed to load state: "+err.Error(), http.StatusInternalServerError)
		return
	}

	session := wasmee.NewSession(instanceID, state)

	_, respBytes, err := runner.Execute(ctx, session, "play_turn", exchangeBuffer, uint64(len(exchangeBuffer)))
	if err != nil {
		http.Error(w, "Failed to execute wasm turn: "+err.Error(), http.StatusInternalServerError)
		return
	}

	if err := json.Unmarshal(respBytes, &currentInstance); err != nil {
		http.Error(w, "Failed to unmarshal response: "+err.Error(), http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(currentInstance)
}

func handleReset(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Only POST allowed", http.StatusMethodNotAllowed)
		return
	}

	if err := startNewGameInstance(r.Context()); err != nil {
		http.Error(w, "Failed to reset game: "+err.Error(), http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(currentInstance)
}

func handleCrash(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Only POST allowed", http.StatusMethodNotAllowed)
		return
	}

	stateMu.Lock()
	defer stateMu.Unlock()

	// CRASH!
	// We wipe our local in-memory cache of currentInstance.
	// The next action/state request will have to load the session state from the store,
	// demonstrating that memory restoration was successful!
	currentInstance = GameState{}

	w.Header().Set("Content-Type", "application/json")
	w.Write([]byte(`{"status":"crashed","message":"Go cache wiped! Next turn will reload state from the latest checkpoint."}`))
}

func handleCheckpoints(w http.ResponseWriter, r *http.Request) {
	stateMu.Lock()
	defer stateMu.Unlock()

	store.mu.Lock()
	defer store.mu.Unlock()

	snapshots := store.snapshots[instanceID]
	deltas := store.deltas[instanceID]
	meta := store.metadata[instanceID]

	version := 0
	if meta != nil {
		version = int(meta.Version)
	}

	response := map[string]interface{}{
		"has_snapshot": len(snapshots) > 0,
		"snapshot_size": len(snapshots),
		"deltas_count":  len(deltas),
		"version":       version,
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(response)
}

func handleClaimVictory(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Only POST allowed", http.StatusMethodNotAllowed)
		return
	}

	stateMu.Lock()
	defer stateMu.Unlock()

	if currentInstance.Status != "paused" || time.Now().Before(pauseEndsAt) {
		http.Error(w, "Cannot claim victory: pause is not active or timeout not expired", http.StatusBadRequest)
		return
	}

	ctx := r.Context()

	reqData := map[string]interface{}{
		"action":          "claim_victory",
		"game_mode":       pausedBy,
		"radiant_actions": []interface{}{},
		"dire_actions":    []interface{}{},
	}
	exchangeBuffer, _ := json.Marshal(reqData)

	state := wasmee.NewSessionState(instanceID, store)
	if err := state.Load(ctx); err != nil {
		http.Error(w, "Failed to load state: "+err.Error(), http.StatusInternalServerError)
		return
	}

	session := wasmee.NewSession(instanceID, state)
	_, respBytes, err := runner.Execute(ctx, session, "play_turn", exchangeBuffer, uint64(len(exchangeBuffer)))
	if err != nil {
		http.Error(w, "Failed to execute claim victory: "+err.Error(), http.StatusInternalServerError)
		return
	}

	if err := json.Unmarshal(respBytes, &currentInstance); err != nil {
		http.Error(w, "Failed to unmarshal response: "+err.Error(), http.StatusInternalServerError)
		return
	}

	pausedBy = ""
	pauseEndsAt = time.Time{}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(currentInstance)
}
