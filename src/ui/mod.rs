use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};

pub struct ViewModel<'a> {
    pub output_lines: Vec<Line<'a>>,
    pub scroll_offset: u16,
    pub show_stats: bool,
    pub stats_line: Line<'static>,
    pub show_soul_stats: bool,
    pub soul_stats_line: Line<'static>,
    pub clock: String,
    pub input_text: String,
    pub cursor_offset: u16,
    pub show_cursor: bool,
    pub guild_dialog: Option<GuildDialogViewModel>,
    pub generic_commands_dialog: Option<GenericCommandsDialogViewModel>,
    pub settings_dialog: Option<SettingsDialogViewModel>,
}

pub struct GuildDialogItem {
    pub name: String,
    pub selected: bool,
}

pub struct GuildDialogViewModel {
    pub items: Vec<GuildDialogItem>,
    pub cursor: usize,
    pub mount_name: String,
    pub show_mount_input: bool,
    pub mount_input_cursor: usize,
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

impl Renderer {
    pub fn render(frame: &mut Frame<'_>, view: &ViewModel<'_>) {
        let mut constraints = vec![Constraint::Min(1), Constraint::Length(1)];
        if view.show_soul_stats {
            constraints.push(Constraint::Length(1));
        }
        constraints.push(Constraint::Length(1));

        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(frame.area());

        let output_area = root[0];
        let stats_area = root[1];
        let soul_stats_area = view.show_soul_stats.then_some(root[2]);
        let input_area = if view.show_soul_stats {
            root[3]
        } else {
            root[2]
        };

        let output =
            Paragraph::new(Text::from(view.output_lines.clone())).scroll((view.scroll_offset, 0));
        frame.render_widget(output, output_area);

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

        if let Some(soul_stats_area) = soul_stats_area {
            let soul_stats_widget = Paragraph::new(view.soul_stats_line.clone());
            frame.render_widget(soul_stats_widget, soul_stats_area);
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
    let area = centered_rect(60, 60, frame.area());
    frame.render_widget(Clear, area);

    let dialog_style = Style::default().bg(Color::Black).fg(Color::White);
    let background = Paragraph::new("").style(dialog_style);
    frame.render_widget(background, area);

    let block = Block::default()
        .title("Guilds")
        .borders(Borders::ALL)
        .style(dialog_style);
    frame.render_widget(&block, area);
    let inner = block.inner(area);

    // Adjust constraints based on whether mount input is shown
    let constraints = if dialog.show_mount_input {
        vec![
            Constraint::Min(1),    // guild list
            Constraint::Length(3), // mount name input
            Constraint::Length(1), // instructions
        ]
    } else {
        vec![
            Constraint::Min(1),    // guild list
            Constraint::Length(1), // instructions
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let items = dialog
        .items
        .iter()
        .map(|item| {
            let marker = if item.selected { "✓" } else { " " };
            ListItem::new(format!("{marker} {}", item.name))
        })
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

    // Render mount name input if Tzarakk is selected
    if dialog.show_mount_input {
        let mount_block = Block::default()
            .title("Mount Name (for Tzarakk)")
            .borders(Borders::ALL)
            .style(dialog_style);
        let mount_input = Paragraph::new(dialog.mount_name.clone()).block(mount_block.clone());
        frame.render_widget(mount_input, chunks[1]);

        // Show cursor in mount name field
        let mount_inner = mount_block.inner(chunks[1]);
        if mount_inner.width > 0 && mount_inner.height > 0 {
            let cursor_offset = dialog
                .mount_input_cursor
                .min(mount_inner.width as usize - 1) as u16;
            let cursor_x = mount_inner.x.saturating_add(cursor_offset);
            let cursor_y = mount_inner.y;
            frame.set_cursor_position((cursor_x, cursor_y));
        }

        let instructions = Paragraph::new(
            "Up/Down: move  Space: toggle  Tab: edit mount  Enter: save  Esc: cancel",
        )
        .style(dialog_style);
        frame.render_widget(instructions, chunks[2]);
    } else {
        let instructions = Paragraph::new("Up/Down: move  Space: toggle  Enter: save  Esc: cancel")
            .style(dialog_style);
        frame.render_widget(instructions, chunks[1]);
    }
}

fn render_settings_dialog(frame: &mut Frame<'_>, dialog: &SettingsDialogViewModel) {
    let area = centered_rect(60, 60, frame.area());
    frame.render_widget(Clear, area);

    let dialog_style = Style::default().bg(Color::Black);
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

    let instructions =
        Paragraph::new("Type: edit  Backspace: delete  Up/Down: move  Enter: save  Esc: cancel")
            .style(dialog_style);
    frame.render_widget(instructions, chunks[1]);
}

fn render_generic_commands_dialog(frame: &mut Frame<'_>, dialog: &GenericCommandsDialogViewModel) {
    let area = centered_rect(70, 70, frame.area());
    frame.render_widget(Clear, area);

    let dialog_style = Style::default().bg(Color::Black).fg(Color::White);
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
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White));
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
