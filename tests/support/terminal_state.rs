//! Independent VT oracle for captured C-process terminal bytes; no REPLAI API use.
use std::io::{self, BufRead};
fn main() {
    let mut p = vt100::Parser::new(24, 80, 1000);
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        if let Some(size) = line.strip_prefix("R ") {
            let n: Vec<u16> = size
                .split_whitespace()
                .map(|x| x.parse().unwrap())
                .collect();
            p.screen_mut().set_size(n[0], n[1]);
        } else if let Some(hex) = line.strip_prefix("D ") {
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            p.process(&bytes);
        }
    }
    let s = p.screen();
    let (row, col) = s.cursor_position();
    println!("cursor {row} {col}");
    println!("text {}", hex(s.contents().as_bytes()));
    println!("alternate {}", s.alternate_screen());
    println!("paste {}", s.bracketed_paste());
    let (rows, cols) = s.size();
    let mut background_default = true;
    for row in 0..rows {
        for col in 0..cols {
            let c = s.cell(row, col).unwrap();
            background_default &= c.bgcolor() == vt100::Color::Default;
            if !c.contents().is_empty() {
                println!(
                    "cell {row} {col} {} {:?} {:?} {}",
                    hex(c.contents().as_bytes()),
                    c.fgcolor(),
                    c.bgcolor(),
                    c.bold()
                );
            }
        }
    }
    println!("background_default {background_default}");
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
