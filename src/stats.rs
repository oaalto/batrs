use egui::{Color32, ProgressBar, RichText};

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

    pub fn show(&self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.horizontal(|ui| {
                ui.label("Hp:");
                self.show_stat_progress_bar(ui, self.hp, self.max_hp, self.diff_hp)
            });

            ui.horizontal(|ui| {
                ui.label("Sp:");
                self.show_stat_progress_bar(ui, self.sp, self.max_sp, self.diff_sp)
            });

            ui.horizontal(|ui| {
                ui.label("Ep:");
                self.show_stat_progress_bar(ui, self.ep, self.max_ep, self.diff_ep)
            });

            ui.horizontal(|ui| {
                ui.label("Exp:");
                self.show_value_label(ui, self.exp, self.diff_exp);
            });

            ui.horizontal(|ui| {
                ui.label("Money:");
                self.show_value_label(ui, self.money, self.diff_money);
            });
        });
    }

    fn show_value_label(&self, ui: &mut egui::Ui, value: i32, diff: i32) {
        let text = if diff == 0 {
            format!("{}", value)
        } else {
            format!("{} ({:+})", value, diff)
        };
        ui.label(text);
    }

    fn show_stat_progress_bar(&self, ui: &mut egui::Ui, value: i32, max_value: i32, diff: i32) {
        let progress_text = if diff == 0 {
            format!("{}/{}", value, max_value)
        } else {
            format!("{}/{} ({:+})", value, max_value, diff)
        };
        let progress: f32 = if value == 0 {
            0.0
        } else {
            value as f32 / max_value as f32
        };
        let progress = ProgressBar::new(progress)
            .fill(progress_color(progress))
            .text(RichText::new(progress_text).color(Color32::BLACK));
        ui.add(progress);
    }
}

fn progress_color(value: f32) -> Color32 {
    if value < 0.1 {
        Color32::DARK_RED
    } else if value < 0.2 {
        Color32::RED
    } else if value < 0.3 {
        Color32::LIGHT_RED
    } else if value < 0.4 {
        Color32::YELLOW
    } else if value < 0.5 {
        Color32::LIGHT_YELLOW
    } else if value < 0.6 {
        Color32::DARK_BLUE
    } else if value < 0.7 {
        Color32::BLUE
    } else if value < 0.8 {
        Color32::LIGHT_BLUE
    } else if value < 0.9 {
        Color32::DARK_GREEN
    } else {
        Color32::GREEN
    }
}
