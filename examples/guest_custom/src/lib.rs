// Shared 1MB exchange buffer for passing inputs/outputs
const BUFFER_SIZE: usize = 1024 * 1024;
static mut EXCHANGE_BUFFER: [u8; BUFFER_SIZE] = [0u8; BUFFER_SIZE];

// Declare external functions imported from the host runner (durable environment)
#[link(wasm_import_module = "env")]
extern "C" {
    // Saves a full execution checkpoint (snapshot of memory/oplog state)
    pub fn checkpoint();

    // Invokes a host-defined API service call (replayed deterministic side-effects)
    pub fn host_call_api(
        api_name_ptr: *const u8,
        api_name_len: usize,
        req_ptr: *const u8,
        req_len: usize,
        resp_ptr: *mut u8,
        resp_max_len: usize,
    ) -> i32;
}

// Export the exchange buffer pointer so the host client can read/write data directly
#[no_mangle]
pub extern "C" fn get_exchange_buffer_pointer() -> *mut u8 {
    std::ptr::addr_of_mut!(EXCHANGE_BUFFER) as *mut u8
}

// Exported durable entrypoint
#[no_mangle]
pub extern "C" fn run_durable_workflow(input_len: u32) -> i32 {
    // 1. Read input parameters from the exchange buffer
    let input_bytes = unsafe { &EXCHANGE_BUFFER[..input_len as usize] };
    let username = std::str::from_utf8(input_bytes).unwrap_or("Guest");

    // 2. Perform a host API call to fetch some external data (e.g. rate or configuration)
    let api_name = "fetch_user_multiplier";
    let req_payload = username.as_bytes();
    let mut resp_buf = [0u8; 128];

    let resp_len = unsafe {
        host_call_api(
            api_name.as_ptr(),
            api_name.len(),
            req_payload.as_ptr(),
            req_payload.len(),
            resp_buf.as_mut_ptr(),
            resp_buf.len(),
        )
    };

    if resp_len < 0 {
        return -1; // API call failed
    }

    let multiplier_str = std::str::from_utf8(&resp_buf[..resp_len as usize]).unwrap_or("1");
    let multiplier: i32 = multiplier_str.parse().unwrap_or(1);

    // 3. Trigger a checkpoint. If the host crashes after this, it will recover precisely from this state!
    unsafe {
        checkpoint();
    }

    // 4. Modify some state (e.g. do memory operations)
    let base_salary = 1000;
    let final_salary = base_salary * multiplier;

    // 5. Write the final result back to the exchange buffer
    let result_str = format!("Hello {}, your final calculated salary is ${}", username, final_salary);
    let result_bytes = result_str.as_bytes();

    if result_bytes.len() > BUFFER_SIZE {
        return -2;
    }

    unsafe {
        EXCHANGE_BUFFER[..result_bytes.len()].copy_from_slice(result_bytes);
    }

    // Return the result length so the host knows how many bytes to read
    result_bytes.len() as i32
}
