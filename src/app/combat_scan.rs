use crate::ansi::palette;
use lazy_static::lazy_static;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use regex::Regex;
use unicode_width::UnicodeWidthStr;

pub(crate) const PROBE_COMMAND: &str = "#scan all";
const PROBE_ECHO: &str = "scan all";
pub(crate) const NOT_IN_COMBAT_LINE: &str = "You are not in combat right now.";
const MAX_LINES_WAITING_FOR_ECHO: u8 = 30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IncomingCombatScanLine {
    Visible,
    InternalProbe,
    CombatEnded { internal_probe: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbePhase {
    Idle,
    WaitingForEcho,
    CapturingRows,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CombatScanRow {
    name: String,
    condition: CombatCondition,
    percent: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CombatCondition {
    Excellent,
    Good,
    SlightlyHurt,
    NoticeablyHurt,
    NotGood,
    Bad,
    VeryBad,
    NearDeath,
}

impl CombatCondition {
    fn parse(text: &str) -> Option<Self> {
        match text {
            "in excellent shape" => Some(Self::Excellent),
            "in a good shape" => Some(Self::Good),
            "slightly hurt" => Some(Self::SlightlyHurt),
            "noticeably hurt" => Some(Self::NoticeablyHurt),
            "not in a good shape" => Some(Self::NotGood),
            "in bad shape" => Some(Self::Bad),
            "in very bad shape" => Some(Self::VeryBad),
            "near death" => Some(Self::NearDeath),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Excellent => "excellent",
            Self::Good => "good",
            Self::SlightlyHurt => "slightly hurt",
            Self::NoticeablyHurt => "noticeably hurt",
            Self::NotGood => "not in good shape",
            Self::Bad => "bad shape",
            Self::VeryBad => "very bad shape",
            Self::NearDeath => "near death",
        }
    }

    fn color(self) -> ratatui::style::Color {
        match self {
            Self::Excellent | Self::Good => palette::GREEN,
            Self::SlightlyHurt | Self::NoticeablyHurt => palette::CYAN,
            Self::NotGood | Self::Bad => palette::YELLOW,
            Self::VeryBad | Self::NearDeath => palette::RED,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CombatScanState {
    active: bool,
    phase: ProbePhase,
    user_command_counter: u8,
    lines_waiting_for_echo: u8,
    pending_rows: Vec<CombatScanRow>,
    snapshot: Vec<CombatScanRow>,
}

impl Default for CombatScanState {
    fn default() -> Self {
        Self {
            active: false,
            phase: ProbePhase::Idle,
            user_command_counter: 0,
            lines_waiting_for_echo: 0,
            pending_rows: Vec::new(),
            snapshot: Vec::new(),
        }
    }
}

impl CombatScanState {
    pub(crate) fn start_combat_round(&mut self) -> Option<&'static str> {
        self.active = true;
        self.request_probe()
    }

    pub(crate) fn end_combat(&mut self) {
        self.active = false;
        self.phase = ProbePhase::Idle;
        self.user_command_counter = 0;
        self.lines_waiting_for_echo = 0;
        self.pending_rows.clear();
        self.snapshot.clear();
    }

    pub(crate) fn observe_user_game_command(&mut self) -> Option<&'static str> {
        if !self.active || self.phase != ProbePhase::Idle {
            return None;
        }

        self.user_command_counter += 1;
        if self.user_command_counter >= 2 {
            self.user_command_counter = 0;
            self.request_probe()
        } else {
            None
        }
    }

    pub(crate) fn handle_incoming_line(&mut self, line: &str) -> IncomingCombatScanLine {
        if line == NOT_IN_COMBAT_LINE {
            let internal_probe = self.phase != ProbePhase::Idle;
            self.end_combat();
            return IncomingCombatScanLine::CombatEnded { internal_probe };
        }

        match self.phase {
            ProbePhase::Idle => IncomingCombatScanLine::Visible,
            ProbePhase::WaitingForEcho => {
                if line == PROBE_ECHO {
                    self.phase = ProbePhase::CapturingRows;
                    self.lines_waiting_for_echo = 0;
                    self.pending_rows.clear();
                    IncomingCombatScanLine::InternalProbe
                } else {
                    self.lines_waiting_for_echo = self.lines_waiting_for_echo.saturating_add(1);
                    if self.lines_waiting_for_echo >= MAX_LINES_WAITING_FOR_ECHO {
                        self.phase = ProbePhase::Idle;
                        self.lines_waiting_for_echo = 0;
                    }
                    IncomingCombatScanLine::Visible
                }
            }
            ProbePhase::CapturingRows => {
                if let Some(row) = parse_scan_row(line) {
                    self.pending_rows.push(row);
                    IncomingCombatScanLine::InternalProbe
                } else {
                    self.complete_scan();
                    IncomingCombatScanLine::Visible
                }
            }
        }
    }

    pub(crate) fn render_lines(&self, width: u16) -> Vec<Line<'static>> {
        if !self.active || self.snapshot.is_empty() {
            return Vec::new();
        }

        let pieces: Vec<Vec<Span<'static>>> =
            self.snapshot.iter().map(|row| row_spans(row)).collect();
        wrap_pieces(pieces, width)
    }

    #[cfg(test)]
    pub(crate) fn snapshot(&self) -> &[CombatScanRow] {
        &self.snapshot
    }

    #[cfg(test)]
    pub(crate) fn is_active(&self) -> bool {
        self.active
    }

    #[cfg(test)]
    pub(crate) fn is_idle(&self) -> bool {
        self.phase == ProbePhase::Idle
    }

    fn request_probe(&mut self) -> Option<&'static str> {
        if self.phase == ProbePhase::Idle {
            self.phase = ProbePhase::WaitingForEcho;
            self.lines_waiting_for_echo = 0;
            Some(PROBE_COMMAND)
        } else {
            None
        }
    }

    fn complete_scan(&mut self) {
        self.snapshot = std::mem::take(&mut self.pending_rows);
        self.phase = ProbePhase::Idle;
        self.lines_waiting_for_echo = 0;
    }
}

fn parse_scan_row(line: &str) -> Option<CombatScanRow> {
    let captures = SCAN_ROW_REGEX.captures(line)?;
    let name = captures.name("name")?.as_str().trim();
    let condition = CombatCondition::parse(captures.name("condition")?.as_str())?;
    let percent = captures.name("percent")?.as_str().parse::<i32>().ok()?;

    Some(CombatScanRow {
        name: name.to_string(),
        condition,
        percent,
    })
}

fn row_spans(row: &CombatScanRow) -> Vec<Span<'static>> {
    vec![
        Span::styled(row.name.clone(), bold_white_style()),
        Span::styled(" is ", normal_text_style()),
        Span::styled(
            row.condition.label().to_string(),
            Style::default().fg(row.condition.color()),
        ),
        Span::styled(" (", normal_text_style()),
        Span::styled(
            row.percent.to_string(),
            Style::default().fg(row.condition.color()),
        ),
        Span::styled("%).", normal_text_style()),
    ]
}

fn wrap_pieces(pieces: Vec<Vec<Span<'static>>>, width: u16) -> Vec<Line<'static>> {
    if width == 0 {
        return pieces.into_iter().map(Line::from).collect();
    }

    let effective_width = width as usize;
    let mut lines: Vec<Vec<Span<'static>>> = Vec::new();
    let mut current = Vec::new();
    let mut current_width = 0;

    for piece in pieces {
        let piece_width = spans_width(&piece);
        let gap = if current.is_empty() { 0 } else { 2 };
        if !current.is_empty() && current_width + gap + piece_width > effective_width {
            lines.push(std::mem::take(&mut current));
            current_width = 0;
        }

        if !current.is_empty() {
            current.push(Span::styled("  ", normal_text_style()));
            current_width += 2;
        }
        current_width += piece_width;
        current.extend(piece);
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines.into_iter().map(Line::from).collect()
}

fn spans_width(spans: &[Span<'_>]) -> usize {
    spans.iter().map(|span| span.content.width()).sum()
}

fn normal_text_style() -> Style {
    Style::default().fg(palette::TEXT)
}

fn bold_white_style() -> Style {
    Style::default()
        .fg(palette::BOLD_WHITE)
        .add_modifier(Modifier::BOLD)
}

lazy_static! {
    static ref SCAN_ROW_REGEX: Regex = Regex::new(
        r"^(?P<name>.+) is (?P<condition>in excellent shape|in a good shape|slightly hurt|noticeably hurt|not in a good shape|in bad shape|in very bad shape|near death) \((?P<percent>[0-9]+)%\)\.$"
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_text(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    #[test]
    fn start_combat_requests_initial_probe_once() {
        let mut state = CombatScanState::default();

        assert_eq!(state.start_combat_round(), Some(PROBE_COMMAND));
        assert_eq!(state.start_combat_round(), None);
    }

    #[test]
    fn later_combat_round_requests_probe_after_previous_probe_completed() {
        let mut state = CombatScanState::default();
        state.start_combat_round();
        state.handle_incoming_line("scan all");
        state.handle_incoming_line("Guard is slightly hurt (70%).");
        state.handle_incoming_line("round output");

        assert_eq!(state.start_combat_round(), Some(PROBE_COMMAND));
    }

    #[test]
    fn captures_rows_after_echo_and_replaces_snapshot_on_terminator() {
        let mut state = CombatScanState::default();
        state.start_combat_round();

        assert_eq!(
            state.handle_incoming_line("scan all"),
            IncomingCombatScanLine::InternalProbe
        );
        assert_eq!(
            state.handle_incoming_line("Guard is noticeably hurt (50%)."),
            IncomingCombatScanLine::InternalProbe
        );
        assert_eq!(
            state.handle_incoming_line("Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >"),
            IncomingCombatScanLine::Visible
        );

        assert_eq!(state.snapshot().len(), 1);
        assert_eq!(state.snapshot()[0].name, "Guard");
        assert_eq!(
            state.snapshot()[0].condition,
            CombatCondition::NoticeablyHurt
        );
        assert_eq!(state.snapshot()[0].percent, 50);
        assert!(state.is_idle());
    }

    #[test]
    fn cadence_requests_every_second_user_command_when_idle() {
        let mut state = CombatScanState::default();
        state.start_combat_round();
        state.handle_incoming_line("scan all");
        state.handle_incoming_line("Guard is noticeably hurt (50%).");
        state.handle_incoming_line("done");

        assert_eq!(state.observe_user_game_command(), None);
        assert_eq!(state.observe_user_game_command(), Some(PROBE_COMMAND));
        assert_eq!(state.observe_user_game_command(), None);
    }

    #[test]
    fn combat_end_clears_state_and_marks_probe_response_internal() {
        let mut state = CombatScanState::default();
        state.start_combat_round();

        assert_eq!(
            state.handle_incoming_line(NOT_IN_COMBAT_LINE),
            IncomingCombatScanLine::CombatEnded {
                internal_probe: true
            }
        );
        assert!(!state.is_active());
        assert!(state.snapshot().is_empty());
        assert!(state.is_idle());
    }

    #[test]
    fn prompt_before_echo_does_not_cancel_in_flight_probe() {
        let mut state = CombatScanState::default();
        state.start_combat_round();

        assert_eq!(
            state.handle_incoming_line("Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >"),
            IncomingCombatScanLine::Visible
        );
        assert_eq!(
            state.handle_incoming_line("scan all"),
            IncomingCombatScanLine::InternalProbe
        );
        assert_eq!(
            state.handle_incoming_line("Guard is slightly hurt (70%)."),
            IncomingCombatScanLine::InternalProbe
        );
        assert_eq!(
            state.handle_incoming_line("done"),
            IncomingCombatScanLine::Visible
        );
        assert_eq!(state.snapshot().len(), 1);
    }

    #[test]
    fn missing_echo_recovers_after_bounded_line_count() {
        let mut state = CombatScanState::default();
        state.start_combat_round();

        for _ in 0..MAX_LINES_WAITING_FOR_ECHO {
            assert_eq!(
                state.handle_incoming_line("ordinary output"),
                IncomingCombatScanLine::Visible
            );
        }

        assert!(state.is_idle());
    }

    #[test]
    fn render_lines_styles_structured_scan_row() {
        let mut state = CombatScanState::default();
        state.start_combat_round();
        state.handle_incoming_line("scan all");
        state.handle_incoming_line("Guard is noticeably hurt (50%).");
        state.handle_incoming_line("done");

        let lines = state.render_lines(120);
        assert_eq!(line_text(&lines[0]), "Guard is noticeably hurt (50%).");
        let condition = lines[0]
            .spans
            .iter()
            .find(|span| span.content.as_ref() == "noticeably hurt")
            .expect("condition span");
        assert_eq!(condition.style.fg, Some(palette::CYAN));
        let percent = lines[0]
            .spans
            .iter()
            .find(|span| span.content.as_ref() == "50")
            .expect("percent span");
        assert_eq!(percent.style.fg, Some(palette::CYAN));
    }
}
