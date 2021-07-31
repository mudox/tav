use std::process::Command;
use std::str;

use termion::{color, style};

use log::debug;

/// Apply style and reset.
pub fn apply_color<S: std::fmt::Display>(style: S, text: &str) -> String {
    format!("{}{}{}", style, text, style::Reset)
}

/// Apply foreground color and reset.
pub fn fg<C: color::Color>(color: C, text: &str) -> String {
    apply_color(color::Fg(color), text)
}

/// Return hidden bar string.
pub fn span(width: usize) -> String {
    let r = format!("{:·^1$}", "·", width);
    fg(color::Black, &r)
}

/// Return terminal size.
pub fn size() -> (usize, usize) {
    let output = Command::new("tmux")
        .arg("list-clients")
        .arg("-F")
        .arg("#{client_width}\t#{client_height}")
        .output()
        .unwrap();

    let output = str::from_utf8(output.stdout.as_slice()).unwrap().trim();
    let mut tokens = output.split("\t");

    let width: usize = tokens.next().unwrap().parse().unwrap();
    let height: usize = tokens.next().unwrap().parse().unwrap();

    (width, height)
}
