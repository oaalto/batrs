use crate::ansi::palette;
use crate::combat_awareness::{CombatCondition, CombatScanRow};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use unicode_width::UnicodeWidthStr;

pub struct ViewModel<'a> {
    pub output_lines: Vec<Line<'a>>,
    pub scroll_offset: u16,
    pub show_stats: bool,
    pub stats_line: Line<'static>,
    /// Combat scan rows shown above the main stats line.
    pub combat_status_lines: Vec<Line<'static>>,
    /// Guild-specific HUD rows below the main stats line, such as Soul, Tzarakk mount, and Nergal minions.
    pub secondary_status_lines: Vec<Line<'static>>,
    pub clock: String,
    pub input_text: String,
    pub cursor_offset: u16,
    pub show_cursor: bool,
    pub guild_dialog: Option<GuildDialogViewModel>,
    pub generic_commands_dialog: Option<GenericCommandsDialogViewModel>,
    pub settings_dialog: Option<SettingsDialogViewModel>,
}

pub struct RiftwalkerEntityRowVm {
    pub title: &'static str,
    pub value: String,
    pub cursor: usize,
    pub active_row: bool,
}

#[derive(Clone, Debug)]
pub enum GuildDialogPresentation {
    BrowseRows {
        labels: Vec<String>,
        cursor: usize,
        /// Thematic-only index 0–4 matching the five civilization-style backgrounds.
        active_primary_index: usize,
    },
    GuildDrill {
        subtitle: String,
        lines: Vec<GuildDrillLineVm>,
        cursor: usize,
    },
}

#[derive(Clone, Debug)]
pub enum GuildDrillLineVm {
    Banner(String),
    Guild { title: String, selected: bool },
}

pub struct GuildDialogViewModel {
    pub presentation: GuildDialogPresentation,
    pub mount_name: String,
    pub show_mount_input: bool,
    pub mount_input_cursor: usize,
    pub mount_input_focused: bool,
    pub sabre_weapon: String,
    pub show_sabre_weapon_input: bool,
    pub sabre_weapon_input_cursor: usize,
    pub sabre_weapon_input_focused: bool,
    pub show_riftwalker_entity_inputs: bool,
    pub riftwalker_rows: Vec<RiftwalkerEntityRowVm>,
}

pub struct SettingsDialogItem {
    pub key: String,
    pub value: String,
}

pub struct SettingsDialogViewModel {
    pub items: Vec<SettingsDialogItem>,
    pub cursor: usize,
}

/// View model for a generic command in the dialog
pub struct GenericCommandViewModel {
    pub alias: String,
    pub command: String,
    pub selected: bool,
    /// Indentation level: 0 = "All", 1 = group, 2 = command
    pub level: usize,
}

/// View model for the generic commands dialog
pub struct GenericCommandsDialogViewModel {
    pub items: Vec<GenericCommandViewModel>,
    pub cursor: usize,
}

pub struct Renderer;

/// Render condition-colored combat scan rows for the HUD, wrapping at `width`.
///
/// Returns no lines when combat is inactive or the snapshot is empty.
pub fn render_combat_status_lines(
    active: bool,
    rows: &[CombatScanRow],
    width: u16,
) -> Vec<Line<'static>> {
    if !active || rows.is_empty() {
        return Vec::new();
    }

    let pieces: Vec<Vec<Span<'static>>> = rows.iter().map(combat_row_spans).collect();
    wrap_combat_pieces(pieces, width)
}

fn combat_condition_color(condition: CombatCondition) -> Color {
    match condition {
        CombatCondition::Excellent | CombatCondition::Good => palette::GREEN,
        CombatCondition::SlightlyHurt | CombatCondition::NoticeablyHurt => palette::CYAN,
        CombatCondition::NotGood | CombatCondition::Bad => palette::YELLOW,
        CombatCondition::VeryBad | CombatCondition::NearDeath => palette::RED,
    }
}

fn combat_row_spans(row: &CombatScanRow) -> Vec<Span<'static>> {
    let condition = row.condition();
    let color = combat_condition_color(condition);
    vec![
        Span::styled(row.name().to_string(), enemy_name_style()),
        Span::styled(" is ", normal_text_style()),
        Span::styled(condition.label().to_string(), Style::default().fg(color)),
        Span::styled(" (", normal_text_style()),
        Span::styled(row.percent().to_string(), Style::default().fg(color)),
        Span::styled("%).", normal_text_style()),
    ]
}

fn wrap_combat_pieces(pieces: Vec<Vec<Span<'static>>>, width: u16) -> Vec<Line<'static>> {
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

impl Renderer {
    pub fn render(frame: &mut Frame<'_>, view: &ViewModel<'_>) {
        let mut constraints = vec![Constraint::Min(1)];
        for _ in &view.combat_status_lines {
            constraints.push(Constraint::Length(1));
        }
        constraints.push(Constraint::Length(1));
        for _ in &view.secondary_status_lines {
            constraints.push(Constraint::Length(1));
        }
        constraints.push(Constraint::Length(1));

        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(frame.area());

        let output_area = root[0];
        let combat_count = view.combat_status_lines.len();
        let stats_area = root[1 + combat_count];
        let secondary_count = view.secondary_status_lines.len();
        let input_area = root[1 + combat_count + 1 + secondary_count];

        let output =
            Paragraph::new(Text::from(view.output_lines.clone())).scroll((view.scroll_offset, 0));
        frame.render_widget(output, output_area);

        for index in 0..combat_count {
            let line = view.combat_status_lines[index].clone();
            let row_area = root[1 + index];
            frame.render_widget(Paragraph::new(line), row_area);
        }

        if view.show_stats {
            let stats_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(10), Constraint::Length(12)])
                .split(stats_area);

            let stats_widget = Paragraph::new(view.stats_line.clone());
            frame.render_widget(stats_widget, stats_chunks[0]);

            let clock_widget = Paragraph::new(view.clock.clone()).alignment(Alignment::Center);
            frame.render_widget(clock_widget, stats_chunks[1]);
        } else {
            let clock_widget = Paragraph::new(view.clock.clone()).alignment(Alignment::Right);
            frame.render_widget(clock_widget, stats_area);
        }

        for index in 0..secondary_count {
            let line = view.secondary_status_lines[index].clone();
            let row_area = root[1 + combat_count + 1 + index];
            frame.render_widget(Paragraph::new(line), row_area);
        }

        let input_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Length(12)])
            .split(input_area);

        let input_block = Block::default();
        let input = Paragraph::new(view.input_text.clone())
            .block(input_block.clone())
            .wrap(Wrap { trim: false });
        frame.render_widget(input, input_chunks[0]);

        frame.render_widget(Paragraph::new(""), input_chunks[1]);

        let input_inner = input_block.inner(input_chunks[0]);
        if view.show_cursor && input_inner.width > 0 && input_inner.height > 0 {
            let cursor_x = input_inner
                .x
                .saturating_add(view.cursor_offset.min(input_inner.width.saturating_sub(1)));
            let cursor_y = input_inner.y;
            frame.set_cursor_position((cursor_x, cursor_y));
        }

        if let Some(dialog) = &view.guild_dialog {
            render_guild_dialog(frame, dialog);
        }
        if let Some(dialog) = &view.generic_commands_dialog {
            render_generic_commands_dialog(frame, dialog);
        }
        if let Some(dialog) = &view.settings_dialog {
            render_settings_dialog(frame, dialog);
        }
    }
}

fn render_guild_dialog(frame: &mut Frame<'_>, dialog: &GuildDialogViewModel) {
    use ratatui::style::Modifier;

    let area = centered_rect(60, 60, frame.area());
    frame.render_widget(Clear, area);

    let dialog_style = Style::default().bg(palette::SURFACE).fg(palette::TEXT);
    let background = Paragraph::new("").style(dialog_style);
    frame.render_widget(background, area);

    let title = Block::default()
        .title("Guilds")
        .borders(Borders::ALL)
        .style(dialog_style);
    frame.render_widget(&title, area);
    let inner = title.inner(area);

    let instructions_text: &'static str = match &dialog.presentation {
        GuildDialogPresentation::BrowseRows { .. } => concat!(
            "Up/Down: move  ",
            "Space: set thematic primary (first five backgrounds)  ",
            "Enter: open group  Esc: cancel"
        ),
        GuildDialogPresentation::GuildDrill { .. } => concat!(
            "Up/Down: move  ",
            "Space: toggle guild  ",
            "Tab / Shift+Tab: next field  Type: edit  ",
            "Enter: save  Esc: back to backgrounds"
        ),
    };

    let mut constraints: Vec<Constraint> = vec![Constraint::Min(1)];

    if dialog.show_mount_input {
        constraints.push(Constraint::Length(3));
    }
    if dialog.show_sabre_weapon_input {
        constraints.push(Constraint::Length(3));
    }
    if dialog.show_riftwalker_entity_inputs {
        constraints.push(Constraint::Length(8));
    }
    constraints.push(Constraint::Length(2));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let mut chunk_index = 0usize;

    match &dialog.presentation {
        GuildDialogPresentation::BrowseRows {
            labels,
            cursor,
            active_primary_index,
        } => {
            let thematic_row_count = labels.len().saturating_sub(1);
            let browse_items = labels
                .iter()
                .enumerate()
                .map(|(row_index, row_label)| {
                    let thematic_primary_row =
                        row_index < thematic_row_count && row_index == *active_primary_index;
                    ListItem::new(format!(
                        "{}{}",
                        if thematic_primary_row { "*" } else { " " },
                        row_label
                    ))
                })
                .collect::<Vec<ListItem<'_>>>();
            let browse_list = List::new(browse_items)
                .highlight_symbol("> ")
                .style(dialog_style)
                .highlight_style(dialog_style);
            let mut browse_state = ListState::default();
            if !labels.is_empty() {
                browse_state.select(Some((*cursor).min(labels.len().saturating_sub(1))));
            }
            frame.render_stateful_widget(browse_list, chunks[chunk_index], &mut browse_state);
            chunk_index += 1;
        }
        GuildDialogPresentation::GuildDrill {
            subtitle,
            lines,
            cursor,
        } => {
            let pane_area = chunks[chunk_index];
            let list_area = if subtitle.is_empty() {
                pane_area
            } else {
                let split = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(1), Constraint::Min(0)])
                    .split(pane_area);
                frame.render_widget(
                    Paragraph::new(subtitle.as_str()).style(dialog_style),
                    split[0],
                );
                split[1]
            };

            let drill_items = lines
                .iter()
                .map(|line_variant| match line_variant {
                    GuildDrillLineVm::Banner(message) => {
                        ListItem::new(Line::from(message.as_str()).dim())
                    }
                    GuildDrillLineVm::Guild {
                        title: title_text,
                        selected,
                    } => {
                        let marker = if *selected { "✓" } else { " " };
                        ListItem::new(format!("{marker} {title_text}"))
                    }
                })
                .collect::<Vec<_>>();

            let drill_widget = List::new(drill_items)
                .highlight_symbol("> ")
                .style(dialog_style)
                .highlight_style(dialog_style);
            let mut drill_state = ListState::default();
            if !lines.is_empty() {
                drill_state.select(Some((*cursor).min(lines.len().saturating_sub(1))));
            }
            frame.render_stateful_widget(drill_widget, list_area, &mut drill_state);
            chunk_index += 1;
        }
    }

    if dialog.show_mount_input {
        let mount_block = Block::default()
            .title("Mount Name (for Tzarakk)")
            .borders(Borders::ALL)
            .style(dialog_style);
        let mount_input = Paragraph::new(dialog.mount_name.clone()).block(mount_block.clone());
        frame.render_widget(mount_input, chunks[chunk_index]);

        let mount_inner = mount_block.inner(chunks[chunk_index]);
        if dialog.mount_input_focused && mount_inner.width > 0 && mount_inner.height > 0 {
            let cursor_offset = dialog
                .mount_input_cursor
                .min(mount_inner.width as usize - 1) as u16;
            let cursor_x = mount_inner.x.saturating_add(cursor_offset);
            let cursor_y = mount_inner.y;
            frame.set_cursor_position((cursor_x, cursor_y));
        }
        chunk_index += 1;
    }

    if dialog.show_sabre_weapon_input {
        let sabre_block = Block::default()
            .title("Main-hand weapon (Sabres)")
            .borders(Borders::ALL)
            .style(dialog_style);
        let sabre_input = Paragraph::new(dialog.sabre_weapon.clone()).block(sabre_block.clone());
        frame.render_widget(sabre_input, chunks[chunk_index]);

        let sabre_inner = sabre_block.inner(chunks[chunk_index]);
        if dialog.sabre_weapon_input_focused && sabre_inner.width > 0 && sabre_inner.height > 0 {
            let cursor_offset = dialog
                .sabre_weapon_input_cursor
                .min(sabre_inner.width as usize - 1) as u16;
            let cursor_x = sabre_inner.x.saturating_add(cursor_offset);
            let cursor_y = sabre_inner.y;
            frame.set_cursor_position((cursor_x, cursor_y));
        }
        chunk_index += 1;
    }

    if dialog.show_riftwalker_entity_inputs {
        let entities_block = Block::default()
            .title("Riftwalker entities")
            .borders(Borders::ALL)
            .style(dialog_style);
        let entity_area = chunks[chunk_index];
        frame.render_widget(entities_block.clone(), entity_area);
        let inner_entities = entities_block.inner(entity_area);
        let row_rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(inner_entities);

        for (row_vm, row_chunk) in dialog.riftwalker_rows.iter().zip(row_rects.iter()) {
            let label = format!("{:<5}: ", row_vm.title);
            let row_style = if row_vm.active_row {
                dialog_style.add_modifier(Modifier::REVERSED)
            } else {
                dialog_style
            };
            let paragraph = Paragraph::new(format!("{}{}", label, row_vm.value)).style(row_style);
            frame.render_widget(paragraph, *row_chunk);

            if row_vm.active_row && row_chunk.width > 0 {
                let label_display_len: u16 = label.chars().count() as u16;
                let value_width = row_chunk.width.saturating_sub(label_display_len);
                let cx = (row_vm.cursor as u16).min(value_width.saturating_sub(1));
                let cursor_x = row_chunk
                    .x
                    .saturating_add(label_display_len)
                    .saturating_add(cx);
                let cursor_y = row_chunk.y;
                frame.set_cursor_position((cursor_x, cursor_y));
            }
        }
        chunk_index += 1;
    }

    let instructions = Paragraph::new(instructions_text)
        .wrap(Wrap { trim: true })
        .style(dialog_style);
    frame.render_widget(instructions, chunks[chunk_index]);
}

fn render_settings_dialog(frame: &mut Frame<'_>, dialog: &SettingsDialogViewModel) {
    let area = centered_rect(60, 60, frame.area());
    frame.render_widget(Clear, area);

    let dialog_style = Style::default().bg(palette::SURFACE);
    let background = Paragraph::new("").style(dialog_style);
    frame.render_widget(background, area);

    let block = Block::default()
        .title("Settings")
        .borders(Borders::ALL)
        .style(dialog_style);
    frame.render_widget(&block, area);
    let inner = block.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    let items = dialog
        .items
        .iter()
        .map(|item| ListItem::new(format!("{}: {}", item.key, item.value)))
        .collect::<Vec<ListItem<'_>>>();

    let list = List::new(items)
        .highlight_symbol("> ")
        .style(dialog_style)
        .highlight_style(dialog_style);
    let mut state = ListState::default();
    if !dialog.items.is_empty() {
        state.select(Some(dialog.cursor.min(dialog.items.len() - 1)));
    }
    frame.render_stateful_widget(list, chunks[0], &mut state);
    set_settings_dialog_cursor_position(frame, dialog, chunks[0], &state);

    let instructions =
        Paragraph::new("Type: edit  Backspace: delete  Up/Down: move  Enter: save  Esc: cancel")
            .style(dialog_style);
    frame.render_widget(instructions, chunks[1]);
}

fn set_settings_dialog_cursor_position(
    frame: &mut Frame<'_>,
    dialog: &SettingsDialogViewModel,
    list_area: Rect,
    state: &ListState,
) {
    if dialog.items.is_empty() || list_area.width == 0 || list_area.height == 0 {
        return;
    }

    let selected_index = dialog.cursor.min(dialog.items.len() - 1);
    let Some(visible_row) = selected_index.checked_sub(state.offset()) else {
        return;
    };
    if visible_row >= list_area.height as usize {
        return;
    }

    let item = &dialog.items[selected_index];
    let cursor_offset = 2u16
        .saturating_add(item.key.chars().count() as u16)
        .saturating_add(2)
        .saturating_add(item.value.chars().count() as u16);
    let cursor_x = list_area
        .x
        .saturating_add(cursor_offset.min(list_area.width.saturating_sub(1)));
    let cursor_y = list_area.y.saturating_add(visible_row as u16);
    frame.set_cursor_position((cursor_x, cursor_y));
}

fn render_generic_commands_dialog(frame: &mut Frame<'_>, dialog: &GenericCommandsDialogViewModel) {
    let area = centered_rect(70, 70, frame.area());
    frame.render_widget(Clear, area);

    let dialog_style = Style::default().bg(palette::SURFACE).fg(palette::TEXT);
    let background = Paragraph::new("").style(dialog_style);
    frame.render_widget(background, area);

    let block = Block::default()
        .title("Generic Commands")
        .borders(Borders::ALL)
        .style(dialog_style);
    frame.render_widget(&block, area);
    let inner = block.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    let items = dialog
        .items
        .iter()
        .map(|item| {
            let indent = match item.level {
                0 => "",
                1 => "  ",
                2 => "    ",
                _ => "",
            };
            let marker = if item.selected { "[x]" } else { "[ ]" };
            let text = if item.level == 0 {
                format!("{indent}{marker} All Commands")
            } else if item.level == 1 {
                format!("{indent}{marker} {}", item.alias)
            } else {
                format!("{indent}{marker} {} → {}", item.alias, item.command)
            };
            ListItem::new(text)
        })
        .collect::<Vec<ListItem<'_>>>();

    let list = List::new(items)
        .highlight_symbol("> ")
        .style(dialog_style)
        .highlight_style(Style::default().bg(palette::SELECTION).fg(palette::TEXT));
    let mut state = ListState::default();
    if !dialog.items.is_empty() {
        state.select(Some(dialog.cursor.min(dialog.items.len() - 1)));
    }
    frame.render_stateful_widget(list, chunks[0], &mut state);

    let instructions = Paragraph::new("Up/Down: move  Space: toggle  Enter: save  Esc: cancel")
        .style(dialog_style);
    frame.render_widget(instructions, chunks[1]);
}

fn centered_rect(percent_x: u16, percent_y: u16, rect: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(rect);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat_awareness::CombatAwareness;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use ratatui::style::Modifier;

    fn line_text(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    #[test]
    fn render_combat_status_wraps_multiple_rows_at_narrow_width() {
        let mut state = CombatAwareness::default();
        state.handle_incoming_line("*** Round 1 ***");
        state.handle_incoming_line("scan all");
        state.handle_incoming_line("Guard is slightly hurt (70%).");
        state.handle_incoming_line("Orc is in bad shape (30%).");
        state.handle_incoming_line("done");

        let lines = render_combat_status_lines(state.is_active(), state.snapshot(), 40);
        assert_eq!(lines.len(), 2);
        assert_eq!(line_text(&lines[0]), "Guard is slightly hurt (70%).");
        assert_eq!(line_text(&lines[1]), "Orc is bad shape (30%).");
    }

    #[test]
    fn render_combat_status_styles_structured_scan_row() {
        let mut state = CombatAwareness::default();
        state.handle_incoming_line("*** Round 1 ***");
        state.handle_incoming_line("scan all");
        state.handle_incoming_line("Guard is noticeably hurt (50%).");
        state.handle_incoming_line("done");

        let lines = render_combat_status_lines(state.is_active(), state.snapshot(), 120);
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

    #[test]
    fn render_places_combat_status_above_stats() {
        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        let view = ViewModel {
            output_lines: vec!["output".into()],
            scroll_offset: 0,
            show_stats: true,
            stats_line: "player stats".into(),
            combat_status_lines: vec!["enemy status".into()],
            secondary_status_lines: vec!["guild status".into()],
            clock: "12:34".to_string(),
            input_text: ">look".to_string(),
            cursor_offset: 5,
            show_cursor: false,
            guild_dialog: None,
            generic_commands_dialog: None,
            settings_dialog: None,
        };

        terminal
            .draw(|frame| Renderer::render(frame, &view))
            .unwrap();
        let buffer = terminal.backend().buffer();

        assert_eq!(row_text(buffer, 0), "output");
        assert_eq!(row_text(buffer, 1), "enemy status");
        assert!(row_text(buffer, 2).starts_with("player stats"));
        assert_eq!(row_text(buffer, 3), "guild status");
        assert_eq!(row_text(buffer, 4), ">look");
    }

    fn row_text(buffer: &Buffer, y: u16) -> String {
        (0..buffer.area.width)
            .map(|x| buffer[(x, y)].symbol())
            .collect::<String>()
            .trim_end()
            .to_string()
    }
}
