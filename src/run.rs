use crate::config::Config;
use crate::fzf::Formatter;
use crate::tmux::{cmd as tmux, snapshot};

use log::{debug, error};

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str;

pub fn run(config: Config) {
    match choose_window(config.clone()) {
        Some(id) => {
            if id.starts_with("$") || id.starts_with("@") {
                tmux::switch_to(&id);
            } else if id.starts_with("[dead]") {
                create_session(&id[6..], config.clone());
            }
        }
        None => debug!("quit with noop"),
    }
}

/// Run `fzf-cmd` to let user choose a window.
fn choose_window(config: Config) -> Option<String> {
    //
    // take snapshot
    //

    let tmux = snapshot::create();

    let window_width = tmux.geometry.window_width;
    let window_height = tmux.geometry.window_height;
    debug!("win width: {}", window_width);
    debug!("win height: {}", window_height);

    //
    // generate fzf feed
    //

    let fzf = Formatter::new(tmux, config);
    let width = fzf.width + 4 * 2 + 5 - 2; // 💀 magic number
    debug!("fzf height: {}", fzf.height);

    let mut height = fzf.height + 2 * 2 + 5 + 1;
    height = height.min(window_height - 16);

    let feed = fzf
        .feed
        .into_iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .join("\n");

    //
    // choose window id
    //

    let mut cmd = Command::new("fzf-tmux");
    let mut cmd_mut_ref = cmd
        .arg("-w") // popup window width
        .arg(width.to_string())
        .arg("-h") // popup window height
        .arg(height.to_string())
        .arg("--with-nth=2..")
        .arg("--no-sort")
        // .arg("--exact")
        .arg("--ansi")
        .arg("--margin=2,4,2,2") // 💀 magic number
        .arg("--inline-info")
        .arg("--header")
        .arg("") // sepratate line
        .arg("--prompt=▶ ")
        .arg("--pointer=▶");

    for key in [/*"esc",*/ "ctrl-c", "ctrl-g", "ctrl-q"] {
        cmd_mut_ref = cmd_mut_ref.arg(format!("--bind={}:unix-line-discard", key));
    }

    let mut child = cmd_mut_ref
        .arg("--color=bg:-1,bg+:-1") // transparent background
        .arg("--border=none")
        // pipe
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        // run
        .spawn()
        .expect("failed to spawn `fzf` command");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(&feed.into_bytes())
        .expect("failed to write to stdin of `fzf` command");

    let output = child
        .wait_with_output()
        .expect("failed to wait `fzf` to exit");
    let output = str::from_utf8(output.stdout.as_slice()).unwrap();

    let id = output.split("\t").take(1).collect::<String>();
    debug!("chosen id: {:?}", id);

    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

/// Create new tmux session for dead session.
fn create_session(name: &str, config: Config) {
    debug!("resurrect session: {}", name);

    let mut path = PathBuf::from(config.sessions_dir);
    path.push(&format!("{}.tmux-session.zsh", name));

    let path = path.to_str().unwrap();
    debug!("run {:?}", path);

    // create session
    let output = Command::new(path).output().unwrap();
    if output.status.success() {
        tmux::switch_to(name);
    } else {
        error!("failed to resurrect session [{}]: {:#?}", name, output);
    }
}
