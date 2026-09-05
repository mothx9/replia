//! Executable Rust side of the C/Rust ABI layout comparison.
use replai_c::*;
fn main() {
    println!("REPLAI_C_ABI_VERSION={}", REPLAI_C_ABI_VERSION);
    println!("REPLAI_OK={}", REPLAI_OK);
    println!("REPLAI_INVALID_ARGUMENT={}", REPLAI_INVALID_ARGUMENT);
    println!("REPLAI_INVALID_UTF8={}", REPLAI_INVALID_UTF8);
    println!("REPLAI_INVALID_RANGE={}", REPLAI_INVALID_RANGE);
    println!("REPLAI_CAPACITY={}", REPLAI_CAPACITY);
    println!("REPLAI_INVALID_STATE={}", REPLAI_INVALID_STATE);
    println!("REPLAI_UNSUITABLE_TERMINAL={}", REPLAI_UNSUITABLE_TERMINAL);
    println!("REPLAI_IO={}", REPLAI_IO);
    println!("REPLAI_BUFFER_TOO_SMALL={}", REPLAI_BUFFER_TOO_SMALL);
    println!("REPLAI_ABI_MISMATCH={}", REPLAI_ABI_MISMATCH);
    println!("REPLAI_BUSY={}", REPLAI_BUSY);
    println!("REPLAI_INTERNAL={}", REPLAI_INTERNAL);
    println!("REPLAI_INVALID_TEXT={}", REPLAI_INVALID_TEXT);
    println!("REPLAI_HISTORY_DISABLED={}", REPLAI_HISTORY_DISABLED);
    println!("REPLAI_INVALID_SEQUENCE={}", REPLAI_INVALID_SEQUENCE);
    println!("REPLAI_EVENT_NONE={}", REPLAI_EVENT_NONE);
    println!("REPLAI_EVENT_SUBMITTED={}", REPLAI_EVENT_SUBMITTED);
    println!("REPLAI_EVENT_INTERRUPTED={}", REPLAI_EVENT_INTERRUPTED);
    println!("REPLAI_EVENT_END_OF_INPUT={}", REPLAI_EVENT_END_OF_INPUT);
    println!(
        "REPLAI_EVENT_COMPLETION_REQUESTED={}",
        REPLAI_EVENT_COMPLETION_REQUESTED
    );
    println!("REPLAI_EVENT_EDIT_REJECTED={}", REPLAI_EVENT_EDIT_REJECTED);
    println!("REPLAI_ROLE_DEFAULT={}", REPLAI_ROLE_DEFAULT);
    println!("REPLAI_ROLE_STRONG={}", REPLAI_ROLE_STRONG);
    println!("REPLAI_ROLE_ACCENT={}", REPLAI_ROLE_ACCENT);
    println!("REPLAI_ROLE_DIM={}", REPLAI_ROLE_DIM);
    println!("REPLAI_ROLE_SUCCESS={}", REPLAI_ROLE_SUCCESS);
    println!("REPLAI_ROLE_WARNING={}", REPLAI_ROLE_WARNING);
    println!("REPLAI_ROLE_ERROR={}", REPLAI_ROLE_ERROR);
    println!("replai_config.size={}", std::mem::size_of::<ReplaiConfig>());
    println!(
        "replai_config.align={}",
        std::mem::align_of::<ReplaiConfig>()
    );
    println!(
        "replai_config.struct_size={}",
        std::mem::offset_of!(ReplaiConfig, struct_size)
    );
    println!(
        "replai_config.abi_version={}",
        std::mem::offset_of!(ReplaiConfig, abi_version)
    );
    println!(
        "replai_config.max_input_bytes={}",
        std::mem::offset_of!(ReplaiConfig, max_input_bytes)
    );
    println!(
        "replai_config.history_entries={}",
        std::mem::offset_of!(ReplaiConfig, history_entries)
    );
    println!(
        "replai_config.reserved={}",
        std::mem::offset_of!(ReplaiConfig, reserved)
    );
    println!("replai_event.size={}", std::mem::size_of::<ReplaiEvent>());
    println!("replai_event.align={}", std::mem::align_of::<ReplaiEvent>());
    println!(
        "replai_event.struct_size={}",
        std::mem::offset_of!(ReplaiEvent, struct_size)
    );
    println!(
        "replai_event.abi_version={}",
        std::mem::offset_of!(ReplaiEvent, abi_version)
    );
    println!(
        "replai_event.kind={}",
        std::mem::offset_of!(ReplaiEvent, kind)
    );
    println!(
        "replai_event.status={}",
        std::mem::offset_of!(ReplaiEvent, status)
    );
    println!(
        "replai_event.text_bytes={}",
        std::mem::offset_of!(ReplaiEvent, text_bytes)
    );
    println!(
        "replai_event.cursor_bytes={}",
        std::mem::offset_of!(ReplaiEvent, cursor_bytes)
    );
    println!(
        "replai_event.reserved={}",
        std::mem::offset_of!(ReplaiEvent, reserved)
    );
}
