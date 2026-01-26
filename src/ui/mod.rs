use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;

pub struct ViewModel<'a> {
    pub output_lines: Vec<Line<'a>>,
    pub scroll_offset: u16,
    pub show_stats: bool,
    pub stats_line: Line<'static>,
    pub clock: String,
    pub input_text: String,
    pub cursor_offset: u16,
}

pub struct Renderer;

impl Renderer {
    pub fn render(frame: &mut Frame<'_>, view: &ViewModel<'_>) {
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(frame.area());

        let output_area = root[0];
        let stats_area = root[1];
        let input_area = root[2];

        let output = Paragraph::new(Text::from(view.output_lines.clone()))
            .scroll((view.scroll_offset, 0));
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
        if input_inner.width > 0 && input_inner.height > 0 {
            let cursor_x = input_inner
                .x
                .saturating_add(view.cursor_offset.min(input_inner.width.saturating_sub(1)));
            let cursor_y = input_inner.y;
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
}
