use std::cell::RefCell;
use std::collections::HashMap;
use std::process::Command;
use std::rc::{Rc, Weak};
use std::str;

use crate::term;

type ID = String;
type Index = u32;

#[derive(Debug)]
pub struct Counts {
    pub session: usize,
    pub window: usize,
    pub pane: usize,
}

impl Counts {
    pub fn new() -> Counts {
        Counts {
            session: 0,
            window: 0,
            pane: 0,
        }
    }
}

#[derive(Debug)]
pub struct Geometry {
    pub session_name_max_width: usize,
    pub window_name_max_width: usize,
    pub pane_title_max_width: usize,

    pub window_width: usize,
    pub window_height: usize,
}

impl Geometry {
    fn new() -> Geometry {
        Geometry {
            session_name_max_width: 0,
            window_name_max_width: 0,
            pane_title_max_width: 0,

            window_width: 0,
            window_height: 0,
        }
    }
}

#[derive(Debug)]
pub struct Snapshot {
    // serverPath: String,
    pub sessions: HashMap<ID, Rc<RefCell<Session>>>,
    pub counts: Counts,
    pub geometry: Geometry,
}

#[derive(Debug)]
pub struct Session {
    pub id: ID,
    pub name: String,

    pub windows: HashMap<ID, Rc<RefCell<Window>>>,
}

#[derive(Debug)]
pub struct Window {
    pub id: ID,
    pub index: Index,
    pub name: String,

    pub session: Weak<RefCell<Session>>,
    pub panes: HashMap<ID, Rc<RefCell<Pane>>>,
}

#[derive(Debug)]
pub struct Pane {
    pub id: ID,
    pub index: Index,
    pub title: String,

    pub window: Weak<RefCell<Window>>,
}

/// Run command `tmux list-panes` and collect output lines
fn list_lines() -> Vec<String> {
    let spec = [
        // session
        "#{session_id}",
        "#{session_name}",
        // window
        "#{window_id}",
        "#{window_index}",
        "#{window_name}",
        // pane
        "#{pane_id}",
        "#{pane_index}",
        "#{pane_title}",
    ]
    .iter()
    .map(|x| x.to_string())
    .collect::<Vec<String>>()
    .join("\t");

    let output = Command::new("tmux")
        .arg("list-panes")
        .arg("-a")
        .arg("-F")
        .arg(spec)
        .output()
        .expect("failed to run `tmux list-panes`");

    let output = str::from_utf8(&output.stdout).unwrap();
    let lines: Vec<&str> = output.split('\n').filter(|x| !x.is_empty()).collect();
    lines.into_iter().map(|x| x.to_string()).collect()
}

/// Take a snapshot for current tmux client.
pub fn create() -> Snapshot {
    let lines = list_lines();

    let mut snw = 0usize; // session name max width
    let mut wnw = 0usize; // window name max width
    let mut ptw = 0usize; // pane title max width

    let mut wc = 0usize; // window count
    let pc = lines.len(); // pane count

    let mut tmux = Snapshot {
        sessions: HashMap::new(),
        counts: Counts::new(),
        geometry: Geometry::new(),
    };

    let (w, h) = term::size();
    tmux.geometry.window_width = w;
    tmux.geometry.window_height = h;

    for line in lines {
        let mut tokens = line.split('\t');

        //
        // session
        //

        let id = tokens.next().unwrap().to_string();
        let name = tokens.next().unwrap().to_string();
        snw = snw.max(name.len());

        let session = Session {
            id: id.clone(),
            name,
            windows: HashMap::new(),
        };
        let session = Rc::new(RefCell::new(session));
        let session = tmux.sessions.entry(id).or_insert(session);
        let mut session_mut_ref = session.borrow_mut();

        //
        // window
        //

        let id = tokens.next().unwrap().to_string();
        let index: Index = tokens.next().unwrap().parse().unwrap();
        let name = tokens.next().unwrap().to_string();
        wnw = wnw.max(name.len());

        let window = Window {
            id: id.clone(),
            index,
            name,

            session: Weak::new(),
            panes: HashMap::new(),
        };
        let window = Rc::new(RefCell::new(window));
        if !session_mut_ref.windows.contains_key(&id) {
            wc += 1;
        }
        let window = session_mut_ref.windows.entry(id).or_insert(window);
        let mut window_mut_ref = window.borrow_mut();
        window_mut_ref.session = Rc::downgrade(&session);

        //
        // pane
        //

        let id = tokens.next().unwrap().to_string();
        let index: Index = tokens.next().unwrap().parse().unwrap();
        let title = tokens.next().unwrap().to_string();
        ptw = ptw.max(title.len());

        let pane = Pane {
            id: id.clone(),
            index,
            title,
            window: Weak::new(),
        };

        let pane = Rc::new(RefCell::new(pane));
        let pane = window_mut_ref.panes.entry(id).or_insert(pane);
        pane.borrow_mut().window = Rc::downgrade(&window);
    }

    // geometry
    tmux.geometry.session_name_max_width = snw;
    tmux.geometry.window_name_max_width = wnw;
    tmux.geometry.pane_title_max_width = ptw;

    // counts
    tmux.counts.session = tmux.sessions.len();
    tmux.counts.window = wc;
    tmux.counts.pane = pc;

    tmux
}
