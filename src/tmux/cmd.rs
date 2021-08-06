use std::process::Command;
use std::str;

use log::debug;

pub fn switch_to(target: &str) {
    debug!("switch to: {}", target);

    Command::new("tmux")
        .arg("switch-client")
        .arg("-t")
        .arg(target)
        .spawn()
        .unwrap();
}

#[inline]
pub fn is_in_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

/// Return tmux client size.
pub fn client_size() -> (usize, usize) {
    let output = Command::new("tmux")
        .arg("display-message")
        .arg("-p")
        .arg("#{client_width}\t#{client_height}")
        .output()
        .unwrap();

    let output = str::from_utf8(&output.stdout).unwrap().trim();
    let mut tokens = output.split("\t");

    let width: usize = tokens.next().unwrap().parse().unwrap();
    let height: usize = tokens.next().unwrap().parse().unwrap();

    (width, height)
}
