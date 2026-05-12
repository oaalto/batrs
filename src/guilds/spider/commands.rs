use crate::abilities;
use crate::ansi::{AnsiCode, StyledLine};
use crate::command;
use crate::command::Command;
use crate::guilds::SpiderGuild;
use std::collections::HashMap;

impl SpiderGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("csw".to_string(), Self::cast_spider_wrath as Command),
            ("chs".to_string(), Self::cast_hunger_of_the_spider),
            ("csum".to_string(), Self::cast_spider_demon_conjuration),
            ("ctrl".to_string(), Self::cast_spider_demon_control),
            ("csac".to_string(), Self::cast_spider_demon_sacrifice),
            ("cban".to_string(), Self::cast_spider_demon_banishment),
            ("cinq".to_string(), Self::cast_spider_demon_inquiry),
            ("cchan".to_string(), Self::cast_spider_demon_channeling),
            ("ctd".to_string(), Self::cast_toxic_dilution),
            ("cvb".to_string(), Self::cast_venom_blade),
            ("cswalk".to_string(), Self::cast_spider_walk),
            ("chw".to_string(), Self::cast_heavy_weight),
            ("cmsac".to_string(), Self::cast_spider_demon_mass_sacrifice),
            ("cpsq".to_string(), Self::cast_prayer_to_the_spider_queen),
            ("crmp".to_string(), Self::cast_remove_poison),
            ("us".to_string(), Self::use_stab),
        ])
    }

    fn cast_spider_wrath(data: &command::Data, _: &mut command::CommandContext) -> Option<String> {
        let args = data.args.trim();
        if args.is_empty() {
            return Some(abilities::cast_quoted_with_suffix("spider wrath", ""));
        }
        Some(abilities::compound_send(&[
            &format!("target {args}"),
            &format!("cast 'spider wrath' {args}"),
        ]))
    }

    fn cast_hunger_of_the_spider(
        data: &command::Data,
        _: &mut command::CommandContext,
    ) -> Option<String> {
        let args = data.args.trim();
        if args.is_empty() {
            return None;
        }
        Some(abilities::compound_send(&[
            &format!("target {args}"),
            &format!("cast hunger of the spider at {args}"),
        ]))
    }

    fn cast_spider_demon_conjuration(
        data: &command::Data,
        _: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(&format!(
            "cast spider demon conjuration at me with {}",
            data.args
        )))
    }

    fn cast_spider_demon_control(
        _: &command::Data,
        _: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(
            "cast spider demon control at me",
        ))
    }

    fn cast_spider_demon_sacrifice(
        data: &command::Data,
        _: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(&format!(
            "cast spider demon sacrifice at {}",
            data.args.trim()
        )))
    }

    fn cast_spider_demon_banishment(
        _: &command::Data,
        _: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(
            "cast spider demon banishment at me",
        ))
    }

    fn cast_spider_demon_inquiry(
        _: &command::Data,
        _: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(
            "cast spider demon inquiry at me",
        ))
    }

    fn cast_spider_demon_channeling(
        _: &command::Data,
        _: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(
            "cast spider demon channeling at me",
        ))
    }

    fn cast_toxic_dilution(
        data: &command::Data,
        _: &mut command::CommandContext,
    ) -> Option<String> {
        let args = data.args.trim();
        if args.is_empty() {
            Some(abilities::client_send_line("cast toxic dilution at me"))
        } else {
            Some(abilities::cast_quoted_with_suffix("toxic dilution", args))
        }
    }

    fn cast_venom_blade(data: &command::Data, ctx: &mut command::CommandContext) -> Option<String> {
        let args = data.args.trim();
        if args.is_empty() {
            let mut banner = StyledLine::new("No target!");
            banner.set_line_color(AnsiCode::Red, true);
            ctx.push_output_line(banner);
            return None;
        }
        Some(abilities::cast_quoted_with_suffix("venom blade", args))
    }

    fn cast_spider_walk(data: &command::Data, _: &mut command::CommandContext) -> Option<String> {
        let args = data.args.trim();
        if args.is_empty() {
            Some(abilities::client_send_line("cast spider walk at me"))
        } else {
            Some(abilities::cast_quoted_with_suffix("spider walk", args))
        }
    }

    fn cast_heavy_weight(data: &command::Data, _: &mut command::CommandContext) -> Option<String> {
        let args = data.args.trim();
        if args.is_empty() {
            Some(abilities::client_send_line("cast heavy weight at me"))
        } else {
            Some(abilities::cast_quoted_with_suffix("heavy weight", args))
        }
    }

    fn cast_spider_demon_mass_sacrifice(
        _: &command::Data,
        _: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(
            "cast spider demon mass sacrifice",
        ))
    }

    fn cast_prayer_to_the_spider_queen(
        _: &command::Data,
        _: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(
            "cast prayer to the spider queen",
        ))
    }

    fn cast_remove_poison(data: &command::Data, _: &mut command::CommandContext) -> Option<String> {
        let args = data.args.trim();
        if args.is_empty() {
            Some(abilities::client_send_line("cast remove poison at me"))
        } else {
            Some(abilities::client_send_line(&format!(
                "cast remove poison at {args}"
            )))
        }
    }

    fn use_stab(data: &command::Data, _: &mut command::CommandContext) -> Option<String> {
        let args = data.args.trim();
        if args.is_empty() {
            return Some(abilities::client_send_line("use 'stab'"));
        }
        Some(abilities::compound_send(&[
            &format!("target {args}"),
            &format!("use 'stab' {args}"),
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandContext;

    fn ctx() -> CommandContext {
        CommandContext::new(std::collections::HashMap::new(), true, String::new())
    }

    #[test]
    fn csw_without_target() {
        let data = command::Data {
            cmd: "csw".into(),
            args: "".into(),
        };
        let out = SpiderGuild::cast_spider_wrath(&data, &mut ctx()).unwrap();
        assert_eq!(out, "@cast 'spider wrath'");
    }

    #[test]
    fn csw_with_target() {
        let data = command::Data {
            cmd: "csw".into(),
            args: "orc".into(),
        };
        let out = SpiderGuild::cast_spider_wrath(&data, &mut ctx()).unwrap();
        assert_eq!(out, "@target orc;cast 'spider wrath' orc");
    }

    #[test]
    fn chs_requires_args() {
        let data = command::Data {
            cmd: "chs".into(),
            args: "".into(),
        };
        assert!(SpiderGuild::cast_hunger_of_the_spider(&data, &mut ctx()).is_none());
    }

    #[test]
    fn chs_strips_and_sends() {
        let data = command::Data {
            cmd: "chs".into(),
            args: "  troll  ".into(),
        };
        let out = SpiderGuild::cast_hunger_of_the_spider(&data, &mut ctx()).unwrap();
        assert_eq!(out, "@target troll;cast hunger of the spider at troll");
    }

    #[test]
    fn csum_allows_empty_with_tail() {
        let data = command::Data {
            cmd: "csum".into(),
            args: "".into(),
        };
        let out = SpiderGuild::cast_spider_demon_conjuration(&data, &mut ctx()).unwrap();
        assert_eq!(out, "@cast spider demon conjuration at me with ");
    }

    #[test]
    fn csum_with_conjuration_material() {
        let data = command::Data {
            cmd: "csum".into(),
            args: "thing".into(),
        };
        let out = SpiderGuild::cast_spider_demon_conjuration(&data, &mut ctx()).unwrap();
        assert_eq!(out, "@cast spider demon conjuration at me with thing");
    }

    #[test]
    fn ctrl_fixed_at_me() {
        let data = command::Data {
            cmd: "ctrl".into(),
            args: "ignored".into(),
        };
        let out = SpiderGuild::cast_spider_demon_control(&data, &mut ctx()).unwrap();
        assert_eq!(out, "@cast spider demon control at me");
    }

    #[test]
    fn ctd_me_when_empty() {
        let data = command::Data {
            cmd: "ctd".into(),
            args: "".into(),
        };
        let out = SpiderGuild::cast_toxic_dilution(&data, &mut ctx()).unwrap();
        assert_eq!(out, "@cast toxic dilution at me");
    }

    #[test]
    fn ctd_quoted_when_args() {
        let data = command::Data {
            cmd: "ctd".into(),
            args: "foo".into(),
        };
        let out = SpiderGuild::cast_toxic_dilution(&data, &mut ctx()).unwrap();
        assert_eq!(out, "@cast 'toxic dilution' foo");
    }

    #[test]
    fn cvb_without_target_echoes_red_and_sends_none() {
        let data = command::Data {
            cmd: "cvb".into(),
            args: "".into(),
        };
        let mut cx = ctx();
        assert!(SpiderGuild::cast_venom_blade(&data, &mut cx).is_none(),);
        assert_eq!(cx.output_lines.len(), 1);
        assert_eq!(cx.output_lines[0].plain_line, "No target!");
    }

    #[test]
    fn cvb_with_target() {
        let data = command::Data {
            cmd: "cvb".into(),
            args: "orc".into(),
        };
        let out = SpiderGuild::cast_venom_blade(&data, &mut ctx()).unwrap();
        assert_eq!(out, "@cast 'venom blade' orc");
    }

    #[test]
    fn cswalk_and_chw_branches() {
        let mut cx = ctx();
        assert_eq!(
            SpiderGuild::cast_spider_walk(
                &command::Data {
                    cmd: "cswalk".into(),
                    args: "".into(),
                },
                &mut cx
            )
            .as_deref(),
            Some("@cast spider walk at me")
        );
        assert_eq!(
            SpiderGuild::cast_spider_walk(
                &command::Data {
                    cmd: "cswalk".into(),
                    args: "x".into(),
                },
                &mut cx,
            )
            .as_deref(),
            Some("@cast 'spider walk' x")
        );
        assert_eq!(
            SpiderGuild::cast_heavy_weight(
                &command::Data {
                    cmd: "chw".into(),
                    args: "".into(),
                },
                &mut cx,
            )
            .as_deref(),
            Some("@cast heavy weight at me")
        );
        assert_eq!(
            SpiderGuild::cast_heavy_weight(
                &command::Data {
                    cmd: "chw".into(),
                    args: "orc".into(),
                },
                &mut cx,
            )
            .as_deref(),
            Some("@cast 'heavy weight' orc")
        );
    }

    #[test]
    fn cmsac_cpsq() {
        let mut cx = ctx();
        assert_eq!(
            SpiderGuild::cast_spider_demon_mass_sacrifice(
                &command::Data {
                    cmd: "cmsac".into(),
                    args: "".into(),
                },
                &mut cx,
            )
            .as_deref(),
            Some("@cast spider demon mass sacrifice")
        );
        assert_eq!(
            SpiderGuild::cast_prayer_to_the_spider_queen(
                &command::Data {
                    cmd: "cpsq".into(),
                    args: "".into(),
                },
                &mut cx,
            )
            .as_deref(),
            Some("@cast prayer to the spider queen")
        );
    }

    #[test]
    fn crmp_branching() {
        let mut cx = ctx();
        assert_eq!(
            SpiderGuild::cast_remove_poison(
                &command::Data {
                    cmd: "crmp".into(),
                    args: "".into(),
                },
                &mut cx,
            )
            .as_deref(),
            Some("@cast remove poison at me")
        );
        assert_eq!(
            SpiderGuild::cast_remove_poison(
                &command::Data {
                    cmd: "crmp".into(),
                    args: "ally".into(),
                },
                &mut cx,
            )
            .as_deref(),
            Some("@cast remove poison at ally")
        );
    }

    #[test]
    fn use_stab_empty_and_with_target() {
        let mut cx = ctx();
        assert_eq!(
            SpiderGuild::use_stab(
                &command::Data {
                    cmd: "us".into(),
                    args: "".into(),
                },
                &mut cx,
            )
            .as_deref(),
            Some("@use 'stab'")
        );
        assert_eq!(
            SpiderGuild::use_stab(
                &command::Data {
                    cmd: "us".into(),
                    args: "orc".into(),
                },
                &mut cx,
            )
            .as_deref(),
            Some("@target orc;use 'stab' orc")
        );
    }
}
