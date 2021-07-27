use std::process::Command;

use log::debug;
use termion::cursor;

pub fn show_message(msg: &str) {
    Command::new("tmux")
        .arg("display-popup")
        .arg("-C") // close existing popup window
        .arg("-w44")
        .arg("-h3")
        .arg("echo")
        .arg("-n")
        .arg(format!("{}{:^40}", cursor::Hide, msg))
        .spawn()
        .unwrap();
}

pub fn switch_to(target: &str) {
    debug!("switch to: {}", target);

    Command::new("tmux")
        .arg("switch-client")
        .arg("-t")
        .arg(target)
        .spawn()
        .unwrap();
}
