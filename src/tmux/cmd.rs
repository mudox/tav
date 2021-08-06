use std::process::Command;

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

pub fn is_in_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}
