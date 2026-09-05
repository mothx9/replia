//! A neutral host loop: echo input, complete a small vocabulary, optionally emit a notice.
use replia::{Editor, Event, Prompt, Role, Terminal};
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut editor = Editor::new(65_536, 100);
    let notice = std::env::args().any(|a| a == "--notice");
    let mut notice_sent = false;
    loop {
        let event;
        {
            let mut terminal = Terminal::open(
                &std::io::stdin(),
                &std::io::stdout(),
                &mut editor,
                Prompt::new("demo")?,
            )?;
            let started = Instant::now();
            loop {
                if notice && !notice_sent && started.elapsed() >= Duration::from_secs(2) {
                    terminal.external_output(Role::Dim, "notice: the draft is still yours")?;
                    notice_sent = true;
                }
                match terminal.poll(Duration::from_millis(100))? {
                    None => {}
                    Some(Event::CompletionRequested) => {
                        let text = terminal.editor().text();
                        let matches: Vec<_> = ["hello", "help", "world"]
                            .into_iter()
                            .filter(|s| s.starts_with(text))
                            .collect();
                        if let [replacement] = matches.as_slice() {
                            terminal.complete(0..text.len(), replacement)?;
                        }
                    }
                    Some(Event::Rejected(error)) => {
                        terminal.external_output(Role::Warning, &error.to_string())?
                    }
                    Some(outcome) => {
                        event = outcome;
                        break;
                    }
                }
            }
        }
        match event {
            Event::Submitted(text) => {
                println!("echo: {text}\n");
                if !text.is_empty() {
                    editor.admit_history(&text)?;
                }
                editor.clear();
            }
            Event::Interrupted => editor.clear(),
            Event::EndOfInput => break,
            _ => unreachable!(),
        }
    }
    Ok(())
}
