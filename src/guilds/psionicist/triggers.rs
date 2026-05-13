use crate::abilities;
use crate::ansi::{AnsiCode, StyledLine};
use crate::automation::Action;
use crate::guilds::PsionicistGuild;
use crate::triggers::{TriggerContext, TriggerOutput};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use unicode_segmentation::UnicodeSegmentation;

lazy_static! {
    static ref NEED_UNIMAGINABLE_AMOUNT: Regex = Regex::new(
        r"You still need (an unimaginable amount) of more knowledge of how the mind works before you can improve your knowledge of mental defence\."
    ).unwrap();
    static ref NEED_EXTREMELY_MUCH_MORE: Regex = Regex::new(
        r"You still need (extremely much more) knowledge of how the mind works before you can improve your knowledge of mental defence\."
    ).unwrap();
    static ref NEED_MUCH_MORE: Regex = Regex::new(
        r"You still need (much more) knowledge of how the mind works before you can improve your knowledge of mental defence\."
    ).unwrap();
    static ref NEED_A_LITTLE_MORE: Regex = Regex::new(
        r"You still need (a little more) knowledge of how the mind works before you can improve your knowledge of mental defence\."
    ).unwrap();
    static ref NEED_MORE: Regex = Regex::new(
        r"You still need (more) knowledge of how the mind works before you can improve your knowledge of mental defence\."
    ).unwrap();
    static ref ONLY_NEED_VERY_LITTLE_MORE: Regex = Regex::new(
        r"You only need (very little more) knowledge of how the mind works before you can improve your knowledge of mental defence\."
    ).unwrap();
    static ref STILL_NEED_KNOWLEDGE_BLUE: Regex = Regex::new(
        r"You still need (.+) knowledge of how the mind works before you can improve your knowledge of mental defence\."
    ).unwrap();
    static ref STUNNED_INTRUSION: Regex = Regex::new(
        r"(.+) is stunned from the intrusion into (.+) mind\."
    ).unwrap();
}

impl PsionicistGuild {
    pub fn psionicist_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let mut output = TriggerOutput::default();
        let plain_owned = styled_line.plain_line.clone();
        let line = plain_owned.as_str();

        if line == "You seize the mind of the monster as it dies." {
            styled_line.set_line_color(AnsiCode::Blue, false);
            output
                .actions
                .push(Action::Send(abilities::client_send_line("psi sense")));
            return output;
        }

        if apply_if_captures(
            styled_line,
            &NEED_UNIMAGINABLE_AMOUNT,
            line,
            1,
            AnsiCode::Red,
            false,
        ) {
            return output;
        }
        if apply_if_captures(
            styled_line,
            &NEED_EXTREMELY_MUCH_MORE,
            line,
            1,
            AnsiCode::Red,
            true,
        ) {
            return output;
        }
        if apply_if_captures(
            styled_line,
            &NEED_MUCH_MORE,
            line,
            1,
            AnsiCode::Magenta,
            false,
        ) {
            return output;
        }
        if apply_if_captures(styled_line, &NEED_MORE, line, 1, AnsiCode::Magenta, true) {
            return output;
        }
        if apply_if_captures(
            styled_line,
            &NEED_A_LITTLE_MORE,
            line,
            1,
            AnsiCode::Yellow,
            false,
        ) {
            return output;
        }
        if apply_if_captures(
            styled_line,
            &ONLY_NEED_VERY_LITTLE_MORE,
            line,
            1,
            AnsiCode::Yellow,
            true,
        ) {
            return output;
        }

        if STUNNED_INTRUSION.is_match(line) {
            styled_line.set_line_color(AnsiCode::Green, true);
            return output;
        }

        if line.contains("YOU GAIN AN INCONCEIVABLE AMOUNT OF KNOWLEDGE!") {
            styled_line.set_line_color(AnsiCode::Green, false);
            return output;
        }

        if matches!(
            line,
            "You gain some knowledge of how the mind works."
                | "You gain useful knowledge of how the mind works."
                | "You gain considerable knowledge of how the mind works."
                | "You gain detailed knowledge of how the mind works."
                | "WOW! Your mind almost has trouble processing this much new knowledge!"
        ) || line
            == "You sense that you have acquired enough knowledge of how the mind works in order to improve your knowledge of mental defence."
        {
            styled_line.set_line_color(AnsiCode::Green, false);
            return output;
        }

        if line == "You gained no new knowledge from such a pitiful monster." {
            styled_line.set_line_color(AnsiCode::Red, false);
            return output;
        }

        if STILL_NEED_KNOWLEDGE_BLUE.is_match(line) {
            styled_line.set_line_color(AnsiCode::Blue, false);
        }

        output
    }
}

fn apply_if_captures(
    styled_line: &mut StyledLine,
    re: &Regex,
    line: &str,
    capture_index: usize,
    color: AnsiCode,
    bold: bool,
) -> bool {
    let Some(caps) = re.captures(line) else {
        return false;
    };
    apply_capture_hilite(styled_line, &caps, capture_index, color, bold);
    true
}

fn apply_capture_hilite(
    styled_line: &mut StyledLine,
    captures: &Captures<'_>,
    index: usize,
    color: AnsiCode,
    bold: bool,
) {
    let Some(m) = captures.get(index) else {
        return;
    };

    let start = byte_to_grapheme_index(&styled_line.plain_line, m.start());
    let end = byte_to_grapheme_index(&styled_line.plain_line, m.end());
    let len = styled_line.styled_chars.len();
    let start = start.min(len);
    let end = end.min(len);

    for grapheme_ix in start..end {
        styled_line.styled_chars[grapheme_ix].color = color.into();
        styled_line.styled_chars[grapheme_ix].bold = bold;
    }
}

fn byte_to_grapheme_index(text: &str, byte_index: usize) -> usize {
    text.get(..byte_index)
        .map(|slice| slice.graphemes(true).count())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::{Action, Automation};
    use crate::stats::Stats;

    fn run(line: &str) -> (TriggerOutput, StyledLine) {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = TriggerContext {
            stats: &mut stats,
            automation: &mut automation,
            rig: None,
            player_name: None,
        };
        let mut styled_line = StyledLine::new(line);
        let output = PsionicistGuild::psionicist_trigger(&mut ctx, &mut styled_line);
        (output, styled_line)
    }

    #[test]
    fn mindseize_death_sends_psi_sense_and_paints_blue() {
        let line = "You seize the mind of the monster as it dies.";
        let (output, styled) = run(line);

        assert_eq!(output.actions.len(), 1);
        match &output.actions[0] {
            Action::Send(send) => assert_eq!(
                send.as_str(),
                abilities::client_send_line("psi sense").as_str()
            ),
            other => panic!("expected Send(psi sense), got {other:?}"),
        }

        assert!(
            styled
                .styled_chars
                .iter()
                .all(|c| c.color == AnsiCode::Blue)
        );
        assert!(styled.styled_chars.iter().all(|c| !c.bold));
    }

    #[test]
    fn need_much_more_highlights_capture_magenta() {
        let full = "You still need much more knowledge of how the mind works before you can improve your knowledge of mental defence.";
        let (_output, styled) = run(full);

        let fragment = styled
            .plain_line
            .find("much more")
            .expect("expected fragment");
        let frag_end = fragment + "much more".len();
        let grapheme_fragment = styled.plain_line[..fragment].graphemes(true).count();
        let grapheme_end = styled.plain_line[..frag_end].graphemes(true).count();

        for i in 0..grapheme_fragment {
            assert!(
                styled.styled_chars[i].color == AnsiCode::DefaultColor,
                "{i}",
            );
        }
        for i in grapheme_fragment..grapheme_end {
            assert_eq!(styled.styled_chars[i].color, AnsiCode::Magenta);
        }
    }
}
