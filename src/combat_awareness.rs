use regex::Regex;
use std::sync::LazyLock;

pub const PROBE_COMMAND: &str = "#scan all";
pub const NOT_IN_COMBAT_LINE: &str = "You are not in combat right now.";
pub const DEATH_COMBAT_END_LINE: &str = "You can see Death, clad in black, collect your corpse.";

pub fn is_combat_end_line(line: &str) -> bool {
    line == NOT_IN_COMBAT_LINE || line == DEATH_COMBAT_END_LINE
}
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
    status: Option<String>,
}

impl CombatScanRow {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn condition(&self) -> CombatCondition {
        self.condition
    }

    pub fn percent(&self) -> i32 {
        self.percent
    }

    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
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
        if is_combat_end_line(line) {
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
            } else if self.phase == ProbePhase::WaitingForEcho {
                self.abandon_in_flight_probe();
            }
            self.active = true;
            let mut effects = vec![
                CombatAwarenessEffect::RoundStarted,
                CombatAwarenessEffect::SendShortScore,
            ];
            if self.request_probe().is_some() {
                effects.push(CombatAwarenessEffect::SendProbe);
            }
            return LineHandlingResult {
                gag: false,
                effects,
            };
        }

        match self.phase {
            ProbePhase::Idle => self.handle_idle_line(line),
            ProbePhase::WaitingForEcho => self.handle_waiting_for_echo(line),
            ProbePhase::CapturingRows => self.handle_capturing_rows(line),
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

    fn abandon_in_flight_probe(&mut self) {
        self.phase = ProbePhase::Idle;
        self.lines_waiting_for_echo = 0;
        self.pending_rows.clear();
    }

    fn handle_idle_line(&mut self, line: &str) -> LineHandlingResult {
        if !self.active {
            return LineHandlingResult {
                gag: false,
                effects: Vec::new(),
            };
        }
        if is_probe_echo(line) {
            self.phase = ProbePhase::CapturingRows;
            self.pending_rows.clear();
            return LineHandlingResult {
                gag: true,
                effects: Vec::new(),
            };
        }
        if let Some(row) = parse_scan_row(line) {
            self.phase = ProbePhase::CapturingRows;
            self.pending_rows.push(row);
            return LineHandlingResult {
                gag: true,
                effects: Vec::new(),
            };
        }
        LineHandlingResult {
            gag: false,
            effects: Vec::new(),
        }
    }

    fn handle_waiting_for_echo(&mut self, line: &str) -> LineHandlingResult {
        if is_probe_echo(line) {
            self.phase = ProbePhase::CapturingRows;
            self.lines_waiting_for_echo = 0;
            self.pending_rows.clear();
            return LineHandlingResult {
                gag: true,
                effects: Vec::new(),
            };
        }
        if let Some(row) = parse_scan_row(line) {
            self.phase = ProbePhase::CapturingRows;
            self.lines_waiting_for_echo = 0;
            self.pending_rows.push(row);
            return LineHandlingResult {
                gag: true,
                effects: Vec::new(),
            };
        }
        self.lines_waiting_for_echo = self.lines_waiting_for_echo.saturating_add(1);
        if self.lines_waiting_for_echo >= MAX_LINES_WAITING_FOR_ECHO {
            self.abandon_in_flight_probe();
        }
        LineHandlingResult {
            gag: false,
            effects: Vec::new(),
        }
    }

    fn handle_capturing_rows(&mut self, line: &str) -> LineHandlingResult {
        if is_probe_echo(line) {
            return LineHandlingResult {
                gag: true,
                effects: Vec::new(),
            };
        }
        if let Some(row) = parse_scan_row(line) {
            self.pending_rows.push(row);
            return LineHandlingResult {
                gag: true,
                effects: Vec::new(),
            };
        }
        self.complete_scan();
        LineHandlingResult {
            gag: false,
            effects: Vec::new(),
        }
    }

    fn complete_scan(&mut self) {
        self.snapshot = std::mem::take(&mut self.pending_rows);
        self.phase = ProbePhase::Idle;
        self.lines_waiting_for_echo = 0;
    }
}

fn is_probe_echo(line: &str) -> bool {
    line == "scan all" || line == PROBE_COMMAND
}

fn parse_scan_row(line: &str) -> Option<CombatScanRow> {
    let captures = SCAN_ROW_REGEX.captures(line)?;
    let name = captures.name("name")?.as_str().trim();
    let condition = CombatCondition::parse(captures.name("condition")?.as_str())?;
    let percent = captures.name("percent")?.as_str().parse::<i32>().ok()?;

    let status = captures
        .name("status")
        .map(|m| m.as_str().trim().to_string());

    Some(CombatScanRow {
        name: name.to_string(),
        condition,
        percent,
        status,
    })
}

static ROUND_HEADER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\*]+ Round .* [\*]+$").unwrap());
static SCAN_ROW_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?P<name>.+) is (?P<condition>in excellent shape|in a good shape|slightly hurt|noticeably hurt|not in a good shape|in bad shape|in very bad shape|near death) \((?P<percent>[0-9]+)%\)(?: and (?P<status>.+))?\.$"
    )
    .unwrap()
});

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
    fn death_combat_end_clears_state_and_emits_combat_ended() {
        let mut state = CombatAwareness::default();
        state.handle_incoming_line("*** Round 1 ***");

        let result = state.handle_incoming_line(DEATH_COMBAT_END_LINE);
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
    fn organic_death_combat_end_is_visible() {
        let mut state = CombatAwareness::default();
        state.handle_incoming_line("*** Round 1 ***");
        state.handle_incoming_line("scan all");
        state.handle_incoming_line("Guard is noticeably hurt (50%).");
        state.handle_incoming_line("done");

        let result = state.handle_incoming_line(DEATH_COMBAT_END_LINE);
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
    fn scan_row_before_echo_is_gagged() {
        let mut state = CombatAwareness::default();
        state.handle_incoming_line("*** Round 1 ***");

        assert!(
            state
                .handle_incoming_line("Guard is slightly hurt (70%).")
                .gag
        );
        assert!(state.handle_incoming_line("scan all").gag);
        assert!(!state.handle_incoming_line("done").gag);
        assert_eq!(state.snapshot().len(), 1);
    }

    #[test]
    fn hash_scan_all_echo_is_gagged() {
        let mut state = CombatAwareness::default();
        state.handle_incoming_line("*** Round 1 ***");

        assert!(state.handle_incoming_line("#scan all").gag);
        assert!(
            state
                .handle_incoming_line("Guard is slightly hurt (70%).")
                .gag
        );
    }

    #[test]
    fn orphan_probe_response_during_idle_combat_is_gagged() {
        let mut state = CombatAwareness::default();
        state.handle_incoming_line("*** Round 1 ***");

        for _ in 0..MAX_LINES_WAITING_FOR_ECHO {
            state.handle_incoming_line("ordinary output");
        }
        assert!(state.is_idle());

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
    fn second_probe_echo_during_capturing_rows_is_gagged() {
        let mut state = CombatAwareness::default();
        state.handle_incoming_line("*** Round 1 ***");
        state.handle_incoming_line("scan all");
        state.handle_incoming_line("Guard is slightly hurt (70%).");

        assert!(state.handle_incoming_line("scan all").gag);
        assert!(!state.handle_incoming_line("done").gag);
        assert_eq!(state.snapshot().len(), 1);
    }

    #[test]
    fn scan_row_with_status_suffix_is_parsed_into_snapshot() {
        let mut state = CombatAwareness::default();
        state.handle_incoming_line("*** Round 1 ***");
        state.handle_incoming_line("scan all");
        assert!(
            state
                .handle_incoming_line("Guard is in a good shape (90%) and stunned.")
                .gag
        );
        state.handle_incoming_line("done");

        assert_eq!(state.snapshot().len(), 1);
        assert_eq!(state.snapshot()[0].name(), "Guard");
        assert_eq!(state.snapshot()[0].condition(), CombatCondition::Good);
        assert_eq!(state.snapshot()[0].percent(), 90);
        assert_eq!(state.snapshot()[0].status(), Some("stunned"));
    }

    #[test]
    fn round_header_during_waiting_for_echo_requests_fresh_probe_once() {
        let mut state = CombatAwareness::default();
        state.handle_incoming_line("*** Round 1 ***");
        assert!(!state.is_idle());

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
}
