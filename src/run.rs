use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str;

use crate::config::{Config, Task};
use crate::fzf::Formatter;
use crate::logging::*;
use crate::tmux::{cmd as tmux, snapshot};

pub fn run(config: Config) {
    console::set_colors_enabled(true);

    if !tmux::is_in_tmux() {
        std::eprintln!("Not run in tmux environment");
        return;
    }

    match config.task {
        Task::Popup => match choose_window(config.clone()) {
            Some(id) => {
                if id.starts_with("$") || id.starts_with("@") {
                    tmux::switch_to(&id);
                } else if id.starts_with("[dead]") {
                    create_session(&id[6..], config.clone());
                }
            }
            None => debug!("quit with noop"),
        },
        Task::Fzf => {}
        Task::List => {
            let snapshot = snapshot::create();
            let formatter = Formatter::new(&snapshot, &config);
            println!("{}", &formatter.feed.join("\n"));
        }
    }
}

/// Run `fzf-cmd` to let user choose a window.
fn choose_window(config: Config) -> Option<String> {
    //
    // take snapshot
    //

    let snapshot = snapshot::create();
    let (_, client_height) = tmux::client_size();

    //
    // generate fzf feed
    //

    let formatter = Formatter::new(&snapshot, &config);
    let width = formatter.width + 4 * 2 + 5 - 2; // 💀 magic number
    debug!("feed height: {}", formatter.height);

    let mut height = formatter.height + 2 * 2 + 5 + 1;
    height = height.min(client_height - 16);

    let feed = formatter.feed.join("\n");

    //
    // choose window id
    //

    let mut cmd = Command::new("fzf-tmux");
    let mut cmd_mut_ref = cmd.env("FZF_DEFAULT_OPTS", ""); // reset env effect

    // poopup
    cmd_mut_ref = cmd_mut_ref
        .arg("-w")
        .arg(width.to_string())
        .arg("-h")
        .arg(height.to_string());

    // search
    cmd_mut_ref = cmd_mut_ref
        .arg("--with-nth=2..")
        .arg("--no-sort")
        // .arg("--exact")
        .arg("--tiebreak=end");

    // appearance
    cmd_mut_ref = cmd_mut_ref
        .arg("--color=bg:-1,bg+:-1") // transparent background
        .arg("--layout=reverse")
        .arg("--ansi")
        .arg("--margin=3,5,3,3") // 💀 magic number
        .arg("--inline-info")
        .arg("--header")
        .arg("") // sepratate line
        .arg("--prompt=▶ ")
        .arg("--pointer=▶");

    // key bindings
    cmd_mut_ref = cmd_mut_ref
        .arg("--bind")
        .arg("ctrl-j:page-down")
        .arg("--bind")
        .arg("ctrl-k:page-up")
        .arg("--bind")
        .arg("ctrl-f:page-down")
        .arg("--bind")
        .arg("ctrl-b:page-up");

    let mut child = cmd_mut_ref
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
