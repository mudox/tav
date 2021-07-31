use crate::config::Config;
use crate::tmux::snapshot::{Session, Snapshot, Window};

use log::debug;
use termion::{color, style};

const SS_WIDTH: usize = 2; // session symbol width
const WS_WIDTH: usize = 2; // session symbol width
const LEFT_MARGIN: usize = 2; // for each fzf list line
const MIN_GAP: usize = 4;

const MIN_WIDTH: usize = 50;

fn apply_color<S: std::fmt::Display>(style: S, text: &str) -> String {
    format!("{}{}{}", style, text, style::Reset)
}

fn fg<C: color::Color>(color: C, text: &str) -> String {
    apply_color(color::Fg(color), text)
}

fn span(width: usize) -> String {
    let r = format!("{:·^1$}", "·", width);
    fg(color::Black, &r)
}

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
        self.gap = span(gap_width);
        self.width = width;
    }

    fn compose_feed(&mut self) {
        for (_, session) in &self.snapshot.sessions {
            let session = session.borrow();
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
        let symbol = "";
        // let symbol = "🐶";
        let symbol = format!("{0:1$}", symbol, WS_WIDTH);
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
        let margin = span(SS_WIDTH);
        // symbol
        let mut symbol = format!("{0:1$}", "-", WS_WIDTH);
        symbol = fg(color::Yellow, &symbol);
        // name
        let mut name = format!("{0:1$}", window.name, self.part1_width - WS_WIDTH);
        name = fg(color::Green, &name);
        // path
        let session_name = window
            .session
            .upgrade()
            .map(|s| s.borrow().name.clone())
            .unwrap_or("[W]".to_string());
        let mut path = format!("{}:{}", session_name, window.index);
        path = format!("{:>1$}", path, self.part2_width);
        path = fg(color::Blue, &path);

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
        let mut symbol = span(WS_WIDTH);
        symbol = fg(gray, &symbol);
        // let symbol = "";
        // let symbol = format!("{0:1$}", symbol, WS_WIDTH);

        // left
        let mut left = format!("{0:1$}", name, self.part1_width);
        left = fg(gray, &left);

        // right
        let right = " ";
        let right = format!("{:>1$}", right, self.part2_width);
        let right = fg(gray, &right);

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
