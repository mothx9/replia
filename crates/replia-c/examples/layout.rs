//! Executable Rust side of the C/Rust ABI layout comparison.
use replia_c::*;
fn main() {
    println!("REPLIA_C_ABI_VERSION={}", REPLIA_C_ABI_VERSION);
    println!("REPLIA_OK={}", REPLIA_OK);
    println!("REPLIA_INVALID_ARGUMENT={}", REPLIA_INVALID_ARGUMENT);
    println!("REPLIA_INVALID_UTF8={}", REPLIA_INVALID_UTF8);
    println!("REPLIA_INVALID_RANGE={}", REPLIA_INVALID_RANGE);
    println!("REPLIA_CAPACITY={}", REPLIA_CAPACITY);
    println!("REPLIA_INVALID_STATE={}", REPLIA_INVALID_STATE);
    println!("REPLIA_UNSUITABLE_TERMINAL={}", REPLIA_UNSUITABLE_TERMINAL);
    println!("REPLIA_IO={}", REPLIA_IO);
    println!("REPLIA_BUFFER_TOO_SMALL={}", REPLIA_BUFFER_TOO_SMALL);
    println!("REPLIA_ABI_MISMATCH={}", REPLIA_ABI_MISMATCH);
    println!("REPLIA_BUSY={}", REPLIA_BUSY);
    println!("REPLIA_INTERNAL={}", REPLIA_INTERNAL);
    println!("REPLIA_INVALID_TEXT={}", REPLIA_INVALID_TEXT);
    println!("REPLIA_HISTORY_DISABLED={}", REPLIA_HISTORY_DISABLED);
    println!("REPLIA_INVALID_SEQUENCE={}", REPLIA_INVALID_SEQUENCE);
    println!("REPLIA_EVENT_NONE={}", REPLIA_EVENT_NONE);
    println!("REPLIA_EVENT_SUBMITTED={}", REPLIA_EVENT_SUBMITTED);
    println!("REPLIA_EVENT_INTERRUPTED={}", REPLIA_EVENT_INTERRUPTED);
    println!("REPLIA_EVENT_END_OF_INPUT={}", REPLIA_EVENT_END_OF_INPUT);
    println!(
        "REPLIA_EVENT_COMPLETION_REQUESTED={}",
        REPLIA_EVENT_COMPLETION_REQUESTED
    );
    println!("REPLIA_EVENT_EDIT_REJECTED={}", REPLIA_EVENT_EDIT_REJECTED);
    println!("REPLIA_ROLE_DEFAULT={}", REPLIA_ROLE_DEFAULT);
    println!("REPLIA_ROLE_STRONG={}", REPLIA_ROLE_STRONG);
    println!("REPLIA_ROLE_ACCENT={}", REPLIA_ROLE_ACCENT);
    println!("REPLIA_ROLE_DIM={}", REPLIA_ROLE_DIM);
    println!("REPLIA_ROLE_SUCCESS={}", REPLIA_ROLE_SUCCESS);
    println!("REPLIA_ROLE_WARNING={}", REPLIA_ROLE_WARNING);
    println!("REPLIA_ROLE_ERROR={}", REPLIA_ROLE_ERROR);
    println!("replia_config.size={}", std::mem::size_of::<RepliaConfig>());
    println!(
        "replia_config.align={}",
        std::mem::align_of::<RepliaConfig>()
    );
    println!(
        "replia_config.struct_size={}",
        std::mem::offset_of!(RepliaConfig, struct_size)
    );
    println!(
        "replia_config.abi_version={}",
        std::mem::offset_of!(RepliaConfig, abi_version)
    );
    println!(
        "replia_config.max_input_bytes={}",
        std::mem::offset_of!(RepliaConfig, max_input_bytes)
    );
    println!(
        "replia_config.history_entries={}",
        std::mem::offset_of!(RepliaConfig, history_entries)
    );
    println!(
        "replia_config.reserved={}",
        std::mem::offset_of!(RepliaConfig, reserved)
    );
    println!("replia_event.size={}", std::mem::size_of::<RepliaEvent>());
    println!("replia_event.align={}", std::mem::align_of::<RepliaEvent>());
    println!(
        "replia_event.struct_size={}",
        std::mem::offset_of!(RepliaEvent, struct_size)
    );
    println!(
        "replia_event.abi_version={}",
        std::mem::offset_of!(RepliaEvent, abi_version)
    );
    println!(
        "replia_event.kind={}",
        std::mem::offset_of!(RepliaEvent, kind)
    );
    println!(
        "replia_event.status={}",
        std::mem::offset_of!(RepliaEvent, status)
    );
    println!(
        "replia_event.text_bytes={}",
        std::mem::offset_of!(RepliaEvent, text_bytes)
    );
    println!(
        "replia_event.cursor_bytes={}",
        std::mem::offset_of!(RepliaEvent, cursor_bytes)
    );
    println!(
        "replia_event.reserved={}",
        std::mem::offset_of!(RepliaEvent, reserved)
    );
}
