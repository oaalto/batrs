use crate::ansi::StyledLine;
use crate::command::catalog::{ShortcutEntry, TriggerCatalogEntry};
use crate::command::{CommandEffect, ParsedCommand};
use crate::generic_commands::GenericCommands;
use crate::guilds::Guild;
use crate::guilds::catalog::GuildSelection;
use crate::guilds::catalog::{GuildPlayability, entry_for_persisted_key};
use crate::triggers::TriggerConfig;
use std::collections::HashMap;

pub struct ShowContext<'a> {
    pub guild_selection: &'a GuildSelection,
    pub active_guilds: &'a [Box<dyn Guild>],
    pub generic: &'a GenericCommands,
    pub trigger_config: &'a TriggerConfig,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShowMode {
    Commands,
    Triggers,
}

pub fn dispatch_show(parsed: &ParsedCommand, ctx: &ShowContext<'_>) -> Vec<CommandEffect> {
    let args: Vec<&str> = parsed.args.split_whitespace().collect();
    let Some(mode_str) = args.first() else {
        return usage_lines();
    };
    let mode = match mode_str.to_ascii_lowercase().as_str() {
        "commands" => ShowMode::Commands,
        "triggers" => ShowMode::Triggers,
        _ => return usage_lines(),
    };
    if args.len() > 2 {
        return error_lines(&["Usage: /show commands|triggers [guild|generic]"]);
    }
    let filter = args.get(1).map(|value| value.to_ascii_lowercase());

    match filter.as_deref() {
        Some("generic") => match mode {
            ShowMode::Commands => generic_shortcuts(ctx.generic),
            ShowMode::Triggers => common_triggers(ctx.trigger_config),
        },
        Some(guild_key) => filtered_guild(mode, guild_key, ctx),
        None => unfiltered(mode, ctx),
    }
}

fn usage_lines() -> Vec<CommandEffect> {
    output_lines(&[
        "Usage: /show commands|triggers [guild|generic]",
        "  /show commands        - Guild and generic shortcuts for active guilds",
        "  /show commands monk   - Shortcuts for one guild (catalog key)",
        "  /show commands generic - Generic shortcuts only",
        "  /show triggers        - Guild and common line triggers",
        "  /show triggers generic - Common triggers only",
    ])
}

fn error_lines(lines: &[&str]) -> Vec<CommandEffect> {
    output_lines(lines)
}

fn output_lines(lines: &[&str]) -> Vec<CommandEffect> {
    lines
        .iter()
        .map(|line| CommandEffect::Output(StyledLine::new(line)))
        .collect()
}

fn unfiltered(mode: ShowMode, ctx: &ShowContext<'_>) -> Vec<CommandEffect> {
    let owners = effective_shortcut_owners(ctx.active_guilds);
    let mut lines = Vec::new();
    let mut any = false;

    for (index, (source, guild)) in guild_sources(ctx.guild_selection)
        .iter()
        .zip(ctx.active_guilds.iter())
        .enumerate()
    {
        let section = match mode {
            ShowMode::Commands => guild_commands_section(
                source,
                guild.as_ref(),
                ctx.trigger_config,
                Some(&owners),
                index,
            ),
            ShowMode::Triggers => {
                guild_triggers_section(source, guild.as_ref(), ctx.trigger_config)
            }
        };
        if !section.is_empty() {
            any = true;
            lines.extend(section);
            lines.push(String::new());
        }
    }

    let generic_section = match mode {
        ShowMode::Commands => generic_shortcut_lines(ctx.generic),
        ShowMode::Triggers => common_trigger_lines(ctx.trigger_config),
    };
    if !generic_section.is_empty() {
        any = true;
        lines.extend(generic_section);
    }

    if !any {
        let message = match mode {
            ShowMode::Commands => "No shortcuts configured.",
            ShowMode::Triggers => "No triggers configured.",
        };
        return error_lines(&[message]);
    }

    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    output_lines(&lines.iter().map(String::as_str).collect::<Vec<_>>())
}

fn filtered_guild(mode: ShowMode, guild_key: &str, ctx: &ShowContext<'_>) -> Vec<CommandEffect> {
    let Some(entry) = entry_for_persisted_key(guild_key) else {
        return error_lines(&[&format!("Unknown guild: {guild_key}")]);
    };
    if matches!(entry.playability, GuildPlayability::BackgroundOnly { .. })
        && entry.persisted_key != ctx.guild_selection.primary_background_keyword()
    {
        return error_lines(&[&format!("Background guild not active: {guild_key}")]);
    }
    let Some(guild) = entry.build() else {
        return error_lines(&[&format!("Unknown guild: {guild_key}")]);
    };

    let source = GuildSource {
        display_name: entry.display_name,
    };
    let lines = match mode {
        ShowMode::Commands => {
            guild_commands_section(&source, guild.as_ref(), ctx.trigger_config, None, 0)
        }
        ShowMode::Triggers => guild_triggers_section(&source, guild.as_ref(), ctx.trigger_config),
    };
    if lines.is_empty() {
        let message = match mode {
            ShowMode::Commands => format!("No shortcuts for {guild_key}."),
            ShowMode::Triggers => format!("No triggers for {guild_key}."),
        };
        return error_lines(&[&message]);
    }
    output_lines(&lines.iter().map(String::as_str).collect::<Vec<_>>())
}

fn generic_shortcuts(generic: &GenericCommands) -> Vec<CommandEffect> {
    let lines = generic_shortcut_lines(generic);
    if lines.is_empty() {
        return error_lines(&["No generic shortcuts configured."]);
    }
    output_lines(&lines.iter().map(String::as_str).collect::<Vec<_>>())
}

fn common_triggers(trigger_config: &TriggerConfig) -> Vec<CommandEffect> {
    let lines = common_trigger_lines(trigger_config);
    if lines.is_empty() {
        return error_lines(&["No common triggers configured."]);
    }
    output_lines(&lines.iter().map(String::as_str).collect::<Vec<_>>())
}

struct GuildSource {
    display_name: &'static str,
}

fn guild_sources(selection: &GuildSelection) -> Vec<GuildSource> {
    let mut sources = Vec::new();
    if let Some(entry) = entry_for_persisted_key(selection.primary_background_keyword())
        .filter(|entry| matches!(entry.playability, GuildPlayability::BackgroundOnly { .. }))
    {
        sources.push(GuildSource {
            display_name: entry.display_name,
        });
    }
    for key in selection.persisted_keys() {
        if let Some(entry) = entry_for_persisted_key(&key) {
            sources.push(GuildSource {
                display_name: entry.display_name,
            });
        }
    }
    sources
}

fn guild_commands_section(
    source: &GuildSource,
    guild: &dyn Guild,
    _trigger_config: &TriggerConfig,
    owners: Option<&HashMap<String, usize>>,
    guild_index: usize,
) -> Vec<String> {
    let mut entries = guild.shortcut_catalog();
    if entries.is_empty() {
        return Vec::new();
    }
    entries.sort_by(|left, right| left.alias.cmp(right.alias));
    let mut lines = vec![format!("=== {} ===", source.display_name)];
    for entry in entries {
        let mut line = format_shortcut(entry);
        if let Some(owners) = owners
            && owners.get(entry.alias).copied() == Some(guild_index)
        {
            line.push('*');
        }
        lines.push(line);
    }
    lines
}

fn guild_triggers_section(
    source: &GuildSource,
    guild: &dyn Guild,
    trigger_config: &TriggerConfig,
) -> Vec<String> {
    let mut entries = guild.trigger_catalog();
    if entries.is_empty() {
        return Vec::new();
    }
    entries.sort_by(|left, right| left.pattern.cmp(&right.pattern));
    let mut header = format!("=== {} ===", source.display_name);
    if !trigger_config.guild_triggers {
        header.push_str(" (guild triggers off)");
    }
    let mut lines = vec![header];
    for entry in entries {
        lines.push(format_trigger(&entry));
    }
    lines
}

fn generic_shortcut_lines(generic: &GenericCommands) -> Vec<String> {
    let mut lines = vec!["=== Generic shortcuts ===".to_string()];
    let mut entries: Vec<(&str, &str, bool)> = Vec::new();
    for group in &generic.groups {
        for command in &group.commands {
            entries.push((&command.alias, command.description(), command.enabled));
        }
    }
    if entries.is_empty() {
        return Vec::new();
    }
    entries.sort_by(|left, right| left.0.cmp(right.0));
    for (alias, description, enabled) in entries {
        let mut line = format!("{alias} - {description}");
        if !enabled {
            line.push_str(" (disabled)");
        }
        lines.push(line);
    }
    lines
}

fn common_trigger_lines(trigger_config: &TriggerConfig) -> Vec<String> {
    let mut lines = vec!["=== Common triggers ===".to_string()];
    if !trigger_config.common_triggers {
        lines[0].push_str(" (common triggers off)");
    }
    let mut entries = crate::triggers::common_trigger_catalog();
    entries.sort_by(|left, right| left.pattern.cmp(&right.pattern));
    for entry in entries {
        lines.push(format_trigger(&entry));
    }
    lines
}

fn format_shortcut(entry: ShortcutEntry) -> String {
    format!("{} - {}", entry.alias, entry.description)
}

fn format_trigger(entry: &TriggerCatalogEntry) -> String {
    format!("{} - {}", entry.pattern, entry.description)
}

fn effective_shortcut_owners(active_guilds: &[Box<dyn Guild>]) -> HashMap<String, usize> {
    let mut owners = HashMap::new();
    for (index, guild) in active_guilds.iter().enumerate() {
        for alias in guild.commands().keys() {
            owners.entry(alias.clone()).or_insert(index);
        }
    }
    owners
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;
    use crate::generic_commands::GenericCommands;
    use crate::guilds::Guild;
    use crate::guilds::catalog::GuildSelection;
    use std::collections::HashMap;

    struct CatalogGuild {
        shortcuts: Vec<ShortcutEntry>,
        handlers: HashMap<String, Command>,
    }

    impl Guild for CatalogGuild {
        fn commands(&self) -> HashMap<String, Command> {
            self.handlers.clone()
        }

        fn triggers(&self) -> Vec<crate::triggers::Trigger> {
            Vec::new()
        }

        fn shortcut_catalog(&self) -> Vec<ShortcutEntry> {
            self.shortcuts.clone()
        }
    }

    fn noop_handler(
        _data: &ParsedCommand,
        _env: &crate::command::CommandEnvironment,
    ) -> Vec<CommandEffect> {
        Vec::new()
    }

    fn show_context<'a>(
        selection: &'a GuildSelection,
        guilds: &'a [Box<dyn Guild>],
        generic: &'a GenericCommands,
        trigger_config: &'a TriggerConfig,
    ) -> ShowContext<'a> {
        ShowContext {
            guild_selection: selection,
            active_guilds: guilds,
            generic,
            trigger_config,
        }
    }

    #[test]
    fn show_commands_lists_active_guild_and_generic() {
        let guilds: Vec<Box<dyn Guild>> = vec![Box::new(CatalogGuild {
            shortcuts: vec![ShortcutEntry::new("ping", "Test shortcut.")],
            handlers: HashMap::from([("ping".to_string(), noop_handler as Command)]),
        })];
        let selection = GuildSelection::from_persisted_keys(&["monk".to_string()], Some("magical"));
        let generic = GenericCommands::default();
        let trigger_config = TriggerConfig::default();
        let ctx = show_context(&selection, &guilds, &generic, &trigger_config);
        let effects = dispatch_show(&ParsedCommand::new("/show commands"), &ctx);
        let lines: Vec<String> = effects
            .iter()
            .filter_map(|effect| match effect {
                CommandEffect::Output(line) => Some(line.plain_line.clone()),
                _ => None,
            })
            .collect();
        assert!(
            lines
                .iter()
                .any(|line| line.contains("ping - Test shortcut."))
        );
        assert!(lines.iter().any(|line| line.contains("Generic shortcuts")));
        assert!(lines.iter().any(|line| line.contains("clw -")));
    }

    #[test]
    fn show_commands_generic_filter_lists_only_generic() {
        let selection = GuildSelection::default();
        let generic = GenericCommands::default();
        let trigger_config = TriggerConfig::default();
        let ctx = show_context(&selection, &[], &generic, &trigger_config);
        let effects = dispatch_show(&ParsedCommand::new("/show commands generic"), &ctx);
        let lines: Vec<String> = effects
            .iter()
            .filter_map(|effect| match effect {
                CommandEffect::Output(line) => Some(line.plain_line.clone()),
                _ => None,
            })
            .collect();
        assert!(
            lines
                .iter()
                .all(|line| !line.starts_with("===") || line.contains("Generic"))
        );
        assert!(lines.iter().any(|line| line.contains("clw -")));
    }

    #[test]
    fn show_unknown_guild_reports_error() {
        let selection = GuildSelection::default();
        let generic = GenericCommands::default();
        let trigger_config = TriggerConfig::default();
        let ctx = show_context(&selection, &[], &generic, &trigger_config);
        let effects = dispatch_show(&ParsedCommand::new("/show commands missing"), &ctx);
        let lines: Vec<String> = effects
            .iter()
            .filter_map(|effect| match effect {
                CommandEffect::Output(line) => Some(line.plain_line.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(lines, vec!["Unknown guild: missing".to_string()]);
    }
}
