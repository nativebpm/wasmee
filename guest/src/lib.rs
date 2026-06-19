// Declare 1MB shared static exchange buffer.
const MAX_BUFFER_SIZE: usize = 1024 * 1024;
static mut EXCHANGE_BUFFER: [u8; MAX_BUFFER_SIZE] = [0u8; MAX_BUFFER_SIZE];

#[link(wasm_import_module = "env")]
extern "C" {
    pub fn checkpoint();
    pub fn host_call_api(
        api_name_ptr: *const u8,
        api_name_len: usize,
        req_ptr: *const u8,
        req_len: usize,
        resp_ptr: *mut u8,
        resp_max_len: usize,
    ) -> i32;
}

#[no_mangle]
pub extern "C" fn get_exchange_buffer_pointer() -> *mut u8 {
    std::ptr::addr_of_mut!(EXCHANGE_BUFFER) as *mut u8
}

#[no_mangle]
pub extern "C" fn execute(_graph_len: u32, _variables_len: u32) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn resume(
    _graph_len: u32,
    _instance_len: u32,
    _completed_task_id_ptr: u32,
    _completed_task_id_len: u32,
) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn run_test() -> i32 {
    let api_name = "test_api";
    let req1 = b"hello";
    let mut resp_buf = vec![0u8; 1024];

    let _res1 = unsafe {
        host_call_api(
            api_name.as_ptr(),
            api_name.len(),
            req1.as_ptr(),
            req1.len(),
            resp_buf.as_mut_ptr(),
            resp_buf.len(),
        )
    };

    unsafe {
        checkpoint();
    }

    // Modify memory at offset 70000 to trigger dirty-page tracking
    unsafe {
        let ptr = 70000 as *mut i32;
        std::ptr::write_volatile(ptr, 42);
    }

    let req2 = b"world";
    let _res2 = unsafe {
        host_call_api(
            api_name.as_ptr(),
            api_name.len(),
            req2.as_ptr(),
            req2.len(),
            resp_buf.as_mut_ptr(),
            resp_buf.len(),
        )
    };

    unsafe {
        checkpoint();
    }

    0
}
