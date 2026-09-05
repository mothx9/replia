//! An embeddable terminal interaction library for line-oriented and REPL-style
//! command interfaces.
//!
//! Early Rust API: deterministic Unicode editing and a Linux terminal adapter.
//! Hosts retain input meaning, completion discovery and history admission.
//!
//! ```
//! use replai::Editor;
//! let mut draft = Editor::new(1024, 20);
//! draft.insert("café 界")?;
//! draft.left();
//! draft.delete();
//! assert_eq!(draft.text(), "café ");
//! # Ok::<(), replai::EditError>(())
//! ```
//!
//! A host owns the loop, including what to do after submission or interruption:
//!
//! ```no_run
//! use replai::{Editor, Event, Prompt, Interaction};
//! use std::time::Duration;
//! let mut terminal = Interaction::new(Editor::new(65_536, 100));
//! terminal.open(
//!     &std::io::stdin(), &std::io::stdout(), Prompt::new("demo")?,
//! )?;
//! loop {
//!     match terminal.poll(Duration::from_millis(100))? {
//!         Some(Event::Submitted(text)) => { /* host consumes text */ break; }
//!         Some(Event::Interrupted | Event::EndOfInput) => break,
//!         Some(Event::CompletionRequested) => { /* host may call complete */ }
//!         Some(Event::Rejected(error)) => { /* host may report error */ }
//!         None => {}
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod core;
mod input;
mod presentation;
#[cfg(target_os = "linux")]
mod terminal;
pub use core::{EditError, Editor};
pub use presentation::{Prompt, Role, Theme};
#[cfg(target_os = "linux")]
pub use terminal::{Error, Event, Interaction};
