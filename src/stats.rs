use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

#[derive(Default, Debug, Clone)]
pub struct Stats {
    hp: i32,
    max_hp: i32,
    diff_hp: i32,
    sp: i32,
    max_sp: i32,
    diff_sp: i32,
    ep: i32,
    max_ep: i32,
    diff_ep: i32,
    exp: i32,
    diff_exp: i32,
    money: i32,
    diff_money: i32,
}

impl Stats {
    pub fn new(stats: [i32; 7]) -> Self {
        Self {
            hp: stats[0],
            max_hp: stats[1],
            sp: stats[2],
            max_sp: stats[3],
            ep: stats[4],
            max_ep: stats[5],
            exp: stats[6],
            ..Default::default()
        }
    }

    pub fn new_from_sc(stats: [i32; 13]) -> Self {
        Self {
            hp: stats[0],
            max_hp: stats[1],
            diff_hp: stats[2],
            sp: stats[3],
            max_sp: stats[4],
            diff_sp: stats[5],
            ep: stats[6],
            max_ep: stats[7],
            diff_ep: stats[8],
            money: stats[9],
            diff_money: stats[10],
            exp: stats[11],
            diff_exp: stats[12],
        }
    }

    pub fn render_inline(&self) -> Line<'static> {
        let hp = self.inline_stat("HP", self.hp, self.max_hp, self.diff_hp, progress_color);
        let sp = self.inline_stat("SP", self.sp, self.max_sp, self.diff_sp, progress_color);
        let ep = self.inline_stat("EP", self.ep, self.max_ep, self.diff_ep, progress_color);
        let exp = self.inline_value("Exp", self.exp, self.diff_exp);
        let money = self.inline_value("Money", self.money, self.diff_money);

        Line::from(vec![
            hp,
            Span::raw("  "),
            sp,
            Span::raw("  "),
            ep,
            Span::raw("  "),
            exp,
            Span::raw("  "),
            money,
        ])
    }

    fn inline_stat(
        &self,
        label: &str,
        value: i32,
        max_value: i32,
        diff: i32,
        color_fn: fn(f32) -> Color,
    ) -> Span<'static> {
        let text = if diff == 0 {
            format!("{label}: {value}/{max_value}")
        } else {
            format!("{label}: {value}/{max_value} ({diff:+})")
        };
        let progress = if value == 0 || max_value == 0 {
            0.0
        } else {
            value as f32 / max_value as f32
        };
        Span::styled(text, Style::default().fg(color_fn(progress)))
    }

    fn inline_value(&self, label: &str, value: i32, diff: i32) -> Span<'static> {
        let text = if diff == 0 {
            format!("{label}: {value}")
        } else {
            format!("{label}: {value} ({diff:+})")
        };
        Span::raw(text)
    }
}

fn progress_color(value: f32) -> Color {
    if value < 0.1 {
        Color::Rgb(128, 0, 0)
    } else if value < 0.2 {
        Color::Red
    } else if value < 0.3 {
        Color::LightRed
    } else if value < 0.4 {
        Color::Yellow
    } else if value < 0.5 {
        Color::LightYellow
    } else if value < 0.6 {
        Color::Rgb(0, 0, 128)
    } else if value < 0.7 {
        Color::Blue
    } else if value < 0.8 {
        Color::LightBlue
    } else if value < 0.9 {
        Color::Rgb(0, 128, 0)
    } else {
        Color::Green
    }
}
