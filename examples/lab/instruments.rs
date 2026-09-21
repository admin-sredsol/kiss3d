//! egui-based lab instruments: the scrolling strip chart.

use std::collections::VecDeque;

/// One measured series (e.g. "speed of the blue ball") drawn as a scrolling
/// line chart over the last `window_secs` of simulation time.
pub struct StripChart {
    series: VecDeque<(f32, f32)>,
    window_secs: f32,
    pub label: String,
    pub color: egui::Color32,
}

impl StripChart {
    pub fn new(label: impl Into<String>, color: egui::Color32) -> Self {
        Self {
            series: VecDeque::new(),
            window_secs: 12.0,
            label: label.into(),
            color,
        }
    }

    /// Record one sample. Call once per fixed step for simulation-accurate data.
    pub fn push(&mut self, t: f32, value: f32) {
        self.series.push_back((t, value));
        while self.series.front().is_some_and(|(t0, _)| t - t0 > self.window_secs) {
            self.series.pop_front();
        }
    }

    pub fn clear(&mut self) {
        self.series.clear();
    }

    /// Largest recorded value in the current window (for y scaling).
    pub fn max_value(&self) -> f32 {
        self.series
            .iter()
            .map(|(_, v)| *v)
            .fold(0.0_f32, f32::max)
    }

    /// Draw the plot area. `t1` is the newest simulation time; `y_max` is the
    /// shared vertical scale.
    pub fn draw(&self, ui: &mut egui::Ui, t1: f32, y_max: f32) {
        let (response, painter) =
            ui.allocate_painter(egui::vec2(340.0, 170.0), egui::Sense::hover());
        let rect = response.rect;
        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(14, 14, 20));

        let t0 = t1 - self.window_secs;
        let stroke_frame = egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 70, 90));
        painter.line_segment([rect.left_bottom(), rect.right_bottom()], stroke_frame);
        painter.line_segment([rect.left_bottom(), rect.left_top()], stroke_frame);

        let points: Vec<egui::Pos2> = self
            .series
            .iter()
            .filter_map(|(t, v)| {
                if *t < t0 {
                    return None;
                }
                let x = rect.left() + (t - t0) / self.window_secs * rect.width();
                let y = rect.bottom() - (v / y_max) * rect.height();
                Some(egui::Pos2::new(x, y))
            })
            .collect();
        if points.len() >= 2 {
            painter.add(egui::Shape::line(points, egui::Stroke::new(1.5, self.color)));
        }
    }
}
