use crate::{
    EditError, Editor, Prompt, Role, Theme,
    input::{Decoder, Key},
    presentation::Frame,
};
use rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    io::{dup, read, write},
    termios::{self, OptionalActions, Termios},
};
use std::{
    fmt, io,
    ops::Range,
    os::fd::{AsFd, OwnedFd},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

const PASTE_ON: &[u8] = b"\x1b[?2004h";
const PASTE_OFF: &[u8] = b"\x1b[?2004l";
const CLEAR: &[u8] = b"\x1b[2J\x1b[H";
const SEQUENCE_IDLE: Duration = Duration::from_millis(250);
static ACTIVE: AtomicBool = AtomicBool::new(false);

/// Recoverable edit rejection or terminal I/O failure.
#[derive(Debug)]
pub enum Error {
    /// Operation requires a different open/closed interaction state.
    State,
    /// Another interaction already owns a terminal in this process.
    Busy,
    /// Input/output are not a matching, suitably sized terminal pair.
    UnsuitableTerminal,
    /// An invalid edit; the terminal remains active and the draft is unchanged.
    Edit(EditError),
    /// Terminal failure. Cleanup has been attempted; a cleanup failure is included in the message.
    Io(io::Error),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::State => f.write_str("invalid interaction state"),
            Self::Busy => f.write_str("another terminal interaction is active"),
            Self::UnsuitableTerminal => {
                f.write_str("unsuitable terminal descriptors or dimensions")
            }
            Self::Edit(e) => e.fmt(f),
            Self::Io(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Edit(e) => Some(e),
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<EditError> for Error {
    fn from(e: EditError) -> Self {
        Self::Edit(e)
    }
}

/// A host-visible interaction event; input is never interpreted by the library.
#[derive(Debug, PartialEq)]
pub enum Event {
    /// Enter submitted the complete input, possibly containing newlines. Terminal restored.
    Submitted(String),
    /// Ctrl-C or explicit host interruption. Terminal restored; draft remains available.
    Interrupted,
    /// Read EOF or Ctrl-D on an empty buffer. Terminal restored.
    EndOfInput,
    /// Tab requests host completion using [`Interaction::editor`].
    CompletionRequested,
    /// Invalid input or capacity rejection; the unchanged draft remains editable.
    Rejected(EditError),
}

/// Host-owned editor and scoped Linux terminal resource, with no borrowed state.
///
/// Moving this value is safe. `open` duplicates caller FDs. Closing, submission,
/// EOF and interruption release those duplicates while retaining the editor.
/// No signal handlers or threads are installed. Only one active terminal is
/// admitted per process. Hosts serialize output and retain application meaning.
/// Drop attempts restoration without panicking; explicit close reports errors.
pub struct Interaction {
    editor: Editor,
    terminal: Option<Terminal>,
}
impl Interaction {
    /// Own an editor without acquiring a terminal.
    pub fn new(editor: Editor) -> Self {
        Self {
            editor,
            terminal: None,
        }
    }
    /// Inspect draft text and byte cursor in any lifecycle state.
    pub fn editor(&self) -> &Editor {
        &self.editor
    }
    /// Mutate editing/history state while closed; active edits use `complete`.
    pub fn editor_mut(&mut self) -> Result<&mut Editor, Error> {
        if self.is_open() {
            Err(Error::State)
        } else {
            Ok(&mut self.editor)
        }
    }
    /// Whether a terminal resource is currently owned.
    pub fn is_open(&self) -> bool {
        self.terminal.is_some()
    }
    /// Acquire a matching Linux TTY pair and draw the retained draft.
    ///
    /// Caller descriptors stay owned by the caller. Failure preserves the editor;
    /// successful close permits reopening with different descriptors or prompt.
    pub fn open(
        &mut self,
        input: &impl AsFd,
        output: &impl AsFd,
        prompt: Prompt,
    ) -> Result<(), Error> {
        if self.is_open() {
            return Err(Error::State);
        }
        self.terminal = Some(Terminal::open(input, output, &self.editor, prompt)?);
        Ok(())
    }
    /// Poll for an event, waiting at most 100 ms. Requires an open interaction.
    /// Pending sequences expire after 250 ms idle; incomplete paste closes with
    /// an error. Terminal outcomes restore termios and release duplicated FDs.
    pub fn poll(&mut self, timeout: Duration) -> Result<Option<Event>, Error> {
        let result = self
            .terminal
            .as_mut()
            .ok_or(Error::State)?
            .poll(&mut self.editor, timeout);
        self.reap();
        result
    }
    /// Apply a host-selected, grapheme-aligned completion to an active draft.
    /// Invalid edits preserve text/cursor; zero/ambiguous matches need no call.
    pub fn complete(&mut self, range: Range<usize>, replacement: &str) -> Result<(), Error> {
        let result = self.terminal.as_mut().ok_or(Error::State)?.complete(
            &mut self.editor,
            range,
            replacement,
        );
        self.reap();
        result
    }
    /// Write safe host text as a line transaction and restore draft/cursor.
    /// Controls other than LF/TAB (or CRLF) reject before output. Input stays raw
    /// and queued bytes are retained; direct concurrent terminal writes are unsupported.
    pub fn external_output(&mut self, role: Role, text: &str) -> Result<(), Error> {
        let result =
            self.terminal
                .as_mut()
                .ok_or(Error::State)?
                .external_output(&self.editor, role, text);
        self.reap();
        result
    }
    /// Deliver a host-observed interrupt and restore/release the terminal.
    /// This method is ordinary control flow, not an async-signal-safe handler.
    pub fn interrupt(&mut self) -> Result<Event, Error> {
        let result = self.terminal.as_mut().ok_or(Error::State)?.interrupt();
        self.reap();
        result
    }
    /// Restore and release the terminal, retaining editor/history. Idempotent.
    pub fn close(&mut self) -> Result<(), Error> {
        self.terminal
            .take()
            .map_or(Ok(()), |mut terminal| terminal.close())
    }
    fn reap(&mut self) {
        if self.terminal.as_ref().is_some_and(|t| !t.active) {
            self.terminal.take();
        }
    }
}

struct Lease;
impl Lease {
    fn acquire() -> io::Result<Self> {
        ACTIVE
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "another terminal interaction is active in this process",
                )
            })?;
        Ok(Self)
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        ACTIVE.store(false, Ordering::Release);
    }
}

struct Terminal {
    input: OwnedFd,
    output: OwnedFd,
    saved: Termios,
    active: bool,
    lease: Option<Lease>,
    prompt: Prompt,
    theme: Theme,
    size: (usize, usize),
    frame: Option<Frame>,
    decoder: Decoder,
    last_byte: Instant,
}

impl Terminal {
    /// Acquire TTY input/output, capture termios, enable editing and draw a prompt.
    ///
    /// Both descriptors must refer to the same terminal. Descriptors are duplicated
    /// and remain owned until this value is dropped. Environment color policy is
    /// captured at acquisition. No input queue is flushed.
    fn open(
        input: &impl AsFd,
        output: &impl AsFd,
        editor: &Editor,
        prompt: Prompt,
    ) -> Result<Self, Error> {
        if !termios::isatty(input) || !termios::isatty(output) {
            return Err(Error::UnsuitableTerminal);
        }
        let istat = rustix::fs::fstat(input).map_err(io::Error::from)?;
        let ostat = rustix::fs::fstat(output).map_err(io::Error::from)?;
        if istat.st_rdev != ostat.st_rdev {
            return Err(Error::UnsuitableTerminal);
        }
        let lease = Lease::acquire().map_err(|_| Error::Busy)?;
        let input = dup(input).map_err(io::Error::from)?;
        let output = dup(output).map_err(io::Error::from)?;
        let saved = termios::tcgetattr(&input).map_err(io::Error::from)?;
        let size = dimensions(&output).map_err(|e| {
            if e.kind() == io::ErrorKind::Unsupported {
                Error::UnsuitableTerminal
            } else {
                Error::Io(e)
            }
        })?;
        let mut raw = saved.clone();
        raw.make_raw();
        let limit = editor.capacity();
        let mut terminal = Self {
            input,
            output,
            saved,
            active: false,
            lease: Some(lease),
            prompt,
            theme: Theme::from_environment(true),
            size,
            frame: None,
            decoder: Decoder::new(limit),
            last_byte: Instant::now(),
        };
        // Mark cleanup required before attempting mutation: also covers partial setup.
        terminal.active = true;
        if let Err(e) = termios::tcsetattr(&terminal.input, OptionalActions::Now, &raw) {
            return Err(terminal.failure(e.into()));
        }
        if let Err(e) = write_all(&terminal.output, PASTE_ON).and_then(|()| terminal.redraw(editor))
        {
            return Err(terminal.failure(e));
        }
        Ok(terminal)
    }
    /// Wait up to the requested duration for one event; return None on a tick or edit.
    ///
    /// Waits are capped at 100 ms so dimensions can be observed without owning
    /// SIGWINCH. Unknown/incomplete sequences expire after 250 ms without bytes.
    /// An incomplete paste is a terminal error and closes this interaction, so
    /// its trailing bytes cannot silently become submitted commands.
    fn poll(&mut self, editor: &mut Editor, timeout: Duration) -> Result<Option<Event>, Error> {
        self.ensure_active()?;
        match self.poll_inner(editor, timeout) {
            Ok(event) => Ok(event),
            Err(error) => Err(self.failure(error)),
        }
    }
    fn poll_inner(&mut self, editor: &mut Editor, timeout: Duration) -> io::Result<Option<Event>> {
        let size = dimensions(&self.output)?;
        if size != self.size {
            self.size = size;
            self.redraw(editor)?;
        }
        let duration = timeout.min(Duration::from_millis(100));
        let ts = Timespec {
            tv_sec: 0,
            tv_nsec: duration.as_nanos() as _,
        };
        let mut fds = [PollFd::new(&self.input, PollFlags::IN)];
        let available = match poll(&mut fds, Some(&ts)) {
            Ok(n) => n,
            Err(rustix::io::Errno::INTR) => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let key = if available == 0 {
            if self.decoder.pending() && self.last_byte.elapsed() >= SEQUENCE_IDLE {
                self.decoder.expire()
            } else {
                None
            }
        } else {
            let mut byte = [0];
            match read(&self.input, &mut byte) {
                Ok(0) => {
                    if self.decoder.pending() {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "end of input during an incomplete sequence",
                        ));
                    }
                    return self.finish(Event::EndOfInput).map(Some);
                }
                Ok(_) => {
                    self.last_byte = Instant::now();
                    self.decoder.feed(byte[0])
                }
                Err(rustix::io::Errno::INTR | rustix::io::Errno::AGAIN) => return Ok(None),
                Err(e) => return Err(e.into()),
            }
        };
        let Some(key) = key else {
            return Ok(None);
        };
        match key {
            Key::Enter => {
                return self
                    .finish(Event::Submitted(editor.text().to_owned()))
                    .map(Some);
            }
            Key::Interrupt => return self.finish(Event::Interrupted).map(Some),
            Key::Eof if editor.text().is_empty() => {
                return self.finish(Event::EndOfInput).map(Some);
            }
            Key::Eof | Key::Delete => editor.delete(),
            Key::Text(text) => {
                if let Err(error) = editor.insert(&text) {
                    return Ok(Some(Event::Rejected(error)));
                }
            }
            Key::Backspace => editor.backspace(),
            Key::Left => editor.left(),
            Key::Right => editor.right(),
            Key::Home => editor.home(),
            Key::End => editor.end(),
            Key::Up => editor.history_up(),
            Key::Down => editor.history_down(),
            Key::Tab => return Ok(Some(Event::CompletionRequested)),
            Key::Rejected(error) => return Ok(Some(Event::Rejected(error))),
            Key::IncompletePaste => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "incomplete bracketed paste; interaction closed",
                ));
            }
            Key::Clear => {
                write_all(&self.output, CLEAR)?;
                self.frame = None;
            }
        }
        self.redraw(editor)?;
        Ok(None)
    }
    /// Apply one host-selected completion. Invalid edits preserve draft and display.
    /// For zero/multiple candidates or host failure, leave the draft unchanged by
    /// making no call; candidate discovery and selection remain host-owned.
    fn complete(
        &mut self,
        editor: &mut Editor,
        range: Range<usize>,
        replacement: &str,
    ) -> Result<(), Error> {
        self.ensure_active()?;
        editor.replace(range, replacement)?;
        self.redraw(editor).map_err(|e| self.failure(e))
    }
    /// Write terminal-safe host text while preserving the exact draft and cursor.
    ///
    /// This synchronous display transaction disables paste framing, clears the
    /// editing surface, writes text with a generic role, adds a final newline if
    /// needed, and redraws. LF/CRLF are normalized. Other controls except TAB are
    /// rejected before mutation. Raw input ownership is retained; queued input
    /// is never flushed. Host content is neither parsed nor interpreted.
    fn external_output(&mut self, editor: &Editor, role: Role, text: &str) -> Result<(), Error> {
        self.ensure_active()?;
        let normalized = text.replace("\r\n", "\n");
        if !crate::core::valid_text(&normalized) {
            return Err(EditError::InvalidText.into());
        }
        let result = (|| {
            write_all(&self.output, PASTE_OFF)?;
            self.erase()?;
            write_all(&self.output, self.theme.sequence(role).as_bytes())?;
            write_all(&self.output, normalized.replace('\n', "\r\n").as_bytes())?;
            write_all(&self.output, self.theme.sequence(Role::Default).as_bytes())?;
            if !normalized.ends_with('\n') {
                write_all(&self.output, b"\r\n")?;
            }
            write_all(&self.output, PASTE_ON)?;
            self.redraw(editor)
        })();
        result.map_err(|e| self.failure(e))
    }
    /// Deliver a host-observed interrupt without executing application cancellation.
    pub fn interrupt(&mut self) -> Result<Event, Error> {
        self.ensure_active()?;
        self.finish(Event::Interrupted).map_err(|e| self.failure(e))
    }
    /// End editing, leave the visible input in scrollback and restore captured termios.
    /// Idempotent after successful cleanup; reports failures rather than panicking.
    pub fn close(&mut self) -> Result<(), Error> {
        if !self.active {
            return Ok(());
        }
        let result = self.leave_line();
        let restored = self.restore();
        combine(result, restored).map_err(Error::Io)
    }
    fn ensure_active(&self) -> Result<(), Error> {
        if self.active {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "terminal interaction is closed",
            )
            .into())
        }
    }
    fn finish(&mut self, event: Event) -> io::Result<Event> {
        let output = (|| {
            if event == Event::Interrupted {
                self.move_to_end()?;
                write_all(&self.output, b"^C\r\n")
            } else {
                self.leave_line()
            }
        })();
        let restored = self.restore();
        combine(output, restored)?;
        Ok(event)
    }
    fn leave_line(&mut self) -> io::Result<()> {
        self.move_to_end()?;
        write_all(&self.output, b"\r\n")
    }
    fn move_to_end(&mut self) -> io::Result<()> {
        if let Some(frame) = self.frame.take() {
            let down = frame.lines.len() - 1 - frame.cursor.row;
            if down > 0 {
                write_all(&self.output, format!("\x1b[{down}B").as_bytes())?;
            }
            write_all(&self.output, b"\r")?;
            if frame.end.col > 0 {
                write_all(&self.output, format!("\x1b[{}C", frame.end.col).as_bytes())?;
            }
        }
        Ok(())
    }
    fn erase(&mut self) -> io::Result<()> {
        if let Some(frame) = self.frame.take() {
            write_all(&self.output, frame.erase().as_bytes())?;
        } else {
            write_all(&self.output, b"\r\x1b[2K")?;
        }
        Ok(())
    }
    fn redraw(&mut self, editor: &Editor) -> io::Result<()> {
        let frame = Frame::new(editor, &self.prompt, self.theme, self.size.0, self.size.1);
        if let Some(old) = &self.frame
            && old.cursor == old.end
            && frame.cursor == frame.end
            && old.lines.len() == frame.lines.len()
            && old.lines[..old.lines.len() - 1] == frame.lines[..frame.lines.len() - 1]
            && let Some(suffix) = frame
                .lines
                .last()
                .unwrap()
                .strip_prefix(old.lines.last().unwrap())
        {
            write_all(&self.output, suffix.as_bytes())?;
            self.frame = Some(frame);
            return Ok(());
        }
        self.erase()?;
        write_all(&self.output, frame.draw().as_bytes())?;
        self.frame = Some(frame);
        Ok(())
    }
    fn restore(&mut self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }
        let cleanup = [PASTE_OFF, self.theme.sequence(Role::Default).as_bytes()].concat();
        let output = write_all(&self.output, &cleanup);
        if output.is_err() {
            // Both descriptors name the same TTY. A readable/writable input can
            // still restore protocol modes if the output descriptor has failed.
            let _ = write_all(&self.input, &cleanup);
        }
        let restored = termios::tcsetattr(&self.input, OptionalActions::Now, &self.saved)
            .map_err(io::Error::from);
        if restored.is_ok() {
            self.active = false;
            self.lease.take();
        }
        combine(output, restored)
    }
    fn failure(&mut self, error: io::Error) -> Error {
        let restored = self.restore();
        Error::Io(match restored {
            Ok(()) => error,
            Err(cleanup) => io::Error::new(
                error.kind(),
                format!("{error}; terminal cleanup also failed: {cleanup}"),
            ),
        })
    }
}
impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}
fn combine(first: io::Result<()>, second: io::Result<()>) -> io::Result<()> {
    match (first, second) {
        (Err(a), Err(b)) => Err(io::Error::new(
            a.kind(),
            format!("{a}; terminal cleanup also failed: {b}"),
        )),
        (Err(e), _) | (_, Err(e)) => Err(e),
        _ => Ok(()),
    }
}
fn dimensions(output: &impl AsFd) -> io::Result<(usize, usize)> {
    let size = termios::tcgetwinsize(output)?;
    if size.ws_col < 2 || size.ws_row < 2 {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "terminal must provide at least 2 columns and 2 rows",
        ));
    }
    Ok((size.ws_col.into(), size.ws_row.into()))
}
fn write_all(output: &impl AsFd, mut bytes: &[u8]) -> io::Result<()> {
    while !bytes.is_empty() {
        match write(output, bytes) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "terminal write made no progress",
                ));
            }
            Ok(n) => bytes = &bytes[n..],
            Err(rustix::io::Errno::INTR) => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn write_failure_during_active_output_restores_termios_and_paste_mode() {
        use rustix::{
            fs::{Mode, OFlags, open},
            pty::{OpenptFlags, ioctl_tiocgptpeer, openpt, unlockpt},
            termios::{Winsize, tcgetattr, tcsetwinsize, ttyname},
        };
        let flags = OpenptFlags::RDWR | OpenptFlags::NOCTTY | OpenptFlags::CLOEXEC;
        let master = openpt(flags).unwrap();
        unlockpt(&master).unwrap();
        let slave = ioctl_tiocgptpeer(&master, flags).unwrap();
        tcsetwinsize(
            &slave,
            Winsize {
                ws_col: 80,
                ws_row: 24,
                ws_xpixel: 0,
                ws_ypixel: 0,
            },
        )
        .unwrap();
        let before = format!("{:?}", tcgetattr(&slave).unwrap());
        let mut editor = Editor::new(100, 1);
        editor.insert("draft").unwrap();
        editor.left();
        let mut t = Terminal::open(&slave, &slave, &editor, Prompt::new("demo").unwrap()).unwrap();
        let mut bytes = [0; 4096];
        read(&master, &mut bytes).unwrap();
        // Replace only the owned output FD with a real read-only handle to the
        // same PTY, after successful acquisition. No mocked writes or cleanup.
        t.output = open(
            ttyname(&slave, Vec::new()).unwrap(),
            OFlags::RDONLY | OFlags::NOCTTY,
            Mode::empty(),
        )
        .unwrap();
        assert!(matches!(
            t.external_output(&editor, Role::Dim, "notice"),
            Err(Error::Io(_))
        ));
        assert_eq!(format!("{:?}", tcgetattr(&slave).unwrap()), before);
        assert_eq!((editor.text(), editor.cursor()), ("draft", 4));
        let n = read(&master, &mut bytes).unwrap();
        assert!(bytes[..n].windows(8).any(|w| w == PASTE_OFF));
        assert!(!t.active);
    }
}
