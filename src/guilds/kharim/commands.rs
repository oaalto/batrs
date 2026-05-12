use crate::abilities;
use crate::ansi::StyledLine;
use crate::command;
use crate::command::Command;
use crate::guilds::KharimGuild;
use crate::guilds::use_skill;
use std::collections::HashMap;

impl KharimGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("ufp".to_string(), Self::use_foul_play as Command),
            ("uam".to_string(), Self::use_act_of_mercy as Command),
            ("ufr".to_string(), Self::use_feigned_remorse as Command),
            ("usd".to_string(), Self::use_scourge as Command),
            ("uvb".to_string(), Self::use_vampiric_blow as Command),
            ("ucc".to_string(), Self::use_chaotic_circulation as Command),
            ("cfa".to_string(), Self::cast_flame_arrow as Command),
            ("cbf".to_string(), Self::cast_blade_of_fire as Command),
            ("cac".to_string(), Self::cast_aura_of_chaos as Command),
            ("kharim_rip".to_string(), Self::rip_action as Command),
            ("tositwar".to_string(), Self::nav_tositwar as Command),
            ("fromsitwar".to_string(), Self::nav_fromsitwar as Command),
            ("tomelee".to_string(), Self::nav_tomelee as Command),
            ("frommelee".to_string(), Self::nav_frommelee as Command),
            ("tosw".to_string(), Self::nav_tosw as Command),
            ("fromsw".to_string(), Self::nav_fromsw as Command),
            ("tose".to_string(), Self::nav_tose as Command),
            ("fromse".to_string(), Self::nav_fromse as Command),
            ("tonw".to_string(), Self::nav_tonw as Command),
            ("fromnw".to_string(), Self::nav_fromnw as Command),
            ("tone".to_string(), Self::nav_tone as Command),
            ("fromne".to_string(), Self::nav_fromne as Command),
            ("tokitan".to_string(), Self::nav_tokitan as Command),
            ("fromkitan".to_string(), Self::nav_fromkitan as Command),
            ("tosouls".to_string(), Self::nav_tosouls as Command),
            ("fromsouls".to_string(), Self::nav_fromsouls as Command),
            ("tocloud".to_string(), Self::nav_tocloud as Command),
            ("fromcloud".to_string(), Self::nav_fromcloud as Command),
            ("toswords".to_string(), Self::nav_toswords as Command),
            ("fromswords".to_string(), Self::nav_fromswords as Command),
            ("todevice".to_string(), Self::nav_todevice as Command),
            ("fromdevice".to_string(), Self::nav_fromdevice as Command),
            ("kharim_help".to_string(), Self::help as Command),
        ])
    }

    pub fn use_foul_play(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let tail = data.args.trim();
        let logical = if tail.is_empty() {
            "kharim observe;use 'foul play'".to_string()
        } else {
            format!("kharim observe;target {tail};use 'foul play' {tail}")
        };
        Some(abilities::client_send_line(&logical))
    }

    pub fn use_act_of_mercy(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(use_skill("deceitful act of mercy", data))
    }

    pub fn use_feigned_remorse(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(use_skill("feigned remorse", data))
    }

    pub fn use_scourge(data: &command::Data, _ctx: &mut command::CommandContext) -> Option<String> {
        let tail = data.args.trim();
        let logical = if tail.is_empty() {
            "use 'scourge of dark steel'".to_string()
        } else {
            format!("kharim observe;target {tail};use 'scourge of dark steel' {tail}")
        };
        Some(abilities::client_send_line(&logical))
    }

    pub fn use_vampiric_blow(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(use_skill("vampiric blow", data))
    }

    pub fn use_chaotic_circulation(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("use Chaotic circulation at me"))
    }

    pub fn cast_flame_arrow(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let logical = if data.args.trim().is_empty() {
            "cast flame arrow at device".to_string()
        } else {
            format!("cast flame arrow at {}", data.args.trim())
        };
        Some(abilities::client_send_line(&logical))
    }

    pub fn cast_blade_of_fire(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("cast blade of fire"))
    }

    pub fn cast_aura_of_chaos(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("cast aura of chaos"))
    }

    pub fn rip_action(_data: &command::Data, _ctx: &mut command::CommandContext) -> Option<String> {
        Some(abilities::client_send_line(
            "rip_action set get all from corpse;kharim drain corpse;drop zinc;drop mowgles",
        ))
    }

    pub fn nav_tositwar(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("6 w;4 sw;4 w;nw"))
    }

    pub fn nav_fromsitwar(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("se;4 e;4 ne;6 e"))
    }

    pub fn nav_tomelee(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("6 w;4 sw;20 w;6 w;nw"))
    }

    pub fn nav_frommelee(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("se;6 e;20 e;4 ne;6 e"))
    }

    pub fn nav_tosw(_data: &command::Data, _ctx: &mut command::CommandContext) -> Option<String> {
        Some(abilities::client_send_line("6 w;4 sw;20 w;ne;2 n;nw"))
    }

    pub fn nav_fromsw(_data: &command::Data, _ctx: &mut command::CommandContext) -> Option<String> {
        Some(abilities::client_send_line("se;2 s;sw;20 e;4 ne;6 e"))
    }

    pub fn nav_tose(_data: &command::Data, _ctx: &mut command::CommandContext) -> Option<String> {
        Some(abilities::client_send_line("6 w;4 sw;9 w;2 nw;ne;n"))
    }

    pub fn nav_fromse(_data: &command::Data, _ctx: &mut command::CommandContext) -> Option<String> {
        Some(abilities::client_send_line("s;sw;2 se;9 e;4 ne;6 e"))
    }

    pub fn nav_tonw(_data: &command::Data, _ctx: &mut command::CommandContext) -> Option<String> {
        Some(abilities::client_send_line(
            "6 w;4 sw;20 w;8 w;3 nw;3 ne;2 n;2 nw;3 n;3 nw;3 w;nw;n;3 ne;6 e",
        ))
    }

    pub fn nav_fromnw(_data: &command::Data, _ctx: &mut command::CommandContext) -> Option<String> {
        Some(abilities::client_send_line(
            "6 w;3 sw;s;se;3 e;3 se;3 s;2 se;2 s;3 sw;3 se;8 e;20 e;4 ne;6 e",
        ))
    }

    pub fn nav_tone(_data: &command::Data, _ctx: &mut command::CommandContext) -> Option<String> {
        Some(abilities::client_send_line(
            "6 w;3 nw;7 w;2 n;nw;2 ne;4 n;nw;3 n;e;ne;6 e",
        ))
    }

    pub fn nav_fromne(_data: &command::Data, _ctx: &mut command::CommandContext) -> Option<String> {
        Some(abilities::client_send_line(
            "6 w;sw;w;3 s;se;4 s;2 sw;se;2 s;7 e;3 se;6 e",
        ))
    }

    pub fn nav_tokitan(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(
            "6 w;4 sw;20 w;8 w;3 nw;3 ne;2 n;2 nw;3 n;3 nw;3 w;nw;3 n;nw;enter",
        ))
    }

    pub fn nav_fromkitan(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(
            "out;se;3 s;se;3 e;3 se;3 s;2 se;2 s;3 sw;3 se;8 e;20 e;4 ne;6 e",
        ))
    }

    pub fn nav_tosouls(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(
            "6 w;3 nw;7 w;2 n;nw;2 ne;4 n;nw;3 n;2 w;nw;enter;ask man about services;kharim souls",
        ))
    }

    pub fn nav_fromsouls(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(
            "out;se;2 e;3 s;se;4 s;2 sw;se;2 s;7 e;3 se;6 e",
        ))
    }

    pub fn nav_tocloud(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(
            "6 w;4 sw;20 w;11 w;4 nw;3 w;cloud",
        ))
    }

    pub fn nav_fromcloud(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(
            "descend;3 e;4 se;11 e;20 e;4 ne;6 e",
        ))
    }

    pub fn nav_toswords(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(
            "4 w;6 nw;5 n;2 e;2 ne;e;2 ne;n;ne;2 e;enter",
        ))
    }

    pub fn nav_fromswords(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(
            "out;2 w;sw;s;2 sw;w;2 sw;2 w;5 s;6 se;4 e",
        ))
    }

    pub fn nav_todevice(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(
            "w;out;3 w;nw;ne;2 nw;4 w;S;E;se;e",
        ))
    }

    pub fn nav_fromdevice(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(
            "w;nw;7 w;7 n;E;s;se;sw;se;E;e;enter",
        ))
    }

    pub fn help(_data: &command::Data, ctx: &mut command::CommandContext) -> Option<String> {
        let lines = [
            "/tositwar goes from the device to the shield trainer",
            "/tomelee goes from the device to the general fighting trainer",
            "/tosw goes from the device to the spell trainer",
            "/tose goes from the device to the <insert name here>",
            "/tonw goes from the device to the scout trainer",
            "/tone goes from the device to the attack skills trainer",
            "/tokitan goes from the device to Kitan",
            "/tosouls goes from the device to the souls room",
            "/tocloud goes from the device to the cloud",
            "/toswords goes from the device to the sword hotel",
            "/todevice goes from the elevator to the device",
        ];
        for line in lines {
            ctx.push_output_line(StyledLine::new(line));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandContext;

    fn data(cmd: &str, args: &str) -> command::Data {
        command::Data {
            cmd: cmd.to_string(),
            args: args.to_string(),
        }
    }

    fn empty_ctx() -> CommandContext {
        CommandContext::new(HashMap::new(), true, String::new())
    }

    #[test]
    fn foul_play_without_target() {
        let out = KharimGuild::use_foul_play(&data("ufp", ""), &mut empty_ctx());
        assert_eq!(out, Some("@kharim observe;use 'foul play'".to_string()));
    }

    #[test]
    fn foul_play_with_target() {
        let out = KharimGuild::use_foul_play(&data("ufp", "orc"), &mut empty_ctx());
        assert_eq!(
            out,
            Some("@kharim observe;target orc;use 'foul play' orc".to_string())
        );
    }

    #[test]
    fn scourge_without_target() {
        let out = KharimGuild::use_scourge(&data("usd", ""), &mut empty_ctx());
        assert_eq!(out, Some("@use 'scourge of dark steel'".to_string()));
    }

    #[test]
    fn scourge_with_target() {
        let out = KharimGuild::use_scourge(&data("usd", "troll"), &mut empty_ctx());
        assert_eq!(
            out,
            Some("@kharim observe;target troll;use 'scourge of dark steel' troll".to_string())
        );
    }

    #[test]
    fn act_of_mercy_uses_targeted_use() {
        let out = KharimGuild::use_act_of_mercy(&data("uam", "orc"), &mut empty_ctx());
        assert_eq!(
            out,
            Some("@target orc;use 'deceitful act of mercy' orc".to_string())
        );
    }

    #[test]
    fn chaotic_circulation_exact_casing() {
        let out = KharimGuild::use_chaotic_circulation(&data("ucc", ""), &mut empty_ctx());
        assert_eq!(out, Some("@use Chaotic circulation at me".to_string()));
    }

    #[test]
    fn flame_arrow_defaults_to_device() {
        let out = KharimGuild::cast_flame_arrow(&data("cfa", ""), &mut empty_ctx());
        assert_eq!(out, Some("@cast flame arrow at device".to_string()));
    }

    #[test]
    fn flame_arrow_with_target() {
        let out = KharimGuild::cast_flame_arrow(&data("cfa", "goblin"), &mut empty_ctx());
        assert_eq!(out, Some("@cast flame arrow at goblin".to_string()));
    }

    #[test]
    fn rip_action_matches_tf() {
        let out = KharimGuild::rip_action(&data("kharim_rip", ""), &mut empty_ctx());
        assert_eq!(
            out,
            Some(
                "@rip_action set get all from corpse;kharim drain corpse;drop zinc;drop mowgles"
                    .to_string()
            )
        );
    }

    #[test]
    fn kharim_help_emits_lines_and_no_send() {
        let mut ctx = empty_ctx();
        let out = KharimGuild::help(&data("kharim_help", ""), &mut ctx);
        assert!(out.is_none());
        assert_eq!(ctx.output_lines.len(), 11);
        assert!(ctx.output_lines[0].plain_line.contains("tositwar"));
    }
}
