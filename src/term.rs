use std::process::Command;
use std::str;

use termion::{color, style};

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
