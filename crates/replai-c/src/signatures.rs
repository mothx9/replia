// Generated signature checks connect the header schema to every exported Rust function.
const _: unsafe extern "C" fn(*mut u32) -> i32 = replai_abi_version;
const _: unsafe extern "C" fn(*const ReplaiConfig, *mut *mut Handle) -> i32 = replai_create;
const _: unsafe extern "C" fn(*mut *mut Handle) -> i32 = replai_destroy;
const _: unsafe extern "C" fn(
    *mut Handle,
    *const u8,
    usize,
    *const u8,
    usize,
    *const u8,
    usize,
) -> i32 = replai_prompt;
const _: unsafe extern "C" fn(*mut Handle, i32, i32) -> i32 = replai_open;
const _: unsafe extern "C" fn(*mut Handle) -> i32 = replai_close;
const _: unsafe extern "C" fn(*mut Handle, u32, *mut ReplaiEvent) -> i32 = replai_poll;
const _: unsafe extern "C" fn(*mut Handle, *mut ReplaiEvent) -> i32 = replai_interrupt;
const _: unsafe extern "C" fn(*mut Handle, *mut u8, usize, *mut usize, *mut usize) -> i32 =
    replai_draft_copy;
const _: unsafe extern "C" fn(*mut Handle, *mut u8, usize, *mut usize) -> i32 =
    replai_submitted_copy;
const _: unsafe extern "C" fn(*mut Handle, *const u8, usize) -> i32 = replai_set_draft;
const _: unsafe extern "C" fn(*mut Handle) -> i32 = replai_clear;
const _: unsafe extern "C" fn(*mut Handle, *const u8, usize) -> i32 = replai_history_add;
const _: unsafe extern "C" fn(*mut Handle, usize, usize, *const u8, usize) -> i32 = replai_complete;
const _: unsafe extern "C" fn(*mut Handle, u32, *const u8, usize) -> i32 = replai_external_output;
const _: unsafe extern "C" fn(i32, *mut u8, usize, *mut usize) -> i32 = replai_status_text;
