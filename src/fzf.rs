use std::cell::RefCell;
use std::rc::Rc;

use console::{pad_str, style, Alignment::*, Color};
use log::debug;

use crate::config::Config;
use crate::tmux::snapshot::{Session, Snapshot, Window};

const SS_WIDTH: usize = 4; // session symbol width
const WS_WIDTH: usize = 2; // session symbol width
const LEFT_MARGIN: usize = 2; // for each fzf list line
const MIN_GAP: usize = 4;

const MIN_WIDTH: usize = 50;

const GRAY: Color = Color::Color256(242);
// lazy_static! {
// static ref GRAY: Color = Color::Color256(246);
// }

pub struct Formatter<'a> {
    config: &'a Config,
    snapshot: &'a Snapshot,

    part1_width: usize,
    part2_width: usize,
    gap: String,

    pub width: usize,  // feed columns
    pub height: usize, // feed lines

    pub feed: Vec<String>,
}

impl<'a> Formatter<'a> {
    pub fn new(snapshot: &'a Snapshot, config: &'a Config) -> Formatter<'a> {
        let mut f = Formatter {
            config,
            snapshot,

            part1_width: 0,
            part2_width: 0,

            gap: String::new(),

            width: 0,
            height: 0,

            feed: Vec::with_capacity(100),
        };

        f.calculate_sizes();
        f.compose_feed();

        f
    }

    fn calculate_sizes(&mut self) {
        // calculate sizes

        let mut part1 = self.snapshot.geometry.session_name_max_width;
        part1 = part1.max(self.snapshot.geometry.window_name_max_width);

        let mut part2 = self.snapshot.geometry.session_name_max_width;
        part2 += 4; // for `:1` window index part
        part2 = part2.max(10);

        let width_without_gap = LEFT_MARGIN + SS_WIDTH + part1 + part2;
        let width_with_gap = width_without_gap + MIN_GAP;
        let width = width_with_gap.max(MIN_WIDTH);
        let gap_width = width - width_without_gap;

        self.part1_width = part1;
        self.part2_width = part2;
        self.gap = xspan(gap_width);
        self.width = width;
    }

    fn compose_feed(&mut self) {
        // live sessions

        let mut sessions = self
            .snapshot
            .sessions
            .values()
            .cloned()
            .collect::<Vec<Rc<RefCell<Session>>>>();

        sessions.sort_by_key(|s| s.borrow().id.clone());

        for (index, session) in sessions.into_iter().enumerate() {
            let session = session.borrow();
            if index > 0 {
                self.feed.push("[sep]\t".to_string());
            }
            self.feed.push(self.live_session_line(&session));

            let mut windows = session
                .windows
                .values()
                .cloned()
                .collect::<Vec<std::rc::Rc<std::cell::RefCell<Window>>>>();
            windows.sort_by_key(|x| x.borrow().index);

            for window in &windows {
                let window = window.borrow();
                self.feed.push(self.window_line(&window));
            }
        }

        // separator line

        if !self.config.dead_session.names.is_empty() {
            let sep = "[sep]\t".to_string();
            self.feed.push(sep);
        }

        // dead sessions

        for name in &self.config.dead_session.names {
            if self
                .snapshot
                .sessions
                .values()
                .find(|s| &s.borrow().name == name)
                .is_none()
            {
                self.feed.push(self.dead_session_line(name));
            }
        }

        self.height = self.feed.len();
    }

    fn live_session_line(&self, session: &Session) -> String {
        // symbol
        let nbsp = "\u{a0}".to_string(); // default symbol
        let symbol = self
            .config
            .session_icons
            .get(&session.name)
            .unwrap_or(&nbsp);

        let symbol = pad_str(symbol, SS_WIDTH, Left, None);

        // name
        let name = style(&session.name).magenta().to_string();
        let name = pad_str(&name, self.part1_width, Left, None);

        format!(
            "{id}\t{symbol}{name}",
            id = session.id,
            symbol = symbol,
            name = name,
        )
    }

    fn window_line(&self, window: &Window) -> String {
        // margin
        let margin = xspan(SS_WIDTH);
        // symbol
        let symbol = style("-").fg(GRAY).to_string();
        let symbol = pad_str(&symbol, WS_WIDTH, Left, None);
        // name
        let name = style(&window.name).green().to_string();
        let name = pad_str(&name, self.part1_width - WS_WIDTH, Left, None);
        // path
        let session_name = window
            .session
            .upgrade()
            .map(|s| s.borrow().name.clone())
            .unwrap_or("[W]".to_string());
        let mut path = format!("{}:{}", session_name, window.index);
        path = style(path).blue().to_string();
        let path = pad_str(&path, self.part2_width, Right, None);

        format!(
            "{id}\t{margin}{symbol}{name}{gap}{path}",
            id = window.id,
            margin = margin,
            symbol = symbol,
            name = name,
            gap = self.gap,
            path = path,
        )
    }

    fn dead_session_line(&self, name: &str) -> String {
        // symbol
        let nbsp = "\u{a0}".to_string(); // default symbol
        let symbol = self.config.session_icons.get(name).unwrap_or(&nbsp);
        let symbol = pad_str(symbol, SS_WIDTH, Left, None);

        // left
        let left = style(name).fg(GRAY).to_string();
        let left = pad_str(&left, self.part1_width, Left, None);

        // right
        let right = " ";
        let right = style(right).fg(GRAY).to_string();
        let right = pad_str(&right, self.part2_width, Right, None);

        let line = format!(
            "[dead]{name}\t{symbol}{left}{gap}{right}",
            name = name,
            symbol = symbol,
            left = left,
            gap = self.gap,
            right = right,
        );

        debug!("dead session line: {}", line);
        line
    }
}

/// Transparent fixed length span.
pub fn xspan(width: usize) -> String {
    let s = style(".").black().to_string();
    pad_str(&s, width, Left, None).to_string()
}
