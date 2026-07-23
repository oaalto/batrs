use crate::ansi::palette;
use lazy_static::lazy_static;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use regex::Regex;
use unicode_width::UnicodeWidthStr;

pub const PROBE_COMMAND: &str = "#scan all";
const PROBE_ECHO: &str = "scan all";
pub const NOT_IN_COMBAT_LINE: &str = "You are not in combat right now.";
const MAX_LINES_WAITING_FOR_ECHO: u8 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatAwarenessEffect {
    RoundStarted,
    CombatEnded,
    SendProbe,
    SendShortScore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineHandlingResult {
    pub gag: bool,
    pub effects: Vec<CombatAwarenessEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatScanRow {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbePhase {
    Idle,
    WaitingForEcho,
    CapturingRows,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatAwareness {
    active: bool,
    phase: ProbePhase,
    user_command_counter: u8,
    lines_waiting_for_echo: u8,
    pending_rows: Vec<CombatScanRow>,
    snapshot: Vec<CombatScanRow>,
}

impl Default for CombatAwareness {
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

pub fn is_round_header(line: &str) -> bool {
    ROUND_HEADER_REGEX.is_match(line)
}

impl CombatAwareness {
    pub fn end_combat(&mut self) {
        self.active = false;
        self.phase = ProbePhase::Idle;
        self.user_command_counter = 0;
        self.lines_waiting_for_echo = 0;
        self.pending_rows.clear();
        self.snapshot.clear();
    }

    pub fn observe_user_game_command(&mut self) -> Option<CombatAwarenessEffect> {
        if !self.active || self.phase != ProbePhase::Idle {
            return None;
        }

        self.user_command_counter += 1;
        if self.user_command_counter >= 2 {
            self.user_command_counter = 0;
            if self.request_probe().is_some() {
                return Some(CombatAwarenessEffect::SendProbe);
            }
        }
        None
    }

    pub fn handle_incoming_line(&mut self, line: &str) -> LineHandlingResult {
        if line == NOT_IN_COMBAT_LINE {
            let internal_probe = self.phase != ProbePhase::Idle;
            self.end_combat();
            return LineHandlingResult {
                gag: internal_probe,
                effects: vec![CombatAwarenessEffect::CombatEnded],
            };
        }

        if is_round_header(line) {
            if self.phase == ProbePhase::CapturingRows {
                self.complete_scan();
            }
            self.active = true;
            let _ = self.request_probe();
            return LineHandlingResult {
                gag: false,
                effects: vec![
                    CombatAwarenessEffect::RoundStarted,
                    CombatAwarenessEffect::SendShortScore,
                    CombatAwarenessEffect::SendProbe,
                ],
            };
        }

        match self.phase {
            ProbePhase::Idle => LineHandlingResult {
                gag: false,
                effects: Vec::new(),
            },
            ProbePhase::WaitingForEcho => {
                if line == PROBE_ECHO {
                    self.phase = ProbePhase::CapturingRows;
                    self.lines_waiting_for_echo = 0;
                    self.pending_rows.clear();
                    LineHandlingResult {
                        gag: true,
                        effects: Vec::new(),
                    }
                } else {
                    self.lines_waiting_for_echo = self.lines_waiting_for_echo.saturating_add(1);
                    if self.lines_waiting_for_echo >= MAX_LINES_WAITING_FOR_ECHO {
                        self.phase = ProbePhase::Idle;
                        self.lines_waiting_for_echo = 0;
                    }
                    LineHandlingResult {
                        gag: false,
                        effects: Vec::new(),
                    }
                }
            }
            ProbePhase::CapturingRows => {
                if let Some(row) = parse_scan_row(line) {
                    self.pending_rows.push(row);
                    LineHandlingResult {
                        gag: true,
                        effects: Vec::new(),
                    }
                } else {
                    self.complete_scan();
                    LineHandlingResult {
                        gag: false,
                        effects: Vec::new(),
                    }
                }
            }
        }
    }

    pub fn render_lines(&self, width: u16) -> Vec<Line<'static>> {
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

    fn request_probe(&mut self) -> Option<()> {
        if self.phase == ProbePhase::Idle {
            self.phase = ProbePhase::WaitingForEcho;
            self.lines_waiting_for_echo = 0;
            Some(())
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
        Span::styled(row.name.clone(), enemy_name_style()),
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

fn enemy_name_style() -> Style {
    Style::default()
        .fg(palette::BOLD_RED)
        .add_modifier(Modifier::BOLD)
}

lazy_static! {
    static ref ROUND_HEADER_REGEX: Regex =
        Regex::new(r"^[\*]+ Round .* [\*]+$").unwrap();
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
    fn round_header_emits_round_started_and_requests_probe() {
        let mut state = CombatAwareness::default();

        let result = state.handle_incoming_line("*** Round 1 ***");
        assert_eq!(
            result.effects,
            vec![
                CombatAwarenessEffect::RoundStarted,
                CombatAwarenessEffect::SendShortScore,
                CombatAwarenessEffect::SendProbe,
            ]
        );
        assert!(state.is_active());
        assert!(!state.is_idle());
    }

    #[test]
    fn later_combat_round_requests_probe_after_previous_probe_completed() {
        let mut state = CombatAwareness::default();
        state.handle_incoming_line("*** Round 1 ***");
        state.handle_incoming_line("scan all");
        state.handle_incoming_line("Guard is slightly hurt (70%).");
        state.handle_incoming_line("round output");

        let result = state.handle_incoming_line("*** Round 2 ***");
        assert_eq!(
            result.effects,
            vec![
                CombatAwarenessEffect::RoundStarted,
                CombatAwarenessEffect::SendShortScore,
                CombatAwarenessEffect::SendProbe,
            ]
        );
        assert!(!state.is_idle());
    }

    #[test]
    fn captures_rows_after_echo_and_replaces_snapshot_on_terminator() {
        let mut state = CombatAwareness::default();
        state.handle_incoming_line("*** Round 1 ***");

        assert!(state.handle_incoming_line("scan all").gag);
        assert!(
            state
                .handle_incoming_line("Guard is noticeably hurt (50%).")
                .gag
        );
        assert!(
            !state
                .handle_incoming_line("Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >")
                .gag
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
        let mut state = CombatAwareness::default();
        state.handle_incoming_line("*** Round 1 ***");
        state.handle_incoming_line("scan all");
        state.handle_incoming_line("Guard is noticeably hurt (50%).");
        state.handle_incoming_line("done");

        assert_eq!(state.observe_user_game_command(), None);
        assert_eq!(
            state.observe_user_game_command(),
            Some(CombatAwarenessEffect::SendProbe)
        );
        assert_eq!(state.observe_user_game_command(), None);
    }

    #[test]
    fn combat_end_clears_state_and_gags_internal_probe_response() {
        let mut state = CombatAwareness::default();
        state.handle_incoming_line("*** Round 1 ***");

        let result = state.handle_incoming_line(NOT_IN_COMBAT_LINE);
        assert_eq!(
            result,
            LineHandlingResult {
                gag: true,
                effects: vec![CombatAwarenessEffect::CombatEnded],
            }
        );
        assert!(!state.is_active());
        assert!(state.snapshot().is_empty());
        assert!(state.is_idle());
    }

    #[test]
    fn organic_combat_end_is_visible() {
        let mut state = CombatAwareness::default();
        state.handle_incoming_line("*** Round 1 ***");
        state.handle_incoming_line("scan all");
        state.handle_incoming_line("Guard is noticeably hurt (50%).");
        state.handle_incoming_line("done");

        let result = state.handle_incoming_line(NOT_IN_COMBAT_LINE);
        assert_eq!(
            result,
            LineHandlingResult {
                gag: false,
                effects: vec![CombatAwarenessEffect::CombatEnded],
            }
        );
    }

    #[test]
    fn prompt_before_echo_does_not_cancel_in_flight_probe() {
        let mut state = CombatAwareness::default();
        state.handle_incoming_line("*** Round 1 ***");

        assert!(
            !state
                .handle_incoming_line("Hp:1/2 Sp:3/4 Ep:5/6 Exp:7 >")
                .gag
        );
        assert!(state.handle_incoming_line("scan all").gag);
        assert!(
            state
                .handle_incoming_line("Guard is slightly hurt (70%).")
                .gag
        );
        assert!(!state.handle_incoming_line("done").gag);
        assert_eq!(state.snapshot().len(), 1);
    }

    #[test]
    fn missing_echo_recovers_after_bounded_line_count() {
        let mut state = CombatAwareness::default();
        state.handle_incoming_line("*** Round 1 ***");

        for _ in 0..MAX_LINES_WAITING_FOR_ECHO {
            assert!(!state.handle_incoming_line("ordinary output").gag);
        }

        assert!(state.is_idle());
    }

    #[test]
    fn render_lines_styles_structured_scan_row() {
        let mut state = CombatAwareness::default();
        state.handle_incoming_line("*** Round 1 ***");
        state.handle_incoming_line("scan all");
        state.handle_incoming_line("Guard is noticeably hurt (50%).");
        state.handle_incoming_line("done");

        let lines = state.render_lines(120);
        assert_eq!(line_text(&lines[0]), "Guard is noticeably hurt (50%).");
        let name = lines[0]
            .spans
            .iter()
            .find(|span| span.content.as_ref() == "Guard")
            .expect("name span");
        assert_eq!(name.style.fg, Some(palette::BOLD_RED));
        assert!(name.style.add_modifier.contains(Modifier::BOLD));
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
