// Generated signature checks connect the header schema to every exported Rust function.
const _: unsafe extern "C" fn(*mut u32) -> i32 = replia_abi_version;
const _: unsafe extern "C" fn(*const RepliaConfig, *mut *mut Handle) -> i32 = replia_create;
const _: unsafe extern "C" fn(*mut *mut Handle) -> i32 = replia_destroy;
const _: unsafe extern "C" fn(
    *mut Handle,
    *const u8,
    usize,
    *const u8,
    usize,
    *const u8,
    usize,
) -> i32 = replia_prompt;
const _: unsafe extern "C" fn(*mut Handle, i32, i32) -> i32 = replia_open;
const _: unsafe extern "C" fn(*mut Handle) -> i32 = replia_close;
const _: unsafe extern "C" fn(*mut Handle, u32, *mut RepliaEvent) -> i32 = replia_poll;
const _: unsafe extern "C" fn(*mut Handle, *mut RepliaEvent) -> i32 = replia_interrupt;
const _: unsafe extern "C" fn(*mut Handle, *mut u8, usize, *mut usize, *mut usize) -> i32 =
    replia_draft_copy;
const _: unsafe extern "C" fn(*mut Handle, *mut u8, usize, *mut usize) -> i32 =
    replia_submitted_copy;
const _: unsafe extern "C" fn(*mut Handle, *const u8, usize) -> i32 = replia_set_draft;
const _: unsafe extern "C" fn(*mut Handle) -> i32 = replia_clear;
const _: unsafe extern "C" fn(*mut Handle, *const u8, usize) -> i32 = replia_history_add;
const _: unsafe extern "C" fn(*mut Handle, usize, usize, *const u8, usize) -> i32 = replia_complete;
const _: unsafe extern "C" fn(*mut Handle, u32, *const u8, usize) -> i32 = replia_external_output;
const _: unsafe extern "C" fn(i32, *mut u8, usize, *mut usize) -> i32 = replia_status_text;
