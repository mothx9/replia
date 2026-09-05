//! Narrow, pre-release C ABI adapter using only the public safe `replai` API.
//! All pointer storage remains caller-owned except the opaque handle allocation.
#![cfg_attr(not(target_os = "linux"), allow(unused))]
#[cfg(not(target_os = "linux"))]
compile_error!("the C binding is qualified only on Linux");
#[cfg(not(panic = "unwind"))]
compile_error!("the C binding requires panic=unwind for containment");

use replai::{EditError, Editor, Error, Event, Interaction, Prompt, Role};
use std::{
    mem,
    os::fd::BorrowedFd,
    panic::{AssertUnwindSafe, catch_unwind},
    ptr,
    time::Duration,
};
mod abi;
pub use abi::*;

/// Opaque C owner. Its Rust representation is never part of the ABI.
pub struct Handle {
    interaction: Interaction,
    prompt: Prompt,
    submitted: Option<String>,
    poisoned: bool,
}

fn guard(f: impl FnOnce() -> i32) -> i32 {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(status) => status,
        Err(payload) => {
            // A panic payload may itself panic on Drop. Contain that too. The
            // secondary payload is deliberately retained in this exceptional
            // situation rather than allowing a third unwind through C.
            if let Err(secondary) = catch_unwind(AssertUnwindSafe(|| drop(payload))) {
                mem::forget(secondary);
            }
            REPLAI_INTERNAL
        }
    }
}
fn aligned<T>(p: *const T) -> Result<(), i32> {
    if p.is_null() || !p.is_aligned() || p.addr().checked_add(mem::size_of::<T>()).is_none() {
        Err(REPLAI_INVALID_ARGUMENT)
    } else {
        Ok(())
    }
}
fn span(p: *const u8, len: usize) -> Result<(), i32> {
    if len > isize::MAX as usize || (p.is_null() && len != 0) || p.addr().checked_add(len).is_none()
    {
        Err(REPLAI_INVALID_ARGUMENT)
    } else {
        Ok(())
    }
}
unsafe fn text<'a>(p: *const u8, len: usize) -> Result<&'a str, i32> {
    span(p, len)?;
    if len == 0 {
        return Ok("");
    }
    // SAFETY: caller provides readable, live bytes for this call; span rejects
    // NULL, address overflow and lengths beyond slice bounds. No reference is retained.
    let bytes = unsafe { std::slice::from_raw_parts(p, len) };
    std::str::from_utf8(bytes).map_err(|_| REPLAI_INVALID_UTF8)
}
unsafe fn record<T: Copy>(p: *const T) -> Result<T, i32> {
    aligned(p)?;
    // SAFETY: ABI record callers provide at least the size prefix, aligned as T.
    // The size is checked BEFORE any later fields are accessed.
    let size = unsafe { p.cast::<u32>().read() };
    if size as usize != mem::size_of::<T>() {
        return Err(REPLAI_ABI_MISMATCH);
    }
    // SAFETY: matching struct_size promises readable storage for the whole C
    // record. Both records have ABI version as their second u32.
    let version = unsafe { p.cast::<u32>().add(1).read() };
    if version != REPLAI_C_ABI_VERSION {
        return Err(REPLAI_ABI_MISMATCH);
    }
    // SAFETY: T is one of the Copy repr(C) ABI records, and its extent was checked.
    Ok(unsafe { p.read() })
}
unsafe fn with_handle(p: *mut Handle, f: impl FnOnce(&mut Handle) -> i32) -> i32 {
    if let Err(e) = aligned(p) {
        return e;
    }
    // SAFETY: caller owns this live handle and serializes all calls on it;
    // no alias to internal data crosses the ABI. NULL/alignment checked above.
    let h = unsafe { &mut *p };
    if h.poisoned {
        return REPLAI_INVALID_STATE;
    }
    let status = guard(|| f(h));
    if status == REPLAI_INTERNAL {
        h.poisoned = true;
        let _ = guard(|| result(h.interaction.close()));
    }
    status
}
fn edit(e: EditError) -> i32 {
    match e {
        EditError::Capacity => REPLAI_CAPACITY,
        EditError::InvalidText => REPLAI_INVALID_TEXT,
        EditError::InvalidRange => REPLAI_INVALID_RANGE,
        EditError::HistoryDisabled => REPLAI_HISTORY_DISABLED,
        EditError::InvalidUtf8 => REPLAI_INVALID_UTF8,
        EditError::InvalidSequence => REPLAI_INVALID_SEQUENCE,
    }
}
fn error(e: Error) -> i32 {
    match e {
        Error::Edit(e) => edit(e),
        Error::Io(_) => REPLAI_IO,
        Error::State => REPLAI_INVALID_STATE,
        Error::Busy => REPLAI_BUSY,
        Error::UnsuitableTerminal => REPLAI_UNSUITABLE_TERMINAL,
    }
}
fn result(r: Result<(), Error>) -> i32 {
    r.map_or_else(error, |()| REPLAI_OK)
}
fn role(tag: u32) -> Result<Role, i32> {
    match tag {
        REPLAI_ROLE_DEFAULT => Ok(Role::Default),
        REPLAI_ROLE_STRONG => Ok(Role::Strong),
        REPLAI_ROLE_ACCENT => Ok(Role::Accent),
        REPLAI_ROLE_DIM => Ok(Role::Dim),
        REPLAI_ROLE_SUCCESS => Ok(Role::Success),
        REPLAI_ROLE_WARNING => Ok(Role::Warning),
        REPLAI_ROLE_ERROR => Ok(Role::Error),
        _ => Err(REPLAI_INVALID_ARGUMENT),
    }
}
unsafe fn copy_text(s: &str, buffer: *mut u8, capacity: usize, required: *mut usize) -> i32 {
    if let Err(e) = aligned(required).and_then(|()| span(buffer, capacity)) {
        return e;
    }
    // SAFETY: required is aligned, writable caller storage, disjoint from the
    // source and buffer by contract. Writing its exact count is allowed on TOO_SMALL.
    unsafe {
        required.write(s.len());
    }
    if buffer.is_null() && capacity == 0 {
        return REPLAI_OK;
    }
    if capacity < s.len() {
        return REPLAI_BUFFER_TOO_SMALL;
    }
    if !s.is_empty() {
        // SAFETY: caller owns capacity writable bytes; checked sufficient above.
        // Source is live Rust text, never exposed as a pointer, and disjoint by contract.
        unsafe {
            ptr::copy_nonoverlapping(s.as_ptr(), buffer, s.len());
        }
    }
    REPLAI_OK
}
unsafe fn event_ready(p: *mut ReplaiEvent) -> Result<(), i32> {
    // SAFETY: caller supplies an event record under the shared record contract.
    let event = unsafe { record(p)? };
    if event.kind != REPLAI_EVENT_NONE
        || event.status != 0
        || event.text_bytes != 0
        || event.cursor_bytes != 0
        || event.reserved != [0; 2]
    {
        return Err(REPLAI_INVALID_ARGUMENT);
    }
    Ok(())
}
unsafe fn event_result(h: &mut Handle, event: Option<Event>, out: *mut ReplaiEvent) -> i32 {
    let (kind, status) = match event {
        None => (REPLAI_EVENT_NONE, REPLAI_OK),
        Some(Event::Submitted(text)) => {
            h.submitted = Some(text);
            (REPLAI_EVENT_SUBMITTED, REPLAI_OK)
        }
        Some(Event::Interrupted) => (REPLAI_EVENT_INTERRUPTED, REPLAI_OK),
        Some(Event::EndOfInput) => (REPLAI_EVENT_END_OF_INPUT, REPLAI_OK),
        Some(Event::CompletionRequested) => (REPLAI_EVENT_COMPLETION_REQUESTED, REPLAI_OK),
        Some(Event::Rejected(e)) => (REPLAI_EVENT_EDIT_REJECTED, edit(e)),
    };
    let value = ReplaiEvent {
        struct_size: mem::size_of::<ReplaiEvent>() as u32,
        abi_version: REPLAI_C_ABI_VERSION,
        kind,
        status,
        text_bytes: h.interaction.editor().text().len() as u64,
        cursor_bytes: h.interaction.editor().cursor() as u64,
        reserved: [0; 2],
    };
    // SAFETY: event_ready validated this caller-owned writable record before use.
    unsafe {
        out.write(value);
    }
    REPLAI_OK
}

// Each exported entrypoint uses the same panic guard. Unsafe regions only cross
// raw-pointer/FD boundaries; all editing and terminal operations use public APIs.

/// Query ABI identity.
/// # Safety
/// `version` must be writable aligned u32 storage, or NULL (rejected).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_abi_version(version: *mut u32) -> i32 {
    guard(|| {
        if let Err(e) = aligned(version) {
            return e;
        }
        // SAFETY: caller owns the validated aligned output storage.
        unsafe {
            version.write(REPLAI_C_ABI_VERSION);
        }
        REPLAI_OK
    })
}
/// Allocate one opaque owner.
/// # Safety
/// Record and output follow the installed header's live-storage/disjointness rules.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_create(config: *const ReplaiConfig, out: *mut *mut Handle) -> i32 {
    guard(|| {
        if let Err(e) = aligned(out) {
            return e;
        }
        // SAFETY: validated output slot is initialized caller-owned storage.
        if !unsafe { out.read() }.is_null() {
            return REPLAI_INVALID_ARGUMENT;
        }
        // SAFETY: caller supplies the versioned config record; record checks prefix first.
        let c = match unsafe { record(config) } {
            Ok(c) => c,
            Err(e) => return e,
        };
        if c.reserved != [0; 2] {
            return REPLAI_INVALID_ARGUMENT;
        }
        if c.max_input_bytes > isize::MAX as u64
            || c.history_entries > (isize::MAX as usize / mem::size_of::<String>()) as u64
        {
            return REPLAI_INVALID_ARGUMENT;
        }
        let h = Box::new(Handle {
            interaction: Interaction::new(Editor::new(
                c.max_input_bytes as usize,
                c.history_entries as usize,
            )),
            prompt: Prompt::new("").expect("empty prompt is valid"),
            submitted: None,
            poisoned: false,
        });
        // SAFETY: ownership transfers only this Box to the validated output slot.
        // replai_destroy is its sole release operation.
        unsafe {
            out.write(Box::into_raw(h));
        }
        REPLAI_OK
    })
}
/// Destroy one opaque owner, including a poisoned owner.
/// # Safety
/// `handle` is an initialized exclusive slot containing a live allocation from create.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_destroy(handle: *mut *mut Handle) -> i32 {
    guard(|| {
        if let Err(e) = aligned(handle) {
            return e;
        }
        // SAFETY: initialized, aligned caller slot is readable by contract.
        let raw = unsafe { handle.read() };
        if let Err(e) = aligned(raw) {
            return e;
        }
        // SAFETY: exactly one Box owner was transferred by create; caller now
        // relinquishes it and must not retain usable aliases after this call.
        let mut h = unsafe { Box::from_raw(raw) };
        // SAFETY: invalidate the same caller-owned slot before fallible cleanup.
        unsafe {
            handle.write(ptr::null_mut());
        }
        result(h.interaction.close())
    })
}
/// Configure literal prompt fields while closed.
/// # Safety
/// Handle and input spans follow the header's live-storage rules for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_prompt(
    handle: *mut Handle,
    label: *const u8,
    label_len: usize,
    suffix: *const u8,
    suffix_len: usize,
    continuation: *const u8,
    continuation_len: usize,
) -> i32 {
    guard(|| {
        // SAFETY: caller owns a live serialized handle; helper checks detectable misuse.
        let operation = |h: &mut Handle| {
            if h.interaction.is_open() {
                return REPLAI_INVALID_STATE;
            }
            // Input references remain within this call; only Prompt's owned strings persist.
            let make = || -> Result<Prompt, i32> {
                // SAFETY: caller provides a readable byte span for this call; no reference is retained.
                let label = unsafe { text(label, label_len) }?;
                // SAFETY: caller provides a readable byte span for this call; no reference is retained.
                let suffix = unsafe { text(suffix, suffix_len) }?;
                // SAFETY: caller provides a readable byte span for this call; no reference is retained.
                let continuation = unsafe { text(continuation, continuation_len) }?;
                Prompt::new(label)
                    .and_then(|p| p.with_state(suffix))
                    .and_then(|p| p.with_continuation(continuation))
                    .map_err(edit)
            };
            match make() {
                Ok(p) => {
                    h.prompt = p;
                    REPLAI_OK
                }
                Err(e) => e,
            }
        };
        // SAFETY: caller supplies a live, exclusively used handle for this call.
        unsafe { with_handle(handle, operation) }
    })
}
/// Acquire caller-supplied terminal descriptors.
/// # Safety
/// The live serialized handle and both FDs remain valid and unclosed throughout the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_open(handle: *mut Handle, input_fd: i32, output_fd: i32) -> i32 {
    guard(|| {
        // SAFETY: fcntl takes integer FDs, touches no caller memory and reports invalid FDs.
        if input_fd < 0 || output_fd < 0 || unsafe { libc::fcntl(input_fd, libc::F_GETFD) } < 0
            // SAFETY: same integer-only validity probe as above.
            || unsafe { libc::fcntl(output_fd, libc::F_GETFD) } < 0
        {
            return REPLAI_INVALID_ARGUMENT;
        }
        // SAFETY: validated FDs are kept live by the caller throughout this call.
        let input = unsafe { BorrowedFd::borrow_raw(input_fd) };
        // SAFETY: same FD lifetime rule; safe core duplicates before returning.
        let output = unsafe { BorrowedFd::borrow_raw(output_fd) };
        // SAFETY: handle is live/exclusive by caller contract; no FD borrow is retained.
        let operation = |h: &mut Handle| match h.interaction.open(&input, &output, h.prompt.clone())
        {
            Ok(()) => {
                h.submitted = None;
                REPLAI_OK
            }
            Err(e) => error(e),
        };
        // SAFETY: caller supplies a live, exclusively used handle for this call.
        unsafe { with_handle(handle, operation) }
    })
}
/// Close the active terminal, retaining editable state.
/// # Safety
/// The handle must be live and exclusively used for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_close(handle: *mut Handle) -> i32 {
    guard(|| {
        if let Err(e) = aligned(handle) {
            return e;
        }
        // SAFETY: caller provides live exclusive handle; close is permitted after panic.
        result(unsafe { &mut *handle }.interaction.close())
    })
}
/// Poll one interaction event.
/// # Safety
/// Handle and writable initialized event record follow the installed header contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_poll(
    handle: *mut Handle,
    timeout_ms: u32,
    event: *mut ReplaiEvent,
) -> i32 {
    guard(|| {
        // SAFETY: event is initialized readable/writable caller storage.
        if let Err(e) = unsafe { event_ready(event) } {
            return e;
        }
        // SAFETY: caller owns a live serialized handle and disjoint event storage.
        let operation = |h: &mut Handle| {
            match h.interaction.poll(Duration::from_millis(timeout_ms.into())) {
                // SAFETY: event_ready validated the initialized writable output record before polling.
                Ok(e) => unsafe { event_result(h, e, event) },
                Err(e) => error(e),
            }
        };
        // SAFETY: caller supplies a live, exclusively used handle for this call.
        unsafe { with_handle(handle, operation) }
    })
}
/// Deliver an interrupt from ordinary host control flow.
/// # Safety
/// Same handle/event record requirements as poll; never call from a signal handler.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_interrupt(handle: *mut Handle, event: *mut ReplaiEvent) -> i32 {
    guard(|| {
        // SAFETY: caller provides the initialized event record for validation.
        if let Err(e) = unsafe { event_ready(event) } {
            return e;
        }
        // SAFETY: live serialized handle and disjoint output record as documented.
        let operation = |h: &mut Handle| match h.interaction.interrupt() {
            // SAFETY: event_ready validated the initialized writable output record before polling.
            Ok(e) => unsafe { event_result(h, Some(e), event) },
            Err(e) => error(e),
        };
        // SAFETY: caller supplies a live, exclusively used handle for this call.
        unsafe { with_handle(handle, operation) }
    })
}
/// Copy/query the current draft and cursor.
/// # Safety
/// Handle, initialized outputs and writable buffer are live and mutually disjoint.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_draft_copy(
    handle: *mut Handle,
    buffer: *mut u8,
    capacity: usize,
    required: *mut usize,
    cursor: *mut usize,
) -> i32 {
    guard(|| {
        if let Err(e) = aligned(cursor) {
            return e;
        }
        // SAFETY: caller meets the live handle/output span contract. Helpers
        // validate detectable errors before writing any output buffer bytes.
        let operation = |h: &mut Handle| {
            // SAFETY: caller outputs are live, writable and disjoint; helper validates sizes first.
            let status =
                unsafe { copy_text(h.interaction.editor().text(), buffer, capacity, required) };
            if status == REPLAI_OK || status == REPLAI_BUFFER_TOO_SMALL {
                // SAFETY: aligned cursor output was validated before the handle operation.
                unsafe {
                    cursor.write(h.interaction.editor().cursor());
                }
            }
            status
        };
        // SAFETY: caller supplies a live, exclusively used handle for this call.
        unsafe { with_handle(handle, operation) }
    })
}
/// Copy/query the most recent submitted input.
/// # Safety
/// Same caller buffer/handle contract as draft_copy.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_submitted_copy(
    handle: *mut Handle,
    buffer: *mut u8,
    capacity: usize,
    required: *mut usize,
) -> i32 {
    guard(|| {
        // SAFETY: live serialized handle and writable disjoint copy outputs.
        let operation = |h: &mut Handle| match &h.submitted {
            // SAFETY: caller outputs are live, writable and disjoint; helper validates sizes first.
            Some(text) => unsafe { copy_text(text, buffer, capacity, required) },
            None => REPLAI_INVALID_STATE,
        };
        // SAFETY: caller supplies a live, exclusively used handle for this call.
        unsafe { with_handle(handle, operation) }
    })
}
/// Replace the closed draft.
/// # Safety
/// Handle and readable byte span remain live for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_set_draft(
    handle: *mut Handle,
    bytes: *const u8,
    length: usize,
) -> i32 {
    guard(|| {
        // SAFETY: caller owns live handle and readable span; only owned text is retained.
        let operation = |h: &mut Handle| {
            // SAFETY: caller provides a readable byte span for this call; no reference is retained.
            let text = match unsafe { text(bytes, length) } {
                Ok(t) => t,
                Err(e) => return e,
            };
            match h.interaction.editor_mut() {
                Ok(editor) => editor
                    .replace(0..editor.text().len(), text)
                    .map_or_else(edit, |()| REPLAI_OK),
                Err(e) => error(e),
            }
        };
        // SAFETY: caller supplies a live, exclusively used handle for this call.
        unsafe { with_handle(handle, operation) }
    })
}
/// Clear the closed editor.
/// # Safety
/// Handle must be live and serialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_clear(handle: *mut Handle) -> i32 {
    guard(|| {
        // SAFETY: caller owns the live serialized opaque handle.
        let operation = |h: &mut Handle| match h.interaction.editor_mut() {
            Ok(e) => {
                e.clear();
                REPLAI_OK
            }
            Err(e) => error(e),
        };
        // SAFETY: caller supplies a live, exclusively used handle for this call.
        unsafe { with_handle(handle, operation) }
    })
}
/// Admit host-selected history text.
/// # Safety
/// Handle and readable byte span follow the common caller contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_history_add(
    handle: *mut Handle,
    bytes: *const u8,
    length: usize,
) -> i32 {
    guard(|| {
        // SAFETY: caller supplies live handle/input; input borrows end with this call.
        let operation = |h: &mut Handle| {
            // SAFETY: caller provides a readable byte span for this call; no reference is retained.
            let text = match unsafe { text(bytes, length) } {
                Ok(t) => t,
                Err(e) => return e,
            };
            match h.interaction.editor_mut() {
                Ok(editor) => editor.admit_history(text).map_or_else(edit, |()| REPLAI_OK),
                Err(e) => error(e),
            }
        };
        // SAFETY: caller supplies a live, exclusively used handle for this call.
        unsafe { with_handle(handle, operation) }
    })
}
/// Apply a host-selected active completion replacement.
/// # Safety
/// Handle and readable byte span follow the common caller contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_complete(
    handle: *mut Handle,
    start: usize,
    end: usize,
    bytes: *const u8,
    length: usize,
) -> i32 {
    guard(|| {
        // SAFETY: caller supplies live handle/input; the safe API validates byte ranges.
        let operation = |h: &mut Handle| {
            // SAFETY: caller provides a readable byte span for this call; no reference is retained.
            let text = match unsafe { text(bytes, length) } {
                Ok(t) => t,
                Err(e) => return e,
            };
            result(h.interaction.complete(start..end, text))
        };
        // SAFETY: caller supplies a live, exclusively used handle for this call.
        unsafe { with_handle(handle, operation) }
    })
}
/// Output terminal-safe host text through the shared renderer.
/// # Safety
/// Handle and readable byte span follow the common caller contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_external_output(
    handle: *mut Handle,
    tag: u32,
    bytes: *const u8,
    length: usize,
) -> i32 {
    guard(|| {
        // SAFETY: caller supplies live handle/input; no raw ANSI or borrowed data is retained.
        let operation = |h: &mut Handle| {
            let role = match role(tag) {
                Ok(r) => r,
                Err(e) => return e,
            };
            // SAFETY: caller provides a readable byte span for this call; no reference is retained.
            let text = match unsafe { text(bytes, length) } {
                Ok(t) => t,
                Err(e) => return e,
            };
            result(h.interaction.external_output(role, text))
        };
        // SAFETY: caller supplies a live, exclusively used handle for this call.
        unsafe { with_handle(handle, operation) }
    })
}
/// Copy a generic diagnostic; machine decisions use numeric status codes.
/// # Safety
/// Buffer and required-count storage follow the common copy contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replai_status_text(
    status: i32,
    buffer: *mut u8,
    capacity: usize,
    required: *mut usize,
) -> i32 {
    guard(|| {
        let text = match status {
            REPLAI_OK => "success",
            REPLAI_INVALID_ARGUMENT => "invalid argument",
            REPLAI_INVALID_UTF8 => "invalid UTF-8",
            REPLAI_INVALID_RANGE => "invalid grapheme range",
            REPLAI_CAPACITY => "capacity exceeded",
            REPLAI_INVALID_STATE => "invalid lifecycle state",
            REPLAI_UNSUITABLE_TERMINAL => "unsuitable terminal pair",
            REPLAI_IO => "terminal I/O or restoration failure",
            REPLAI_BUFFER_TOO_SMALL => "buffer too small",
            REPLAI_ABI_MISMATCH => "ABI size or version mismatch",
            REPLAI_BUSY => "terminal already owned",
            REPLAI_INTERNAL => "internal panic contained; close or destroy",
            REPLAI_INVALID_TEXT => "unsupported control text",
            REPLAI_HISTORY_DISABLED => "history disabled",
            REPLAI_INVALID_SEQUENCE => "invalid terminal sequence",
            _ => return REPLAI_INVALID_ARGUMENT,
        };
        // SAFETY: caller provides writable disjoint output spans per copy contract.
        unsafe { copy_text(text, buffer, capacity, required) }
    })
}
include!("signatures.rs");

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn panic_guard_contains_unwinding_and_a_panicking_payload_destructor() {
        assert_eq!(
            guard(|| panic!("unexpected internal failure")),
            REPLAI_INTERNAL
        );
        struct Payload;
        impl Drop for Payload {
            fn drop(&mut self) {
                panic!("payload destructor");
            }
        }
        assert_eq!(guard(|| std::panic::panic_any(Payload)), REPLAI_INTERNAL);
        assert_eq!(guard(|| REPLAI_OK), REPLAI_OK);
    }
}
