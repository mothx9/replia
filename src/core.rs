use std::{collections::VecDeque, fmt, ops::Range};
use unicode_segmentation::UnicodeSegmentation;

/// A rejected edit. Rejection leaves the text and cursor unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EditError {
    /// The configured UTF-8 byte capacity would be exceeded.
    Capacity,
    /// Text contains a control character other than LF or TAB.
    InvalidText,
    /// A replacement range is reversed, outside the buffer, or splits a grapheme.
    InvalidRange,
    /// The host configured zero history entries.
    HistoryDisabled,
    /// Input was not valid, complete UTF-8.
    InvalidUtf8,
    /// An unknown or incomplete terminal sequence was rejected.
    InvalidSequence,
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Capacity => "input capacity exceeded",
            Self::InvalidText => "unsupported control character",
            Self::InvalidRange => "replacement must use ordered grapheme boundaries",
            Self::HistoryDisabled => "history is disabled",
            Self::InvalidUtf8 => "invalid or incomplete UTF-8",
            Self::InvalidSequence => "unknown or incomplete terminal sequence",
        })
    }
}
impl std::error::Error for EditError {}

pub(crate) fn valid_text(text: &str) -> bool {
    text.chars()
        .all(|c| !c.is_control() || c == '\n' || c == '\t')
}

/// Deterministic bounded text editing, with extended-grapheme cursor movement.
///
/// Offsets are UTF-8 byte offsets at extended grapheme boundaries. Home and End
/// refer to the entire input, including multiline input. History admission is
/// explicit; navigating an edited recalled entry never changes stored history.
#[derive(Debug)]
pub struct Editor {
    text: String,
    cursor: usize,
    limit: usize,
    history: VecDeque<String>,
    history_limit: usize,
    selected: Option<usize>,
    draft: Option<(String, usize)>,
}

impl Editor {
    /// Create an empty editor with byte and history-entry limits. Zero is allowed.
    pub fn new(max_bytes: usize, history_entries: usize) -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            limit: max_bytes,
            history: VecDeque::new(),
            history_limit: history_entries,
            selected: None,
            draft: None,
        }
    }
    /// Current input, without application interpretation.
    pub fn text(&self) -> &str {
        &self.text
    }
    /// Cursor as a UTF-8 byte offset at an extended grapheme boundary.
    pub fn cursor(&self) -> usize {
        self.cursor
    }
    /// Configured maximum input size in UTF-8 bytes.
    pub fn capacity(&self) -> usize {
        self.limit
    }
    fn boundary(&self, index: usize) -> bool {
        index == self.text.len() || self.text.grapheme_indices(true).any(|(i, _)| i == index)
    }
    /// Replace a grapheme-aligned byte range atomically. Cursor follows replacement.
    pub fn replace(&mut self, range: Range<usize>, text: &str) -> Result<(), EditError> {
        if range.start > range.end
            || range.end > self.text.len()
            || !self.boundary(range.start)
            || !self.boundary(range.end)
        {
            return Err(EditError::InvalidRange);
        }
        if !valid_text(text) {
            return Err(EditError::InvalidText);
        }
        let remaining = self.text.len() - range.len();
        if text.len() > self.limit - remaining {
            return Err(EditError::Capacity);
        }
        let wanted = range.start + text.len();
        self.text.replace_range(range, text);
        // Joining marks/ZWJ may merge both sides of the insertion.
        self.cursor = self
            .text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .find(|&i| i >= wanted)
            .unwrap_or(self.text.len());
        Ok(())
    }
    /// Insert text at the cursor without interpreting commands or trimming it.
    pub fn insert(&mut self, text: &str) -> Result<(), EditError> {
        self.replace(self.cursor..self.cursor, text)
    }
    /// Clear the active input and navigation draft; retain admitted history.
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
        self.selected = None;
        self.draft = None;
    }
    /// Move left one extended grapheme, stopping at the beginning.
    pub fn left(&mut self) {
        self.cursor = self.text[..self.cursor]
            .grapheme_indices(true)
            .next_back()
            .map_or(0, |(i, _)| i);
    }
    /// Move right one extended grapheme, stopping at the end.
    pub fn right(&mut self) {
        if let Some(g) = self.text[self.cursor..].graphemes(true).next() {
            self.cursor += g.len();
        }
    }
    /// Move to the beginning of the entire input.
    pub fn home(&mut self) {
        self.cursor = 0;
    }
    /// Move to the end of the entire input.
    pub fn end(&mut self) {
        self.cursor = self.text.len();
    }
    /// Remove the preceding extended grapheme, or do nothing at the beginning.
    pub fn backspace(&mut self) {
        let end = self.cursor;
        self.left();
        self.text.drain(self.cursor..end);
        self.snap_cursor();
    }
    /// Remove the following extended grapheme, or do nothing at the end.
    pub fn delete(&mut self) {
        let start = self.cursor;
        self.right();
        self.text.drain(start..self.cursor);
        self.cursor = start;
        self.snap_cursor();
    }
    fn snap_cursor(&mut self) {
        self.cursor = self
            .text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .find(|&i| i >= self.cursor)
            .unwrap_or(self.text.len());
    }
    /// Admit one host-selected history entry; evict the oldest at capacity.
    ///
    /// No deduplication, trimming, persistence or automatic admission occurs.
    pub fn admit_history(&mut self, text: &str) -> Result<(), EditError> {
        if !valid_text(text) {
            return Err(EditError::InvalidText);
        }
        if text.len() > self.limit {
            return Err(EditError::Capacity);
        }
        if self.history_limit == 0 {
            return Err(EditError::HistoryDisabled);
        }
        self.selected = None;
        self.draft = None;
        if self.history.len() == self.history_limit {
            self.history.pop_front();
        }
        self.history.push_back(text.to_owned());
        Ok(())
    }
    /// Recall the previous entry, preserving the current draft and its cursor.
    pub fn history_up(&mut self) {
        if self.history.is_empty() || self.selected == Some(0) {
            return;
        }
        let index = self.selected.map_or_else(
            || {
                self.draft = Some((self.text.clone(), self.cursor));
                self.history.len() - 1
            },
            |i| i - 1,
        );
        self.selected = Some(index);
        self.text.clone_from(&self.history[index]);
        self.end();
    }
    /// Recall the next entry, or return to the original draft and cursor.
    pub fn history_down(&mut self) {
        let Some(index) = self.selected else {
            return;
        };
        if index + 1 < self.history.len() {
            self.selected = Some(index + 1);
            self.text.clone_from(&self.history[index + 1]);
            self.end();
        } else {
            if let Some((text, cursor)) = self.draft.take() {
                self.text = text;
                self.cursor = cursor;
            }
            self.selected = None;
        }
    }
}
