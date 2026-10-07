// Line-family charts (part 2): StemPlot, AreaChart, StepPlot
//
// Ported from the Makepad 1.0 makepad-plot library to Makepad 2.0 / Octoscript,
// following the LinePlot exemplar in charts/line.rs.

use crate::charts::line::step_points;
use crate::plot_view::*;
use crate::script_util::*;
use crate::types::*;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*

    mod.plot.StemPlotBase = #(StemPlot::register_widget(vm))

    mod.plot.StemPlot = set_type_default() do mod.plot.StemPlotBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 52.0, top: 28.0, right: 16.0, bottom: 34.0}
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }

    mod.plot.AreaChartBase = #(AreaChart::register_widget(vm))

    mod.plot.AreaChart = set_type_default() do mod.plot.AreaChartBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 52.0, top: 28.0, right: 16.0, bottom: 34.0}
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }

    mod.plot.StepPlotBase = #(StepPlot::register_widget(vm))

    mod.plot.StepPlot = set_type_default() do mod.plot.StepPlotBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 52.0, top: 28.0, right: 16.0, bottom: 34.0}
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }
}

// =============================================================================
// StemPlot — lollipop chart: baseline, vertical stems, markers on top
// =============================================================================

#[derive(Script, ScriptHook, Widget)]
pub struct StemPlot {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub series: Vec<Series>,

    // Explicit axis limits (raw data space); None = auto-fit
    #[rust]
    x_lim: Option<(f64, f64)>,
    #[rust]
    y_lim: Option<(f64, f64)>,

    #[live(0.0)]
    pub baseline: f64,
    #[live(4.0)]
    pub marker_size: f64,
    #[live(1.5)]
    pub stem_width: f64,
    #[live(true)]
    pub demo_data: bool,

    #[rust]
    fitted: bool,
}

impl StemPlot {
    // ---- Rust-side API (mirrors the 1.0 library surface) ----

    pub fn add_series(&mut self, series: Series) {
        self.series.push(series);
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

    pub fn set_baseline(&mut self, baseline: f64) {
        self.baseline = baseline;
        self.fitted = false;
    }

    pub fn set_marker_size(&mut self, size: f64) {
        self.marker_size = size;
    }

    pub fn set_stem_width(&mut self, width: f64) {
        self.stem_width = width;
    }

    pub fn set_legend(&mut self, position: LegendPosition) {
        self.plot_view.legend = position;
    }

    // ---- Demo data ----

    fn make_demo_data(&mut self) {
        let mut rng = DemoRng::new(11);
        let n = 24;
        let xs: Vec<f64> = (0..n).map(|i| i as f64 * 0.5).collect();
        let ys: Vec<f64> = xs
            .iter()
            .map(|&x| (x * 0.9).sin() / (1.0 + 0.12 * x) + (rng.next_f64() - 0.5) * 0.12)
            .collect();
        self.series
            .push(Series::new("").with_data(xs, ys).with_color(cycle_color(0)));
    }

    // ---- Fitting ----

    fn fit(&mut self) {
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        // The baseline is always part of the y range (old auto_range behavior)
        let mut y_min = self.baseline;
        let mut y_max = self.baseline;
        let mut any = false;
        for s in &self.series {
            for &x in &s.x {
                x_min = x_min.min(x);
                x_max = x_max.max(x);
                any = true;
            }
            for &y in &s.y {
                y_min = y_min.min(y);
                y_max = y_max.max(y);
            }
        }
        if !any || !x_min.is_finite() {
            x_min = 0.0;
            x_max = 1.0;
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

    fn draw_stems(&mut self, _cx: &mut Cx2d) {
        let baseline = self.baseline;
        for si in 0..self.series.len() {
            let s = self.series[si].clone();
            let color = s.color.unwrap_or_else(|| cycle_color(si));
            let marker = if s.marker_style != MarkerStyle::None {
                s.marker_style
            } else {
                MarkerStyle::Circle
            };
            let msize = s.marker_size.unwrap_or(self.marker_size) as f32;
            let stem_width = s.line_width.unwrap_or(self.stem_width) as f32;

            let n = s.x.len().min(s.y.len());
            for i in 0..n {
                let x = s.x[i];
                let y = s.y[i];
                // Stem: vertical line from baseline to the point
                self.plot_view
                    .draw_line_data(x, baseline, x, y, color, stem_width, s.line_style);
                // Marker on top
                self.plot_view.draw_marker_data(x, y, msize, marker, color);
            }
        }
    }
}

impl Widget for StemPlot {
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

        // Baseline (dashed) if it is inside the visible y range
        let baseline = self.baseline;
        let vp = self.plot_view.viewport.clone();
        let tb = self.plot_view.y_scale.transform(baseline);
        if tb > vp.y_min && tb < vp.y_max {
            self.plot_view
                .draw_hline(baseline, vec4(0.5, 0.5, 0.5, 0.7), 1.0, LineStyle::Dashed);
        }

        self.draw_stems(cx);

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
                self.series.clear();
                self.series
                    .push(Series::new("").with_data(xs, ys).with_color(cycle_color(0)));
                self.fitted = false;
            }
            x if x == live_id!(add_series) => {
                let label = script_arg_string(vm, &args, 0).unwrap_or_default();
                let xs = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                let ys = script_arg_f64_array(vm, &args, 2).unwrap_or_default();
                let idx = self.series.len();
                self.series
                    .push(Series::new(label).with_data(xs, ys).with_color(cycle_color(idx)));
                self.fitted = false;
            }
            x if x == live_id!(set_baseline) => {
                if let Some(v) = script_arg_f64(vm, &args, 0) {
                    self.set_baseline(v);
                }
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
// AreaChart — filled area under curves, multiple (optionally stacked) series
// =============================================================================

/// One series of an AreaChart
#[derive(Clone, Debug)]
pub struct AreaSeries {
    pub name: String,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub color: Vec4,
}

impl AreaSeries {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            x: Vec::new(),
            y: Vec::new(),
            color: cycle_color(0),
        }
    }

    pub fn with_data(mut self, x: Vec<f64>, y: Vec<f64>) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct AreaChart {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub series: Vec<AreaSeries>,

    #[live(false)]
    pub stacked: bool,
    #[live(0.45)]
    pub fill_alpha: f64,
    #[live(2.0)]
    pub line_width: f64,
    #[live(true)]
    pub demo_data: bool,

    #[rust]
    fitted: bool,
}

impl AreaChart {
    // ---- Rust-side API (mirrors the 1.0 library surface) ----

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn add_series(&mut self, series: AreaSeries) {
        self.series.push(series);
        self.fitted = false;
    }

    pub fn set_x_label(&mut self, label: impl Into<String>) {
        self.plot_view.xlabel = label.into();
    }

    pub fn set_y_label(&mut self, label: impl Into<String>) {
        self.plot_view.ylabel = label.into();
    }

    pub fn set_stacked(&mut self, stacked: bool) {
        self.stacked = stacked;
        self.fitted = false;
    }

    pub fn set_show_grid(&mut self, show: bool) {
        self.plot_view.show_grid = show;
    }

    pub fn clear(&mut self) {
        self.series.clear();
        self.fitted = false;
    }

    // ---- Bounds (ported from the 1.0 get_bounds) ----

    fn get_bounds(&self) -> (f64, f64, f64, f64) {
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let y_min = 0.0f64;
        let mut y_max = f64::NEG_INFINITY;

        if self.stacked {
            // For stacked, compute cumulative max
            if let Some(first) = self.series.first() {
                let n = first.x.len();
                for i in 0..n {
                    let mut sum = 0.0;
                    for s in &self.series {
                        if i < s.y.len() {
                            sum += s.y[i];
                        }
                    }
                    y_max = y_max.max(sum);
                }
            }
        }

        for s in &self.series {
            for &x in &s.x {
                x_min = x_min.min(x);
                x_max = x_max.max(x);
            }
            if !self.stacked {
                for &y in &s.y {
                    y_max = y_max.max(y);
                }
            }
        }

        if !x_min.is_finite() {
            x_min = 0.0;
            x_max = 1.0;
        }
        if !y_max.is_finite() {
            y_max = 1.0;
        }
        (x_min, x_max, y_min, y_max)
    }

    fn fit(&mut self) {
        let (mut x_min, mut x_max, mut y_min, mut y_max) = self.get_bounds();
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

    // ---- Demo data ----

    fn make_demo_data(&mut self) {
        let mut rng = DemoRng::new(23);
        let n = 40;
        let xs: Vec<f64> = (0..n).map(|i| i as f64 * 0.5).collect();
        let a: Vec<f64> = xs
            .iter()
            .map(|&x| 2.2 + (x * 0.35).sin() * 0.9 + (rng.next_f64() - 0.5) * 0.25)
            .collect();
        let b: Vec<f64> = xs
            .iter()
            .map(|&x| 1.4 + (x * 0.22 + 1.0).cos() * 0.6 + (rng.next_f64() - 0.5) * 0.2)
            .collect();
        self.series.push(
            AreaSeries::new("alpha")
                .with_data(xs.clone(), a)
                .with_color(cycle_color(0)),
        );
        self.series
            .push(AreaSeries::new("beta").with_data(xs, b).with_color(cycle_color(1)));
    }

    fn draw_areas(&mut self, _cx: &mut Cx2d) {
        let stacked = self.stacked;
        let fill_alpha = self.fill_alpha as f32;
        let line_width = self.line_width as f32;
        let mut cumulative: Vec<f64> =
            vec![0.0; self.series.first().map(|s| s.x.len()).unwrap_or(0)];

        for si in 0..self.series.len() {
            let s = self.series[si].clone();
            let n = s.x.len().min(s.y.len());
            if n < 2 {
                continue;
            }

            // Top curve (stacked on the cumulative sum if requested)
            let top: Vec<f64> = (0..n)
                .map(|i| {
                    if stacked {
                        s.y[i] + cumulative.get(i).copied().unwrap_or(0.0)
                    } else {
                        s.y[i]
                    }
                })
                .collect();
            // Base curve: previous cumulative for stacked, y = 0 otherwise
            let base: Vec<f64> = (0..n)
                .map(|i| {
                    if stacked {
                        cumulative.get(i).copied().unwrap_or(0.0)
                    } else {
                        0.0
                    }
                })
                .collect();

            // Filled polygon: top curve forward, base curve backward.
            // (Replaces the old per-strip gradient fill with a solid
            // translucent fill at the gradient's midpoint alpha.)
            let mut poly_x = Vec::with_capacity(n * 2);
            let mut poly_y = Vec::with_capacity(n * 2);
            for i in 0..n {
                poly_x.push(s.x[i]);
                poly_y.push(top[i]);
            }
            for i in (0..n).rev() {
                poly_x.push(s.x[i]);
                poly_y.push(base[i]);
            }
            let fill = vec4(s.color.x, s.color.y, s.color.z, s.color.w * fill_alpha);
            self.plot_view.fill_polygon_data(&poly_x, &poly_y, fill);

            // Top edge line in the solid series color
            self.plot_view
                .draw_polyline_data(&s.x[0..n], &top, s.color, line_width, LineStyle::Solid);

            // Update cumulative for stacked mode
            if stacked {
                for i in 0..cumulative.len().min(n) {
                    cumulative[i] += s.y[i];
                }
            }
        }
    }
}

impl Widget for AreaChart {
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
        self.draw_areas(cx);

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
                self.series.clear();
                self.series.push(
                    AreaSeries::new("")
                        .with_data(xs, ys)
                        .with_color(cycle_color(0)),
                );
                self.fitted = false;
            }
            x if x == live_id!(add_series) => {
                let label = script_arg_string(vm, &args, 0).unwrap_or_default();
                let xs = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                let ys = script_arg_f64_array(vm, &args, 2).unwrap_or_default();
                let idx = self.series.len();
                self.series.push(
                    AreaSeries::new(label)
                        .with_data(xs, ys)
                        .with_color(cycle_color(idx)),
                );
                self.fitted = false;
            }
            x if x == live_id!(clear) => {
                self.clear();
            }
            x if x == live_id!(set_title) => {
                self.plot_view.title = script_arg_string(vm, &args, 0).unwrap_or_default();
            }
            x if x == live_id!(set_stacked) => {
                self.set_stacked(script_arg_bool(vm, &args, 0).unwrap_or(false));
            }
            x if x == live_id!(set_xlabel) => {
                self.plot_view.xlabel = script_arg_string(vm, &args, 0).unwrap_or_default();
            }
            x if x == live_id!(set_ylabel) => {
                self.plot_view.ylabel = script_arg_string(vm, &args, 0).unwrap_or_default();
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
// StepPlot — discrete step-wise line visualization
// =============================================================================

/// One series of a StepPlot
#[derive(Clone, Debug)]
pub struct StepSeries {
    pub name: String,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub color: Vec4,
    pub style: StepStyle,
}

impl StepSeries {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            x: Vec::new(),
            y: Vec::new(),
            color: cycle_color(0),
            style: StepStyle::Pre,
        }
    }

    pub fn with_data(mut self, x: Vec<f64>, y: Vec<f64>) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn with_style(mut self, style: StepStyle) -> Self {
        self.style = style;
        self
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct StepPlot {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub series: Vec<StepSeries>,

    /// Default step mode for series created via `set_data` / demo data.
    /// `None` falls back to the old widget's default, `Pre`.
    #[live]
    pub step_style: StepStyle,
    #[live(2.0)]
    pub line_width: f64,
    #[live(false)]
    pub show_markers: bool,
    #[live(4.0)]
    pub marker_size: f64,
    #[live(true)]
    pub demo_data: bool,

    #[rust]
    fitted: bool,
}

impl StepPlot {
    // ---- Rust-side API (mirrors the 1.0 library surface) ----

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn add_series(&mut self, series: StepSeries) {
        self.series.push(series);
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

    pub fn set_show_markers(&mut self, show: bool) {
        self.show_markers = show;
    }

    pub fn clear(&mut self) {
        self.series.clear();
        self.fitted = false;
    }

    /// The step mode used for set_data / demo series
    fn default_step_style(&self) -> StepStyle {
        if self.step_style == StepStyle::None {
            StepStyle::Pre
        } else {
            self.step_style
        }
    }

    // ---- Demo data ----

    fn make_demo_data(&mut self) {
        let mut rng = DemoRng::new(41);
        let n = 14;
        let xs: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let mut level = 3.0;
        let ys: Vec<f64> = (0..n)
            .map(|_| {
                level += (rng.next_f64() - 0.45) * 1.6;
                level
            })
            .collect();
        let style = self.default_step_style();
        self.series.push(
            StepSeries::new("")
                .with_data(xs, ys)
                .with_color(cycle_color(0))
                .with_style(style),
        );
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

    fn draw_steps(&mut self, _cx: &mut Cx2d) {
        let line_width = self.line_width as f32;
        let show_markers = self.show_markers;
        let msize = self.marker_size as f32;
        for si in 0..self.series.len() {
            let s = self.series[si].clone();
            let n = s.x.len().min(s.y.len());
            if n < 2 {
                continue;
            }

            match s.style {
                StepStyle::None => {
                    self.plot_view.draw_polyline_data(
                        &s.x[0..n],
                        &s.y[0..n],
                        s.color,
                        line_width,
                        LineStyle::Solid,
                    );
                }
                _ => {
                    let (sx, sy) = step_points(&s.x[0..n], &s.y[0..n], s.style);
                    self.plot_view
                        .draw_polyline_data(&sx, &sy, s.color, line_width, LineStyle::Solid);
                }
            }

            if show_markers {
                for i in 0..n {
                    self.plot_view
                        .draw_marker_data(s.x[i], s.y[i], msize, MarkerStyle::Circle, s.color);
                }
            }
        }
    }
}

impl Widget for StepPlot {
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
        self.draw_steps(cx);
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
                let style = self.default_step_style();
                self.series.clear();
                self.series.push(
                    StepSeries::new("")
                        .with_data(xs, ys)
                        .with_color(cycle_color(0))
                        .with_style(style),
                );
                self.fitted = false;
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
