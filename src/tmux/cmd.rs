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
