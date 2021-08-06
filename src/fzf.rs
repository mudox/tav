use std::cell::RefCell;
use std::rc::Rc;

use log::debug;
use termion::color;
use termkit::ui::*;

use crate::config::Config;
use crate::tmux::snapshot::{Session, Snapshot, Window};

const SS_WIDTH: usize = 4; // session symbol width
const WS_WIDTH: usize = 2; // session symbol width
const LEFT_MARGIN: usize = 2; // for each fzf list line
const MIN_GAP: usize = 4;

const MIN_WIDTH: usize = 50;

pub struct Formatter {
    config: Config,
    snapshot: Snapshot,

    part1_width: usize,
    part2_width: usize,
    gap: String,

    pub width: usize,  // feed columns
    pub height: usize, // feed lines

    pub feed: Vec<String>,
}

impl Formatter {
    pub fn new(snapshot: Snapshot, config: Config) -> Formatter {
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

        let symbol = lspan(symbol, color::White, WS_WIDTH);

        // name
        let name = fg(color::Magenta, &session.name);

        format!(
            "{id}\t{symbol:symbol_width$}{name:name_width$}",
            id = session.id,
            symbol = symbol,
            symbol_width = SS_WIDTH,
            name = name,
            name_width = self.part1_width,
        )
    }

    fn window_line(&self, window: &Window) -> String {
        // margin
        let margin = xspan(SS_WIDTH);
        // symbol
        let symbol = lspan("-", color::Rgb(80, 80, 80), WS_WIDTH);
        // name
        let name = lspan(&window.name, color::Green, self.part1_width - WS_WIDTH);
        // path
        let session_name = window
            .session
            .upgrade()
            .map(|s| s.borrow().name.clone())
            .unwrap_or("[W]".to_string());
        let mut path = format!("{}:{}", session_name, window.index);
        path = rspan(&path, color::Blue, self.part2_width);

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
        let gray = color::Rgb(80, 80, 80);

        // symbol
        let nbsp = "\u{a0}".to_string(); // default symbol
        let symbol = self.config.session_icons.get(name).unwrap_or(&nbsp);
        let symbol = lspan(symbol, color::White, SS_WIDTH);

        // left
        let left = lspan(name, gray, self.part1_width);

        // right
        let right = " ";
        let right = rspan(&right, gray, self.part2_width);

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
