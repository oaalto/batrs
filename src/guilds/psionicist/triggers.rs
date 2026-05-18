use crate::abilities;
use crate::ansi::TextStyle;
use crate::automation::Action;
use crate::guilds::PsionicistGuild;
use crate::triggers::{LineEffect, TriggerEffects, TriggerFacts, TriggerLine};
use lazy_static::lazy_static;
use regex::Regex;

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
    pub fn psionicist_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        let mut output = TriggerEffects::default();
        let line = line.plain_line;

        if line == "You seize the mind of the monster as it dies." {
            output = output.style_line(TextStyle::BLUE);
            output
                .actions
                .push(Action::Send(abilities::client_send_line("psi sense")));
            return output;
        }

        if let Some(effect) =
            capture_hilite_effect(&NEED_UNIMAGINABLE_AMOUNT, line, 1, TextStyle::RED)
        {
            output.original.edits.push(effect);
            return output;
        }
        if let Some(effect) =
            capture_hilite_effect(&NEED_EXTREMELY_MUCH_MORE, line, 1, TextStyle::BRIGHT_RED)
        {
            output.original.edits.push(effect);
            return output;
        }
        if let Some(effect) = capture_hilite_effect(&NEED_MUCH_MORE, line, 1, TextStyle::MAGENTA) {
            output.original.edits.push(effect);
            return output;
        }
        if let Some(effect) = capture_hilite_effect(&NEED_MORE, line, 1, TextStyle::BRIGHT_MAGENTA)
        {
            output.original.edits.push(effect);
            return output;
        }
        if let Some(effect) = capture_hilite_effect(&NEED_A_LITTLE_MORE, line, 1, TextStyle::YELLOW)
        {
            output.original.edits.push(effect);
            return output;
        }
        if let Some(effect) = capture_hilite_effect(
            &ONLY_NEED_VERY_LITTLE_MORE,
            line,
            1,
            TextStyle::BRIGHT_YELLOW,
        ) {
            output.original.edits.push(effect);
            return output;
        }

        if STUNNED_INTRUSION.is_match(line) {
            output = output.style_line(TextStyle::BRIGHT_GREEN);
            return output;
        }

        if line.contains("YOU GAIN AN INCONCEIVABLE AMOUNT OF KNOWLEDGE!") {
            output = output.style_line(TextStyle::GREEN);
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
            output = output.style_line(TextStyle::GREEN);
            return output;
        }

        if line == "You gained no new knowledge from such a pitiful monster." {
            output = output.style_line(TextStyle::RED);
            return output;
        }

        if STILL_NEED_KNOWLEDGE_BLUE.is_match(line) {
            output = output.style_line(TextStyle::BLUE);
        }

        output
    }
}

fn capture_hilite_effect(
    re: &Regex,
    line: &str,
    capture_index: usize,
    style: TextStyle,
) -> Option<LineEffect> {
    let caps = re.captures(line)?;
    caps.get(capture_index)
        .map(|m| LineEffect::StylePlainByteRange {
            range: m.range(),
            style,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::AnsiCode;
    use crate::ansi::StyledLine;
    use crate::automation::Action;
    use crate::triggers::{TriggerFacts, TriggerLine};
    use unicode_segmentation::UnicodeSegmentation;

    fn run(line: &str) -> (TriggerEffects, StyledLine) {
        let mut styled_line = StyledLine::new(line);
        let output =
            PsionicistGuild::psionicist_trigger(&TriggerLine::new(line), &TriggerFacts::default());
        output.apply_line_effects_to(&mut styled_line);
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
