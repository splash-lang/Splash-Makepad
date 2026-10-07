// Scatter-family charts: ScatterPlot, BubbleChart, HexbinChart
//
// Ported from the Makepad 1.0 makepad-plot library to Makepad 2.0 / Octoscript.

use crate::plot_view::*;
use crate::script_util::*;
use crate::types::*;
use makepad_widgets::*;
use std::collections::HashMap;

script_mod! {
    use mod.prelude.widgets.*

    mod.plot.ScatterPlotBase = #(ScatterPlot::register_widget(vm))
    mod.plot.BubbleChartBase = #(BubbleChart::register_widget(vm))
    mod.plot.HexbinChartBase = #(HexbinChart::register_widget(vm))

    mod.plot.ScatterPlot = set_type_default() do mod.plot.ScatterPlotBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 52.0, top: 28.0, right: 16.0, bottom: 34.0}
        colormap: "Viridis"
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }

    mod.plot.BubbleChart = set_type_default() do mod.plot.BubbleChartBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 52.0, top: 28.0, right: 16.0, bottom: 34.0}
        colormap: "Viridis"
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }

    mod.plot.HexbinChart = set_type_default() do mod.plot.HexbinChartBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 52.0, top: 28.0, right: 16.0, bottom: 34.0}
        colormap: "Viridis"
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }
}

// =============================================================================
// ScatterPlot
// =============================================================================

#[derive(Script, ScriptHook, Widget)]
pub struct ScatterPlot {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub series: Vec<Series>,

    #[live(5.0)]
    pub point_radius: f64,
    #[live(false)]
    pub use_gradient: bool,
    #[live]
    pub colormap: String,
    #[live(true)]
    pub demo_data: bool,

    // Explicit axis limits (raw data space); None = auto-fit
    #[rust]
    x_lim: Option<(f64, f64)>,
    #[rust]
    y_lim: Option<(f64, f64)>,

    #[rust]
    fitted: bool,
}

impl ScatterPlot {
    // ---- Rust-side API (mirrors the 1.0 library surface) ----

    pub fn add_series(&mut self, series: Series) {
        self.series.push(series);
        self.fitted = false;
    }

    pub fn set_data(&mut self, xs: Vec<f64>, ys: Vec<f64>) {
        self.series.clear();
        self.series
            .push(Series::new("").with_data(xs, ys).with_marker(MarkerStyle::Circle));
        self.fitted = false;
    }

    pub fn clear(&mut self) {
        self.series.clear();
        self.fitted = false;
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn set_xlabel(&mut self, label: impl Into<String>) {
        self.plot_view.xlabel = label.into();
    }

    pub fn set_ylabel(&mut self, label: impl Into<String>) {
        self.plot_view.ylabel = label.into();
    }

    pub fn set_xlim(&mut self, min: f64, max: f64) {
        self.x_lim = Some((min, max));
        self.fitted = false;
    }

    pub fn set_ylim(&mut self, min: f64, max: f64) {
        self.y_lim = Some((min, max));
        self.fitted = false;
    }

    pub fn set_point_radius(&mut self, radius: f64) {
        self.point_radius = radius;
    }

    pub fn set_legend(&mut self, position: LegendPosition) {
        self.plot_view.legend = position;
    }

    pub fn set_use_gradient(&mut self, use_gradient: bool) {
        self.use_gradient = use_gradient;
    }

    pub fn set_interactive(&mut self, interactive: bool) {
        self.plot_view.interactive = interactive;
    }

    pub fn reset_view(&mut self) {
        self.fitted = false;
    }

    // ---- Demo data ----

    fn make_demo_data(&mut self) {
        let mut rng = DemoRng::new(0xC0FFEE);
        let clusters: [(f64, f64, f64, MarkerStyle, &str); 3] = [
            (2.5, 3.0, 1.0, MarkerStyle::Circle, "cluster a"),
            (6.0, 6.5, 1.3, MarkerStyle::TriangleUp, "cluster b"),
            (8.0, 2.5, 0.8, MarkerStyle::Diamond, "cluster c"),
        ];
        for (si, (cx0, cy0, spread, marker, label)) in clusters.into_iter().enumerate() {
            let mut xs = Vec::new();
            let mut ys = Vec::new();
            for _ in 0..40 {
                // Sum of uniforms approximates a gaussian
                let gx = rng.next_f64() + rng.next_f64() + rng.next_f64() + rng.next_f64() - 2.0;
                let gy = rng.next_f64() + rng.next_f64() + rng.next_f64() + rng.next_f64() - 2.0;
                xs.push(cx0 + gx * spread);
                ys.push(cy0 + gy * spread);
            }
            self.series.push(
                Series::new(label)
                    .with_data(xs, ys)
                    .with_color(cycle_color(si))
                    .with_marker(marker),
            );
        }
    }

    // ---- Fitting ----

    fn fit(&mut self) {
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        for s in &self.series {
            for &x in &s.x {
                x_min = x_min.min(x);
                x_max = x_max.max(x);
            }
            for &y in &s.y {
                y_min = y_min.min(y);
                y_max = y_max.max(y);
            }
        }
        if !x_min.is_finite() {
            x_min = 0.0;
            x_max = 1.0;
        }
        if !y_min.is_finite() {
            y_min = 0.0;
            y_max = 1.0;
        }
        if let Some((lo, hi)) = self.x_lim {
            x_min = lo;
            x_max = hi;
        }
        if let Some((lo, hi)) = self.y_lim {
            y_min = lo;
            y_max = hi;
        }
        if x_min == x_max {
            x_min -= 0.5;
            x_max += 0.5;
        }
        if y_min == y_max {
            y_min -= 0.5;
            y_max += 0.5;
        }
        self.plot_view.fit_data(x_min, x_max, y_min, y_max);
        self.fitted = true;
    }

    fn draw_points(&mut self, _cx: &mut Cx2d) {
        // Raw y range across all series, for the optional gradient along y
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        if self.use_gradient {
            for s in &self.series {
                for &y in &s.y {
                    y_min = y_min.min(y);
                    y_max = y_max.max(y);
                }
            }
        }
        let cmap = Colormap::from_name(&self.colormap);

        for si in 0..self.series.len() {
            let s = self.series[si].clone();
            let base_color = s.color.unwrap_or_else(|| cycle_color(si));
            let marker = if s.marker_style == MarkerStyle::None {
                MarkerStyle::Circle
            } else {
                s.marker_style
            };
            let radius = s.marker_size.unwrap_or(self.point_radius) as f32;
            let n = s.x.len().min(s.y.len());
            for i in 0..n {
                let color = if self.use_gradient && y_max > y_min {
                    cmap.sample((s.y[i] - y_min) / (y_max - y_min))
                } else {
                    base_color
                };
                self.plot_view
                    .draw_marker_data(s.x[i], s.y[i], radius, marker, color);
            }
        }
    }
}

impl Widget for ScatterPlot {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.series.is_empty() && self.demo_data {
            self.make_demo_data();
        }
        if !self.fitted {
            self.fit();
        }

        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx);
        self.draw_points(cx);

        // Legend
        let entries: Vec<(String, Vec4)> = self
            .series
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.label.is_empty())
            .map(|(i, s)| (s.label.clone(), s.color.unwrap_or_else(|| cycle_color(i))))
            .collect();
        self.plot_view.draw_legend(cx, &entries);

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
            x if x == live_id!(set_data) => {
                let xs = script_arg_f64_array(vm, &args, 0).unwrap_or_default();
                let ys = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                self.set_data(xs, ys);
            }
            x if x == live_id!(add_series) => {
                let label = script_arg_string(vm, &args, 0).unwrap_or_default();
                let xs = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                let ys = script_arg_f64_array(vm, &args, 2).unwrap_or_default();
                let idx = self.series.len();
                self.series.push(
                    Series::new(label)
                        .with_data(xs, ys)
                        .with_color(cycle_color(idx))
                        .with_marker(MarkerStyle::Circle),
                );
                self.fitted = false;
            }
            x if x == live_id!(set_point_radius) => {
                if let Some(r) = script_arg_f64(vm, &args, 0) {
                    self.set_point_radius(r);
                }
            }
            x if x == live_id!(set_use_gradient) => {
                self.set_use_gradient(script_arg_bool(vm, &args, 0).unwrap_or(true));
            }
            x if x == live_id!(clear) => {
                self.clear();
            }
            x if x == live_id!(set_title) => {
                self.plot_view.title = script_arg_string(vm, &args, 0).unwrap_or_default();
            }
            x if x == live_id!(set_xlabel) => {
                self.plot_view.xlabel = script_arg_string(vm, &args, 0).unwrap_or_default();
            }
            x if x == live_id!(set_ylabel) => {
                self.plot_view.ylabel = script_arg_string(vm, &args, 0).unwrap_or_default();
            }
            x if x == live_id!(set_interactive) => {
                self.plot_view.interactive = script_arg_bool(vm, &args, 0).unwrap_or(true);
            }
            x if x == live_id!(reset_view) => {
                self.fitted = false;
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
// BubbleChart
// =============================================================================

/// A single bubble: position + size encoding
#[derive(Clone, Debug, Default)]
pub struct BubblePoint {
    pub x: f64,
    pub y: f64,
    pub size: f64,
    pub color: Option<Vec4>,
    pub label: Option<String>,
}

impl BubblePoint {
    pub fn new(x: f64, y: f64, size: f64) -> Self {
        Self {
            x,
            y,
            size,
            color: None,
            label: None,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// A named series of bubbles
#[derive(Clone, Debug)]
pub struct BubbleSeries {
    pub name: String,
    pub points: Vec<BubblePoint>,
    pub color: Vec4,
}

impl BubbleSeries {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            points: Vec::new(),
            color: cycle_color(0),
        }
    }

    pub fn with_points(mut self, points: Vec<BubblePoint>) -> Self {
        self.points = points;
        self
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct BubbleChart {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub series: Vec<BubbleSeries>,

    #[live(5.0)]
    pub min_bubble_radius: f64,
    #[live(40.0)]
    pub max_bubble_radius: f64,
    #[live(false)]
    pub use_gradient: bool,
    #[live]
    pub colormap: String,
    #[live(true)]
    pub demo_data: bool,

    #[rust]
    fitted: bool,
}

impl BubbleChart {
    // ---- Rust-side API ----

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn add_series(&mut self, series: BubbleSeries) {
        self.series.push(series);
        self.fitted = false;
    }

    pub fn set_data(&mut self, xs: Vec<f64>, ys: Vec<f64>, sizes: Vec<f64>) {
        let n = xs.len().min(ys.len()).min(sizes.len());
        let points = (0..n)
            .map(|i| BubblePoint::new(xs[i], ys[i], sizes[i]))
            .collect();
        self.series.clear();
        self.series
            .push(BubbleSeries::new("").with_points(points).with_color(cycle_color(0)));
        self.fitted = false;
    }

    pub fn set_x_label(&mut self, label: impl Into<String>) {
        self.plot_view.xlabel = label.into();
    }

    pub fn set_y_label(&mut self, label: impl Into<String>) {
        self.plot_view.ylabel = label.into();
    }

    pub fn set_show_grid(&mut self, show: bool) {
        self.plot_view.show_grid = show;
    }

    pub fn set_bubble_radius_range(&mut self, min: f64, max: f64) {
        self.min_bubble_radius = min;
        self.max_bubble_radius = max;
    }

    pub fn set_use_gradient(&mut self, use_gradient: bool) {
        self.use_gradient = use_gradient;
    }

    pub fn clear(&mut self) {
        self.series.clear();
        self.use_gradient = false;
        self.fitted = false;
    }

    // ---- Demo data ----

    fn make_demo_data(&mut self) {
        let mut rng = DemoRng::new(0xB0BB1E);
        let mut points = Vec::new();
        for _ in 0..18 {
            let x = rng.next_f64() * 10.0;
            let y = rng.next_f64() * 10.0;
            let size = 5.0 + rng.next_f64() * 95.0;
            points.push(BubblePoint::new(x, y, size));
        }
        self.series.push(
            BubbleSeries::new("bubbles")
                .with_points(points)
                .with_color(cycle_color(0)),
        );
    }

    fn size_bounds(&self) -> (f64, f64) {
        let mut size_min = f64::INFINITY;
        let mut size_max = f64::NEG_INFINITY;
        for s in &self.series {
            for p in &s.points {
                size_min = size_min.min(p.size);
                size_max = size_max.max(p.size);
            }
        }
        if !size_min.is_finite() {
            (1.0, 1.0)
        } else {
            (size_min, size_max)
        }
    }

    fn fit(&mut self) {
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        for s in &self.series {
            for p in &s.points {
                x_min = x_min.min(p.x);
                x_max = x_max.max(p.x);
                y_min = y_min.min(p.y);
                y_max = y_max.max(p.y);
            }
        }
        if !x_min.is_finite() {
            x_min = 0.0;
            x_max = 1.0;
        }
        if !y_min.is_finite() {
            y_min = 0.0;
            y_max = 1.0;
        }
        if x_min == x_max {
            x_min -= 0.5;
            x_max += 0.5;
        }
        if y_min == y_max {
            y_min -= 0.5;
            y_max += 0.5;
        }
        self.plot_view.fit_data(x_min, x_max, y_min, y_max);
        self.fitted = true;
    }

    fn draw_bubbles(&mut self, cx: &mut Cx2d) {
        let (size_min, size_max) = self.size_bounds();
        let size_range = (size_max - size_min).max(0.001);
        let cmap = Colormap::from_name(&self.colormap);

        for si in 0..self.series.len() {
            let s = self.series[si].clone();
            for p in &s.points {
                let (px, py) = self.plot_view.data_to_px(p.x, p.y);
                let size_norm = ((p.size - size_min) / size_range).clamp(0.0, 1.0);
                let radius = (self.min_bubble_radius
                    + size_norm * (self.max_bubble_radius - self.min_bubble_radius))
                    as f32;

                let base = p.color.unwrap_or(s.color);
                let fill = if self.use_gradient {
                    let c = cmap.sample(size_norm);
                    vec4(c.x, c.y, c.z, 0.75)
                } else {
                    vec4(base.x, base.y, base.z, 0.6)
                };
                self.plot_view.fill_circle_px(px, py, radius, fill);

                // Outline
                let outline = if self.use_gradient {
                    cmap.sample(size_norm)
                } else {
                    base
                };
                self.plot_view.stroke_circle_px(px, py, radius, outline, 1.5);

                // Optional label above the bubble
                if let Some(label) = &p.label {
                    self.plot_view.draw_text_centered_px(
                        cx,
                        px as f64,
                        py as f64 - radius as f64 - 8.0,
                        label,
                        vec4(0.2, 0.2, 0.2, 1.0),
                        9.0,
                    );
                }
            }
        }
    }
}

impl Widget for BubbleChart {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.series.is_empty() && self.demo_data {
            self.make_demo_data();
        }
        if !self.fitted {
            self.fit();
        }

        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx);
        self.draw_bubbles(cx);

        // Legend
        let entries: Vec<(String, Vec4)> = self
            .series
            .iter()
            .filter(|s| !s.name.is_empty())
            .map(|s| (s.name.clone(), s.color))
            .collect();
        self.plot_view.draw_legend(cx, &entries);

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
            x if x == live_id!(set_data) => {
                let xs = script_arg_f64_array(vm, &args, 0).unwrap_or_default();
                let ys = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                let sizes = script_arg_f64_array(vm, &args, 2).unwrap_or_default();
                self.set_data(xs, ys, sizes);
            }
            x if x == live_id!(set_title) => {
                self.plot_view.title = script_arg_string(vm, &args, 0).unwrap_or_default();
            }
            x if x == live_id!(set_use_gradient) => {
                self.set_use_gradient(script_arg_bool(vm, &args, 0).unwrap_or(true));
            }
            x if x == live_id!(clear) => {
                self.clear();
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
// HexbinChart
// =============================================================================

/// A single raw data point for hexagonal binning
#[derive(Clone, Debug, Default)]
pub struct HexbinPoint {
    pub x: f64,
    pub y: f64,
}

impl HexbinPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct HexbinChart {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub points: Vec<HexbinPoint>,

    /// Approximate number of hexagons across the plot area
    #[live(16.0)]
    pub grid_size: f64,
    #[live]
    pub colormap: String,
    #[live(true)]
    pub demo_data: bool,

    #[rust]
    fitted: bool,
}

impl HexbinChart {
    // ---- Rust-side API ----

    pub fn set_data(&mut self, points: Vec<HexbinPoint>) {
        self.points = points;
        self.fitted = false;
    }

    pub fn set_data_xy(&mut self, xs: Vec<f64>, ys: Vec<f64>) {
        let n = xs.len().min(ys.len());
        self.points = (0..n).map(|i| HexbinPoint::new(xs[i], ys[i])).collect();
        self.fitted = false;
    }

    pub fn set_grid_size(&mut self, n: f64) {
        self.grid_size = n.max(2.0);
    }

    pub fn set_colormap(&mut self, name: impl Into<String>) {
        self.colormap = name.into();
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.fitted = false;
    }

    // ---- Demo data ----

    fn make_demo_data(&mut self) {
        let mut rng = DemoRng::new(0x4EBB1);
        let clusters: [(f64, f64, f64, usize); 2] =
            [(3.0, 3.5, 1.2, 400), (7.0, 6.5, 1.6, 300)];
        for (cx0, cy0, spread, count) in clusters {
            for _ in 0..count {
                let gx = rng.next_f64() + rng.next_f64() + rng.next_f64() + rng.next_f64() - 2.0;
                let gy = rng.next_f64() + rng.next_f64() + rng.next_f64() + rng.next_f64() - 2.0;
                self.points
                    .push(HexbinPoint::new(cx0 + gx * spread, cy0 + gy * spread));
            }
        }
    }

    // ---- Cube coordinate rounding (from the 1.0 implementation) ----

    fn cube_round(q: f64, r: f64) -> (i32, i32, i32) {
        let s = -q - r;
        let mut rq = q.round();
        let mut rr = r.round();
        let mut rs = s.round();

        let q_diff = (rq - q).abs();
        let r_diff = (rr - r).abs();
        let s_diff = (rs - s).abs();

        if q_diff > r_diff && q_diff > s_diff {
            rq = -rr - rs;
        } else if r_diff > s_diff {
            rr = -rq - rs;
        } else {
            rs = -rq - rr;
        }

        (rq as i32, rr as i32, rs as i32)
    }

    fn fit(&mut self) {
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        for p in &self.points {
            x_min = x_min.min(p.x);
            x_max = x_max.max(p.x);
            y_min = y_min.min(p.y);
            y_max = y_max.max(p.y);
        }
        if !x_min.is_finite() {
            x_min = 0.0;
            x_max = 1.0;
        }
        if !y_min.is_finite() {
            y_min = 0.0;
            y_max = 1.0;
        }
        if x_min == x_max {
            x_min -= 0.5;
            x_max += 0.5;
        }
        if y_min == y_max {
            y_min -= 0.5;
            y_max += 0.5;
        }
        self.plot_view.fit_data(x_min, x_max, y_min, y_max);
        self.fitted = true;
    }

    fn draw_hexes(&mut self, _cx: &mut Cx2d) {
        let pr = self.plot_view.plot_rect().clone();
        if pr.size.x <= 0.0 || pr.size.y <= 0.0 {
            return;
        }
        let sqrt3 = 3.0_f64.sqrt();
        let grid = self.grid_size.max(2.0);
        // Pointy-topped hexes: horizontal spacing is sqrt(3) * R, so `grid`
        // hexagons span the smaller plot dimension.
        let hex_r = (pr.size.x.min(pr.size.y) / (grid * sqrt3)).max(2.0);

        // Bin data points into hexagonal cube-coordinate cells (pixel space,
        // relative to the plot rect origin so pan/zoom re-bins consistently).
        let mut bins: HashMap<(i32, i32), usize> = HashMap::new();
        for p in &self.points {
            let (px, py) = self.plot_view.data_to_px(p.x, p.y);
            let x = px as f64 - pr.pos.x;
            let y = py as f64 - pr.pos.y;
            // Skip points outside the visible plot area
            if x < -hex_r || y < -hex_r || x > pr.size.x + hex_r || y > pr.size.y + hex_r {
                continue;
            }
            // Pixel -> fractional axial coordinates (pointy-topped)
            let q = (x * sqrt3 / 3.0 - y / 3.0) / hex_r;
            let r = (y * 2.0 / 3.0) / hex_r;
            let (q, r, _s) = Self::cube_round(q, r);
            *bins.entry((q, r)).or_insert(0) += 1;
        }

        if bins.is_empty() {
            return;
        }
        let max_count = bins.values().copied().max().unwrap_or(1).max(1);
        let cmap = Colormap::from_name(&self.colormap);
        let draw_r = hex_r * 0.94;

        for (&(q, r), &count) in &bins {
            // Axial -> pixel center (pointy-topped)
            let cx_px = pr.pos.x + hex_r * (sqrt3 * q as f64 + sqrt3 * 0.5 * r as f64);
            let cy_px = pr.pos.y + hex_r * 1.5 * r as f64;

            let t = count as f64 / max_count as f64;
            let color = cmap.sample(t);

            let pts: Vec<(f32, f32)> = (0..6)
                .map(|i| {
                    let a = std::f64::consts::PI / 3.0 * i as f64 + std::f64::consts::FRAC_PI_2;
                    (
                        (cx_px + draw_r * a.cos()) as f32,
                        (cy_px + draw_r * a.sin()) as f32,
                    )
                })
                .collect();
            self.plot_view.fill_polygon_px(&pts, color);
        }
    }
}

impl Widget for HexbinChart {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.points.is_empty() && self.demo_data {
            self.make_demo_data();
        }
        if !self.fitted {
            self.fit();
        }

        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx);
        self.draw_hexes(cx);
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
            x if x == live_id!(set_data) => {
                let xs = script_arg_f64_array(vm, &args, 0).unwrap_or_default();
                let ys = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                self.set_data_xy(xs, ys);
            }
            x if x == live_id!(set_grid_size) => {
                if let Some(n) = script_arg_f64(vm, &args, 0) {
                    self.set_grid_size(n);
                }
            }
            x if x == live_id!(set_colormap) => {
                self.colormap = script_arg_string(vm, &args, 0).unwrap_or_default();
            }
            x if x == live_id!(set_title) => {
                self.plot_view.title = script_arg_string(vm, &args, 0).unwrap_or_default();
            }
            x if x == live_id!(clear) => {
                self.clear();
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
