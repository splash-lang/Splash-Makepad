// Statistical distribution charts: BoxPlotChart (box-and-whisker) and
// ViolinPlot (kernel density estimation violins).
//
// Both are cartesian widgets with a categorical x-axis: items are laid out in
// bands [i, i+1] in data space, category labels are drawn under each band and
// y grid lines / tick labels are drawn by the widgets themselves (prototype
// sets show_grid/show_ticks false).

use crate::plot_view::*;
use crate::script_util::*;
use crate::types::*;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*

    mod.plot.BoxPlotChartBase = #(BoxPlotChart::register_widget(vm))

    mod.plot.BoxPlotChart = set_type_default() do mod.plot.BoxPlotChartBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 52.0, top: 28.0, right: 16.0, bottom: 34.0}
        show_grid: false
        show_ticks: false
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }

    mod.plot.ViolinPlotBase = #(ViolinPlot::register_widget(vm))

    mod.plot.ViolinPlot = set_type_default() do mod.plot.ViolinPlotBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 52.0, top: 28.0, right: 16.0, bottom: 34.0}
        show_grid: false
        show_ticks: false
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }
}

// =============================================================================
// Shared categorical-axis helpers
// =============================================================================

/// Draw horizontal grid lines + y tick labels for a categorical chart
/// (the prototype disables PlotView's own grid/ticks because the x axis is
/// categorical and numeric x labels would be meaningless).
fn draw_y_grid_and_ticks(pv: &mut PlotView, cx: &mut Cx2d) {
    let vp = pv.viewport.clone();
    let ticks = nice_ticks(vp.y_min, vp.y_max, 6);
    let pr = pv.plot_rect;
    let font_size = pv.tick_font_size;
    for &ty in &ticks {
        let (_, py) = pv.tdata_to_px(0.0, ty);
        pv.draw_grid_line_h_px(cx, py as f64);
        let label = format_tick_value(ScaleType::Linear, ty);
        let est_w = label.len() as f64 * font_size as f64 * 0.55;
        pv.draw_text_px(
            cx,
            (pr.pos.x - est_w - 6.0).max(pv.rect.pos.x),
            py as f64 - font_size as f64 * 0.6,
            &label,
            vec4(0.25, 0.25, 0.25, 1.0),
            font_size,
        );
    }
}

/// Draw category labels centered under each band [i, i+1].
fn draw_category_labels(pv: &mut PlotView, cx: &mut Cx2d, labels: &[String]) {
    let pr = pv.plot_rect;
    let font_size = pv.tick_font_size;
    for (i, label) in labels.iter().enumerate() {
        let (px, _) = pv.tdata_to_px(i as f64 + 0.5, 0.0);
        pv.draw_text_centered_px(
            cx,
            px as f64,
            pr.pos.y + pr.size.y + 11.0,
            label,
            vec4(0.25, 0.25, 0.25, 1.0),
            font_size,
        );
    }
}

// =============================================================================
// BoxPlotChart
// =============================================================================

/// Quartile / whisker / outlier statistics for a box plot
#[derive(Clone, Debug, Default)]
pub struct BoxPlotStats {
    pub min: f64,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub max: f64,
    pub outliers: Vec<f64>,
}

impl BoxPlotStats {
    pub fn from_values(values: &[f64]) -> Option<Self> {
        if values.is_empty() {
            return None;
        }

        let mut sorted: Vec<f64> = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let n = sorted.len();
        let median = if n % 2 == 0 {
            (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
        } else {
            sorted[n / 2]
        };

        let q1_idx = n / 4;
        let q3_idx = (3 * n / 4).min(n - 1);
        let q1 = sorted[q1_idx];
        let q3 = sorted[q3_idx];

        let iqr = q3 - q1;
        let lower_fence = q1 - 1.5 * iqr;
        let upper_fence = q3 + 1.5 * iqr;

        let outliers: Vec<f64> = sorted
            .iter()
            .filter(|&&v| v < lower_fence || v > upper_fence)
            .cloned()
            .collect();

        let whisker_min = sorted
            .iter()
            .find(|&&v| v >= lower_fence)
            .cloned()
            .unwrap_or(q1);
        let whisker_max = sorted
            .iter()
            .rev()
            .find(|&&v| v <= upper_fence)
            .cloned()
            .unwrap_or(q3);

        Some(BoxPlotStats {
            min: whisker_min,
            q1,
            median,
            q3,
            max: whisker_max,
            outliers,
        })
    }
}

/// Box plot data item
#[derive(Clone, Debug)]
pub struct BoxPlotItem {
    pub label: String,
    pub stats: BoxPlotStats,
    pub color: Option<Vec4>,
}

impl BoxPlotItem {
    pub fn new(label: impl Into<String>, values: &[f64]) -> Option<Self> {
        BoxPlotStats::from_values(values).map(|stats| BoxPlotItem {
            label: label.into(),
            stats,
            color: None,
        })
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = Some(color);
        self
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct BoxPlotChart {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub items: Vec<BoxPlotItem>,

    #[live(true)]
    pub show_outliers: bool,
    #[live(0.6)]
    pub box_width_ratio: f64,
    #[live(true)]
    pub demo_data: bool,

    #[rust]
    fitted: bool,
}

impl BoxPlotChart {
    // ---- Rust-side API (mirrors the 1.0 library surface) ----

    pub fn add_item(&mut self, item: BoxPlotItem) {
        self.items.push(item);
        self.fitted = false;
    }

    pub fn add_from_values(&mut self, label: impl Into<String>, values: &[f64]) {
        if let Some(item) = BoxPlotItem::new(label, values) {
            self.items.push(item);
            self.fitted = false;
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.fitted = false;
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn set_show_outliers(&mut self, show: bool) {
        self.show_outliers = show;
        self.fitted = false;
    }

    // ---- Demo data ----

    fn make_demo_data(&mut self) {
        let mut rng = DemoRng::new(42);
        let specs: [(&str, f64, f64); 4] = [
            ("A", 55.0, 8.0),
            ("B", 70.0, 12.0),
            ("C", 62.0, 6.0),
            ("D", 48.0, 10.0),
        ];
        for (label, mean, spread) in specs {
            let mut values = Vec::with_capacity(42);
            for _ in 0..40 {
                // Approximately normal: sum of 4 uniforms
                let u = rng.next_f64() + rng.next_f64() + rng.next_f64() + rng.next_f64();
                values.push(mean + (u - 2.0) * spread);
            }
            // A couple of clear outliers per category
            values.push(mean + spread * (3.2 + rng.next_f64()));
            values.push(mean - spread * (3.0 + rng.next_f64()));
            self.add_from_values(label, &values);
        }
    }

    // ---- Fitting ----

    fn y_range(&self) -> (f64, f64) {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for item in &self.items {
            min = min.min(item.stats.min);
            max = max.max(item.stats.max);
            if self.show_outliers {
                for &outlier in &item.stats.outliers {
                    min = min.min(outlier);
                    max = max.max(outlier);
                }
            }
        }
        if !min.is_finite() || !max.is_finite() {
            return (0.0, 1.0);
        }
        if min == max {
            min -= 0.5;
            max += 0.5;
        }
        let padding = (max - min) * 0.1;
        (min - padding, max + padding)
    }

    fn fit(&mut self) {
        let n = self.items.len().max(1);
        let (y_min, y_max) = self.y_range();
        self.plot_view.set_viewport(0.0, n as f64, y_min, y_max);
        self.fitted = true;
    }

    // ---- Drawing ----

    fn draw_boxes(&mut self, _cx: &mut Cx2d) {
        let items = self.items.clone();
        let half = self.box_width_ratio * 0.5;
        let whisker_color = vec4(0.3, 0.3, 0.3, 1.0);

        for (i, item) in items.iter().enumerate() {
            let color = item.color.unwrap_or_else(|| cycle_color(i));
            let xc = i as f64 + 0.5;
            let st = &item.stats;

            let (x_left, q3_py) = self.plot_view.data_to_px(xc - half, st.q3);
            let (x_right, q1_py) = self.plot_view.data_to_px(xc + half, st.q1);
            let (cx_px, median_py) = self.plot_view.data_to_px(xc, st.median);
            let (_, min_py) = self.plot_view.data_to_px(xc, st.min);
            let (_, max_py) = self.plot_view.data_to_px(xc, st.max);
            let box_w = x_right - x_left;

            // Box (Q1 to Q3)
            self.plot_view.fill_rect_px(
                x_left,
                q3_py,
                box_w,
                (q1_py - q3_py).max(1.0),
                color,
            );

            // Median line (white)
            self.plot_view.set_color(vec4(1.0, 1.0, 1.0, 1.0));
            self.plot_view.line_px(x_left, median_py, x_right, median_py, 2.0);

            // Whiskers
            self.plot_view.set_color(whisker_color);
            // Lower whisker + cap
            self.plot_view.line_px(cx_px, q1_py, cx_px, min_py, 1.0);
            self.plot_view.line_px(
                cx_px - box_w * 0.25,
                min_py,
                cx_px + box_w * 0.25,
                min_py,
                1.0,
            );
            // Upper whisker + cap
            self.plot_view.line_px(cx_px, q3_py, cx_px, max_py, 1.0);
            self.plot_view.line_px(
                cx_px - box_w * 0.25,
                max_py,
                cx_px + box_w * 0.25,
                max_py,
                1.0,
            );

            // Outliers
            if self.show_outliers {
                for &outlier in &st.outliers {
                    let (_, oy) = self.plot_view.data_to_px(xc, outlier);
                    self.plot_view.fill_circle_px(cx_px, oy, 3.0, color);
                }
            }
        }
    }
}

impl Widget for BoxPlotChart {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.items.is_empty() && self.demo_data {
            self.make_demo_data();
        }
        if !self.fitted {
            self.fit();
        }

        self.plot_view.begin(cx, walk);

        draw_y_grid_and_ticks(&mut self.plot_view, cx);
        self.plot_view.draw_axes(cx);
        self.draw_boxes(cx);

        let labels: Vec<String> = self.items.iter().map(|i| i.label.clone()).collect();
        draw_category_labels(&mut self.plot_view, cx, &labels);

        self.plot_view.end(cx);
        DrawStep::done()
    }

    fn script_call(
        &mut self,
        vm: &mut ScriptVm,
        method: LiveId,
        args: ScriptValue,
    ) -> ScriptAsyncResult {
        let mut handled = true;
        match method {
            x if x == live_id!(add_from_values) => {
                let label = script_arg_string(vm, &args, 0).unwrap_or_default();
                let values = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                self.add_from_values(label, &values);
            }
            x if x == live_id!(clear) => {
                self.clear();
            }
            x if x == live_id!(set_show_outliers) => {
                let show = script_arg_bool(vm, &args, 0).unwrap_or(true);
                self.set_show_outliers(show);
            }
            x if x == live_id!(set_title) => {
                self.plot_view.title = script_arg_string(vm, &args, 0).unwrap_or_default();
            }
            _ => {
                handled = false;
            }
        }
        if handled {
            vm.with_cx_mut(|cx| {
                self.plot_view.redraw(cx);
            });
            return ScriptAsyncResult::Return(NIL);
        }
        ScriptAsyncResult::MethodNotFound
    }
}

// =============================================================================
// ViolinPlot
// =============================================================================

/// Violin plot data item (raw sample values)
#[derive(Clone, Debug)]
pub struct ViolinItem {
    pub label: String,
    pub values: Vec<f64>,
    pub color: Option<Vec4>,
}

impl ViolinItem {
    pub fn new(label: impl Into<String>, values: Vec<f64>) -> Self {
        Self {
            label: label.into(),
            values,
            color: None,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = Some(color);
        self
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct ViolinPlot {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub items: Vec<ViolinItem>,

    #[live(true)]
    pub show_box: bool,
    #[live(true)]
    pub show_median: bool,
    /// KDE bandwidth; <= 0.0 means auto (Silverman's rule)
    #[live(0.0)]
    pub bandwidth: f64,
    #[live(true)]
    pub demo_data: bool,

    #[rust]
    fitted: bool,
}

impl ViolinPlot {
    // ---- Rust-side API (mirrors the 1.0 library surface) ----

    pub fn add_item(&mut self, item: ViolinItem) {
        self.items.push(item);
        self.fitted = false;
    }

    pub fn add_from_values(&mut self, label: impl Into<String>, values: &[f64]) {
        self.items.push(ViolinItem::new(label, values.to_vec()));
        self.fitted = false;
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.fitted = false;
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn set_show_box(&mut self, show: bool) {
        self.show_box = show;
    }

    pub fn set_show_median(&mut self, show: bool) {
        self.show_median = show;
    }

    // ---- Demo data ----

    fn make_demo_data(&mut self) {
        let mut rng = DemoRng::new(7);
        // Group 1: unimodal
        let mut g1 = Vec::with_capacity(60);
        for _ in 0..60 {
            let u = rng.next_f64() + rng.next_f64() + rng.next_f64() + rng.next_f64();
            g1.push(50.0 + (u - 2.0) * 8.0);
        }
        // Group 2: bimodal
        let mut g2 = Vec::with_capacity(60);
        for k in 0..60 {
            let u = rng.next_f64() + rng.next_f64() + rng.next_f64() + rng.next_f64();
            let center = if k % 2 == 0 { 40.0 } else { 62.0 };
            g2.push(center + (u - 2.0) * 5.0);
        }
        // Group 3: wide spread
        let mut g3 = Vec::with_capacity(60);
        for _ in 0..60 {
            let u = rng.next_f64() + rng.next_f64() + rng.next_f64() + rng.next_f64();
            g3.push(55.0 + (u - 2.0) * 14.0);
        }
        self.add_from_values("Alpha", &g1);
        self.add_from_values("Beta", &g2);
        self.add_from_values("Gamma", &g3);
    }

    // ---- Statistics ----

    fn value_range(&self) -> (f64, f64) {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for item in &self.items {
            for &v in &item.values {
                min = min.min(v);
                max = max.max(v);
            }
        }
        if !min.is_finite() || !max.is_finite() {
            return (0.0, 1.0);
        }
        if min == max {
            min -= 0.5;
            max += 0.5;
        }
        let padding = (max - min) * 0.1;
        (min - padding, max + padding)
    }

    /// Gaussian kernel density estimation, sampled at `n` points over [y_min, y_max]
    fn compute_kde(values: &[f64], bw: f64, y_min: f64, y_max: f64, n: usize) -> Vec<(f64, f64)> {
        let step = (y_max - y_min) / (n - 1) as f64;
        (0..n)
            .map(|i| {
                let y = y_min + i as f64 * step;
                let density: f64 = values
                    .iter()
                    .map(|&v| (-(y - v).powi(2) / (2.0 * bw * bw)).exp())
                    .sum();
                (
                    y,
                    density / (values.len() as f64 * bw * (2.0 * std::f64::consts::PI).sqrt()),
                )
            })
            .collect()
    }

    /// Bandwidth: explicit if set, else Silverman's rule over the pooled values
    fn effective_bandwidth(&self) -> f64 {
        if self.bandwidth > 0.0 {
            return self.bandwidth;
        }
        let all: Vec<f64> = self
            .items
            .iter()
            .flat_map(|i| i.values.iter().cloned())
            .collect();
        if all.is_empty() {
            return 1.0;
        }
        let mean = all.iter().sum::<f64>() / all.len() as f64;
        let std =
            (all.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / all.len() as f64).sqrt();
        let bw = 1.06 * std * (all.len() as f64).powf(-0.2);
        if bw > 0.0 {
            bw
        } else {
            1.0
        }
    }

    fn fit(&mut self) {
        let n = self.items.len().max(1);
        let (y_min, y_max) = self.value_range();
        self.plot_view.set_viewport(0.0, n as f64, y_min, y_max);
        self.fitted = true;
    }

    // ---- Drawing ----

    fn draw_violins(&mut self, _cx: &mut Cx2d) {
        let items = self.items.clone();
        if items.is_empty() {
            return;
        }
        let (y_min, y_max) = self.value_range();
        let bw = self.effective_bandwidth();

        // Band width in pixels (1.0 data unit along x)
        let (x0_px, _) = self.plot_view.data_to_px(0.0, y_min);
        let (x1_px, _) = self.plot_view.data_to_px(1.0, y_min);
        let band_px = (x1_px - x0_px).abs();
        let max_w_px = band_px * 0.4;

        for (i, item) in items.iter().enumerate() {
            if item.values.is_empty() {
                continue;
            }
            let xc = i as f64 + 0.5;
            let kde = Self::compute_kde(&item.values, bw, y_min, y_max, 50);
            let max_d = kde.iter().map(|(_, d)| *d).fold(0.0f64, f64::max);
            if max_d <= 0.0 {
                continue;
            }
            let color = item.color.unwrap_or_else(|| cycle_color(i));
            let fill_color = vec4(color.x, color.y, color.z, 0.6);

            // Build the violin outline: left side bottom→top, right side top→bottom
            let mut pts: Vec<(f32, f32)> = Vec::with_capacity(kde.len() * 2);
            for &(y, d) in &kde {
                let (cx_px, py) = self.plot_view.data_to_px(xc, y);
                let w = (d / max_d * max_w_px as f64) as f32;
                pts.push((cx_px - w, py));
            }
            for &(y, d) in kde.iter().rev() {
                let (cx_px, py) = self.plot_view.data_to_px(xc, y);
                let w = (d / max_d * max_w_px as f64) as f32;
                pts.push((cx_px + w, py));
            }
            self.plot_view.fill_polygon_px(&pts, fill_color);
            self.plot_view.stroke_polygon_px(&pts, color, 1.5);

            // Inner box + median marker
            if self.show_box || self.show_median {
                let mut s = item.values.clone();
                s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let q1 = s[s.len() / 4];
                let med = s[s.len() / 2];
                let q3 = s[(3 * s.len() / 4).min(s.len() - 1)];

                let (cx_px, q1_py) = self.plot_view.data_to_px(xc, q1);
                let (_, q3_py) = self.plot_view.data_to_px(xc, q3);
                let (_, med_py) = self.plot_view.data_to_px(xc, med);
                let box_hw = max_w_px * 0.15;

                if self.show_box {
                    self.plot_view.fill_rect_px(
                        cx_px - box_hw,
                        q3_py,
                        box_hw * 2.0,
                        (q1_py - q3_py).max(1.0),
                        vec4(0.3, 0.3, 0.3, 0.8),
                    );
                }
                if self.show_median {
                    let (half, mcolor) = if self.show_box {
                        (box_hw, vec4(1.0, 1.0, 1.0, 1.0))
                    } else {
                        (max_w_px * 0.3, vec4(0.15, 0.15, 0.15, 1.0))
                    };
                    self.plot_view.set_color(mcolor);
                    self.plot_view
                        .line_px(cx_px - half, med_py, cx_px + half, med_py, 2.0);
                }
            }
        }
    }
}

impl Widget for ViolinPlot {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.items.is_empty() && self.demo_data {
            self.make_demo_data();
        }
        if !self.fitted {
            self.fit();
        }

        self.plot_view.begin(cx, walk);

        draw_y_grid_and_ticks(&mut self.plot_view, cx);
        self.plot_view.draw_axes(cx);
        self.draw_violins(cx);

        let labels: Vec<String> = self.items.iter().map(|i| i.label.clone()).collect();
        draw_category_labels(&mut self.plot_view, cx, &labels);

        self.plot_view.end(cx);
        DrawStep::done()
    }

    fn script_call(
        &mut self,
        vm: &mut ScriptVm,
        method: LiveId,
        args: ScriptValue,
    ) -> ScriptAsyncResult {
        let mut handled = true;
        match method {
            x if x == live_id!(add_from_values) => {
                let label = script_arg_string(vm, &args, 0).unwrap_or_default();
                let values = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                self.add_from_values(label, &values);
            }
            x if x == live_id!(clear) => {
                self.clear();
            }
            x if x == live_id!(set_show_box) => {
                let show = script_arg_bool(vm, &args, 0).unwrap_or(true);
                self.set_show_box(show);
            }
            x if x == live_id!(set_show_median) => {
                let show = script_arg_bool(vm, &args, 0).unwrap_or(true);
                self.set_show_median(show);
            }
            x if x == live_id!(set_title) => {
                self.plot_view.title = script_arg_string(vm, &args, 0).unwrap_or_default();
            }
            _ => {
                handled = false;
            }
        }
        if handled {
            vm.with_cx_mut(|cx| {
                self.plot_view.redraw(cx);
            });
            return ScriptAsyncResult::Return(NIL);
        }
        ScriptAsyncResult::MethodNotFound
    }
}
