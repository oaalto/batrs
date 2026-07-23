use lazy_static::lazy_static;
use regex::Regex;

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

/// One combatant row from a completed `#scan all` snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatScanRow {
    name: String,
    condition: CombatCondition,
    percent: i32,
}

impl CombatScanRow {
    /// Enemy or combatant name from the scan line.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Parsed health condition phrase.
    pub fn condition(&self) -> CombatCondition {
        self.condition
    }

    /// Remaining health percent from the scan line.
    pub fn percent(&self) -> i32 {
        self.percent
    }
}

/// Health condition parsed from a `#scan all` row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatCondition {
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

    /// Short label for HUD rendering (may differ slightly from the BatMUD phrase).
    pub fn label(self) -> &'static str {
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

    /// Latest completed scan rows; empty when no probe has finished this combat.
    pub fn snapshot(&self) -> &[CombatScanRow] {
        &self.snapshot
    }

    /// Whether combat is considered active (round header seen, combat not ended).
    pub fn is_active(&self) -> bool {
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
        assert_eq!(state.snapshot()[0].name(), "Guard");
        assert_eq!(
            state.snapshot()[0].condition(),
            CombatCondition::NoticeablyHurt
        );
        assert_eq!(state.snapshot()[0].percent(), 50);
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
}
