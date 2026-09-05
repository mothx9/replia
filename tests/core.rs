//! Deterministic editing and host-owned history contracts.
use replai::{EditError, Editor};

#[test]
fn editing_moves_and_deletes_extended_graphemes() {
    let mut e = Editor::new(128, 3);
    e.insert("aé界e\u{301}👩‍💻").unwrap();
    e.left();
    assert_eq!(&e.text()[e.cursor()..], "👩‍💻");
    e.backspace();
    assert_eq!(e.text(), "aé界👩‍💻");
    e.delete();
    assert_eq!(e.text(), "aé界");
    e.home();
    e.delete();
    e.insert("X").unwrap();
    e.end();
    e.right();
    assert_eq!(e.text(), "Xé界");
    assert_eq!(e.cursor(), e.text().len());
}

#[test]
fn capacity_and_replacement_fail_atomically() {
    let mut e = Editor::new(5, 1);
    e.insert("é界").unwrap();
    for (range, text, error) in [
        (5..5, "x", EditError::Capacity),
        (1..2, "x", EditError::InvalidRange),
        (0..2, "\x1b", EditError::InvalidText),
        (
            std::ops::Range { start: 4, end: 3 },
            "",
            EditError::InvalidRange,
        ),
    ] {
        assert_eq!(e.replace(range, text), Err(error));
        assert_eq!((e.text(), e.cursor()), ("é界", 5));
    }
    e.replace(0..2, "a").unwrap();
    assert_eq!((e.text(), e.cursor()), ("a界", 1));
}

#[test]
fn insertion_that_joins_a_grapheme_keeps_a_valid_cursor() {
    let mut e = Editor::new(100, 0);
    e.insert("👩💻").unwrap();
    e.left();
    e.insert("\u{200d}").unwrap();
    assert_eq!(e.text(), "👩‍💻");
    assert_eq!(e.cursor(), e.text().len());
    e.backspace();
    assert_eq!(e.text(), "");
}

#[test]
fn history_restores_original_text_and_cursor_without_mutating_entries() {
    let mut e = Editor::new(100, 2);
    e.history_up();
    e.history_down();
    e.admit_history("one").unwrap();
    e.admit_history("two").unwrap();
    e.insert("draft").unwrap();
    e.left();
    e.history_up();
    e.insert("!").unwrap();
    e.history_up();
    e.history_up();
    assert_eq!(e.text(), "one");
    e.history_down();
    assert_eq!(e.text(), "two");
    e.history_down();
    assert_eq!((e.text(), e.cursor()), ("draft", 4));
    e.admit_history("three").unwrap();
    e.history_up();
    e.history_up();
    assert_eq!(e.text(), "two");
}

#[test]
fn history_admission_is_explicit_and_bounded() {
    let mut e = Editor::new(4, 1);
    e.insert("keep").unwrap();
    e.clear();
    e.history_up();
    assert_eq!(e.text(), "");
    assert_eq!(e.admit_history("large"), Err(EditError::Capacity));
    e.admit_history("ok").unwrap();
    e.history_up();
    assert_eq!(e.text(), "ok");
    assert_eq!(
        Editor::new(4, 0).admit_history("ok"),
        Err(EditError::HistoryDisabled)
    );
}

#[test]
fn mixed_operations_keep_cursor_and_capacity_invariants() {
    use unicode_segmentation::UnicodeSegmentation;
    let mut editor = Editor::new(96, 3);
    let mut seed = 0x1234_u32;
    let fragments = [
        "a",
        "é",
        "界",
        "e\u{301}",
        "👩‍💻",
        "\u{200d}",
        "🇮🇹",
        "\n",
        "\t",
    ];
    for _ in 0..2000 {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        match seed % 10 {
            0..=2 => {
                let before = (editor.text().to_owned(), editor.cursor());
                if editor
                    .insert(fragments[(seed as usize / 10) % fragments.len()])
                    .is_err()
                {
                    assert_eq!(
                        (editor.text(), editor.cursor()),
                        (before.0.as_str(), before.1)
                    );
                }
            }
            3 => editor.left(),
            4 => editor.right(),
            5 => editor.backspace(),
            6 => editor.delete(),
            7 => editor.home(),
            8 => editor.end(),
            _ => {
                let text = editor.text().to_owned();
                editor.admit_history(&text).unwrap();
                editor.history_up();
                editor.history_down();
            }
        }
        assert!(editor.text().len() <= editor.capacity());
        assert!(
            editor.cursor() == editor.text().len()
                || editor
                    .text()
                    .grapheme_indices(true)
                    .any(|(i, _)| i == editor.cursor())
        );
    }
}
