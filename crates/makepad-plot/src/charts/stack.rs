// Stacked / layered area charts + dual-axis line plot:
// LinePlotDual, Stackplot (StackSeries/StackedPoint/StackOrder/StackOffset),
// Streamgraph (StreamSeries).
//
// Ported from the Makepad 1.0 plot library to Makepad 2.0 / Octoscript.

use crate::plot_view::*;
use crate::script_util::*;
use crate::types::*;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*

    mod.plot.LinePlotDualBase = #(LinePlotDual::register_widget(vm))

    mod.plot.LinePlotDual = set_type_default() do mod.plot.LinePlotDualBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 52.0, top: 28.0, right: 52.0, bottom: 34.0}
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }

    mod.plot.StackplotBase = #(Stackplot::register_widget(vm))

    mod.plot.Stackplot = set_type_default() do mod.plot.StackplotBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 52.0, top: 28.0, right: 16.0, bottom: 34.0}
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }

    mod.plot.StreamgraphBase = #(Streamgraph::register_widget(vm))

    mod.plot.Streamgraph = set_type_default() do mod.plot.StreamgraphBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 52.0, top: 28.0, right: 16.0, bottom: 34.0}
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }
}

fn darken(c: Vec4, amount: f32) -> Vec4 {
    vec4(
        c.x * (1.0 - amount),
        c.y * (1.0 - amount),
        c.z * (1.0 - amount),
        c.w,
    )
}

// =============================================================================
// LinePlotDual — line plot with independent left and right y-axes
// =============================================================================

#[derive(Script, ScriptHook, Widget)]
pub struct LinePlotDual {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub series_left: Vec<Series>,
    #[rust]
    pub series_right: Vec<Series>,
    #[rust]
    pub y2label: String,

    // Explicit limits (raw data space); None = auto-fit
    #[rust]
    x_lim: Option<(f64, f64)>,
    #[rust]
    y_lim: Option<(f64, f64)>,
    #[rust]
    y2_lim: Option<(f64, f64)>,
    // Right axis data range (padded), computed in fit()
    #[rust]
    y2_range: (f64, f64),

    #[live(2.0)]
    pub line_width: f64,
    #[live(true)]
    pub demo_data: bool,

    #[rust]
    fitted: bool,
}

impl LinePlotDual {
    // ---- Rust-side API (mirrors the 1.0 library surface) ----

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn set_xlabel(&mut self, label: impl Into<String>) {
        self.plot_view.xlabel = label.into();
    }

    pub fn set_ylabel(&mut self, label: impl Into<String>) {
        self.plot_view.ylabel = label.into();
    }

    pub fn set_y2label(&mut self, label: impl Into<String>) {
        self.y2label = label.into();
    }

    pub fn add_series_left(&mut self, series: Series) {
        self.series_left.push(series);
        self.fitted = false;
    }

    pub fn add_series_right(&mut self, series: Series) {
        self.series_right.push(series);
        self.fitted = false;
    }

    pub fn set_xlim(&mut self, min: f64, max: f64) {
        self.x_lim = Some((min, max));
        self.fitted = false;
    }

    pub fn set_ylim(&mut self, min: f64, max: f64) {
        self.y_lim = Some((min, max));
        self.fitted = false;
    }

    pub fn set_y2lim(&mut self, min: f64, max: f64) {
        self.y2_lim = Some((min, max));
        self.fitted = false;
    }

    pub fn set_grid(&mut self, show: bool) {
        self.plot_view.show_grid = show;
    }

    pub fn set_legend(&mut self, pos: LegendPosition) {
        self.plot_view.legend = pos;
    }

    pub fn clear(&mut self) {
        self.series_left.clear();
        self.series_right.clear();
        self.fitted = false;
    }

    // ---- Demo data ----

    fn make_demo_data(&mut self) {
        let n = 60;
        let xs: Vec<f64> = (0..n).map(|i| i as f64 * 0.2).collect();
        let temp: Vec<f64> = xs.iter().map(|x| 20.0 + 8.0 * (x * 0.6).sin()).collect();
        let humid: Vec<f64> = xs
            .iter()
            .map(|x| 0.55 + 0.35 * (x * 0.45 + 1.2).cos())
            .collect();
        self.series_left.push(
            Series::new("temperature")
                .with_data(xs.clone(), temp)
                .with_color(cycle_color(0)),
        );
        self.series_right.push(
            Series::new("humidity")
                .with_data(xs, humid)
                .with_color(cycle_color(1)),
        );
        if self.plot_view.ylabel.is_empty() {
            self.plot_view.ylabel = "temp".to_string();
        }
        if self.y2label.is_empty() {
            self.y2label = "humidity".to_string();
        }
    }

    // ---- Fitting ----

    fn fit(&mut self) {
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        for s in self.series_left.iter().chain(self.series_right.iter()) {
            for &x in &s.x {
                x_min = x_min.min(x);
                x_max = x_max.max(x);
            }
        }
        for s in &self.series_left {
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

        // Right axis range (padded 5% like fit_data so both axes align)
        let mut y2_min = f64::INFINITY;
        let mut y2_max = f64::NEG_INFINITY;
        for s in &self.series_right {
            for &y in &s.y {
                y2_min = y2_min.min(y);
                y2_max = y2_max.max(y);
            }
        }
        if !y2_min.is_finite() {
            y2_min = 0.0;
            y2_max = 1.0;
        }
        if let Some((lo, hi)) = self.y2_lim {
            y2_min = lo;
            y2_max = hi;
        }
        if y2_min == y2_max {
            y2_min -= 0.5;
            y2_max += 0.5;
        }
        let pad = (y2_max - y2_min).abs().max(1e-9) * 0.05;
        self.y2_range = (y2_min - pad, y2_max + pad);
        self.fitted = true;
    }

    /// Map a right-axis data y to a (transformed) viewport y, so right series
    /// track pan/zoom of the shared viewport.
    fn right_to_ty(&self, y: f64) -> f64 {
        let (lo, hi) = self.y2_range;
        let span = hi - lo;
        let span = if span.abs() < 1e-12 { 1.0 } else { span };
        let vp = &self.plot_view.viewport;
        vp.y_min + (y - lo) / span * vp.y_range()
    }

    fn draw_right_series(&mut self, _cx: &mut Cx2d) {
        for si in 0..self.series_right.len() {
            let s = self.series_right[si].clone();
            let color = s
                .color
                .unwrap_or_else(|| cycle_color(si + self.series_left.len()));
            let width = s.line_width.unwrap_or(self.line_width) as f32;
            let n = s.x.len().min(s.y.len());
            if n < 2 {
                continue;
            }
            self.plot_view.set_color(color);
            let mut prev: Option<(f32, f32)> = None;
            for i in 0..n {
                let tx = self.plot_view.x_scale.transform(s.x[i]);
                let ty = self.right_to_ty(s.y[i]);
                let (px, py) = self.plot_view.tdata_to_px(tx, ty);
                if let Some((ppx, ppy)) = prev {
                    self.plot_view
                        .line_styled_px(ppx, ppy, px, py, width, s.line_style);
                }
                prev = Some((px, py));
            }
        }
    }

    fn draw_right_axis(&mut self, cx: &mut Cx2d) {
        let (lo, hi) = self.y2_range;
        if !(hi > lo) {
            return;
        }
        let ticks = nice_ticks(lo, hi, 6);
        let pr = self.plot_view.plot_rect;
        let text_color = vec4(0.2, 0.2, 0.2, 1.0);
        let font_size = self.plot_view.tick_font_size;
        for &v in &ticks {
            let ty = self.right_to_ty(v);
            let (_, py) = self.plot_view.tdata_to_px(0.0, ty);
            if (py as f64) < pr.pos.y - 0.5 || (py as f64) > pr.pos.y + pr.size.y + 0.5 {
                continue;
            }
            // small tick mark on the right edge
            let edge = (pr.pos.x + pr.size.x) as f32;
            self.plot_view.set_color(vec4(0.6, 0.6, 0.6, 1.0));
            self.plot_view.line_px(edge, py, edge + 4.0, py, 1.0);
            let label = format_tick_value(ScaleType::Linear, v);
            self.plot_view.draw_text_px(
                cx,
                pr.pos.x + pr.size.x + 7.0,
                py as f64 - font_size as f64 * 0.6,
                &label,
                text_color,
                font_size,
            );
        }
        // Right y-axis label (top-right, horizontal)
        if !self.y2label.is_empty() {
            let y2label = self.y2label.clone();
            let font_size = self.plot_view.label_font_size;
            let est_w = y2label.len() as f64 * font_size as f64 * 0.5;
            let rect = self.plot_view.rect;
            self.plot_view.draw_text_px(
                cx,
                rect.pos.x + rect.size.x - est_w - 4.0,
                rect.pos.y + 4.0,
                &y2label,
                text_color,
                font_size,
            );
        }
    }
}

impl Widget for LinePlotDual {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.series_left.is_empty() && self.series_right.is_empty() && self.demo_data {
            self.make_demo_data();
            self.fitted = false;
        }
        if !self.fitted {
            self.fit();
        }

        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx);
        self.draw_right_axis(cx);

        // Left series (use the shared viewport directly)
        for si in 0..self.series_left.len() {
            let s = self.series_left[si].clone();
            let color = s.color.unwrap_or_else(|| cycle_color(si));
            let width = s.line_width.unwrap_or(self.line_width) as f32;
            self.plot_view
                .draw_polyline_data(&s.x, &s.y, color, width, s.line_style);
        }

        self.draw_right_series(cx);

        // Legend covers both axes
        let mut entries: Vec<(String, Vec4)> = Vec::new();
        for (i, s) in self.series_left.iter().enumerate() {
            if !s.label.is_empty() {
                entries.push((s.label.clone(), s.color.unwrap_or_else(|| cycle_color(i))));
            }
        }
        let n_left = self.series_left.len();
        for (i, s) in self.series_right.iter().enumerate() {
            if !s.label.is_empty() {
                entries.push((
                    s.label.clone(),
                    s.color.unwrap_or_else(|| cycle_color(i + n_left)),
                ));
            }
        }
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
            x if x == live_id!(set_data_left) => {
                let xs = script_arg_f64_array(vm, &args, 0).unwrap_or_default();
                let ys = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                self.series_left.clear();
                self.series_left
                    .push(Series::new("").with_data(xs, ys).with_color(cycle_color(0)));
                self.fitted = false;
            }
            x if x == live_id!(set_data_right) => {
                let xs = script_arg_f64_array(vm, &args, 0).unwrap_or_default();
                let ys = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                self.series_right.clear();
                self.series_right
                    .push(Series::new("").with_data(xs, ys).with_color(cycle_color(1)));
                self.fitted = false;
            }
            x if x == live_id!(set_title) => {
                self.plot_view.title = script_arg_string(vm, &args, 0).unwrap_or_default();
            }
            x if x == live_id!(set_xlabel) => {
                self.plot_view.xlabel = script_arg_string(vm, &args, 0).unwrap_or_default();
            }
            x if x == live_id!(set_ylabel_left) => {
                self.plot_view.ylabel = script_arg_string(vm, &args, 0).unwrap_or_default();
            }
            x if x == live_id!(set_ylabel_right) => {
                self.y2label = script_arg_string(vm, &args, 0).unwrap_or_default();
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
// Stackplot — stacked area chart with ordering / offset (wiggle, silhouette…)
// =============================================================================

/// Stack ordering method
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StackOrder {
    /// No reordering, maintain original series order
    #[default]
    None,
    /// Sort by sum of values ascending
    Ascending,
    /// Sort by sum of values descending
    Descending,
    /// Sort so smallest series are in the middle
    InsideOut,
    /// Reverse the current order
    Reverse,
}

impl StackOrder {
    pub fn from_name(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "ascending" => StackOrder::Ascending,
            "descending" => StackOrder::Descending,
            "insideout" | "inside_out" => StackOrder::InsideOut,
            "reverse" => StackOrder::Reverse,
            _ => StackOrder::None,
        }
    }
}

/// Stack offset method
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StackOffset {
    /// No offset, stack from zero
    #[default]
    None,
    /// Normalize to fill [0, 1] range
    Expand,
    /// Center around zero (diverging stacks)
    Diverging,
    /// Center the baseline (silhouette)
    Silhouette,
    /// Streamgraph wiggle minimization
    Wiggle,
}

impl StackOffset {
    pub fn from_name(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "expand" => StackOffset::Expand,
            "diverging" => StackOffset::Diverging,
            "silhouette" => StackOffset::Silhouette,
            "wiggle" => StackOffset::Wiggle,
            _ => StackOffset::None,
        }
    }
}

/// A single series for the stackplot
#[derive(Clone, Debug)]
pub struct StackSeries {
    pub label: String,
    pub values: Vec<f64>,
    pub color: Option<Vec4>,
}

impl StackSeries {
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

/// Stacked point with y0 (bottom) and y1 (top) bounds
#[derive(Clone, Debug)]
pub struct StackedPoint {
    pub y0: f64,
    pub y1: f64,
}

impl StackedPoint {
    pub fn new(y0: f64, y1: f64) -> Self {
        Self { y0, y1 }
    }

    pub fn height(&self) -> f64 {
        self.y1 - self.y0
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct Stackplot {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub series: Vec<StackSeries>,
    #[rust]
    pub x: Vec<f64>,
    #[rust]
    pub order: StackOrder,
    #[rust]
    pub offset: StackOffset,

    #[live(true)]
    pub show_lines: bool,
    #[live(true)]
    pub demo_data: bool,

    #[rust]
    fitted: bool,
}

impl Stackplot {
    // ---- Rust-side API (mirrors the 1.0 library surface) ----

    pub fn set_data(&mut self, series: Vec<StackSeries>, x: Vec<f64>) {
        self.series = series;
        self.x = x;
        self.fitted = false;
    }

    pub fn add_series(&mut self, series: StackSeries) {
        self.series.push(series);
        self.fitted = false;
    }

    pub fn set_x(&mut self, x: Vec<f64>) {
        self.x = x;
        self.fitted = false;
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn set_order(&mut self, order: StackOrder) {
        self.order = order;
        self.fitted = false;
    }

    pub fn set_offset(&mut self, offset: StackOffset) {
        self.offset = offset;
        self.fitted = false;
    }

    pub fn set_show_lines(&mut self, show: bool) {
        self.show_lines = show;
    }

    pub fn clear(&mut self) {
        self.series.clear();
        self.x.clear();
        self.fitted = false;
    }

    // ---- Stacking (ported 1:1 from the 1.0 widget) ----

    fn compute_stacked(&self) -> Vec<Vec<StackedPoint>> {
        let n_series = self.series.len();
        if n_series == 0 {
            return vec![];
        }
        let n_points = self.series.iter().map(|s| s.values.len()).max().unwrap_or(0);
        if n_points == 0 {
            return vec![];
        }

        let mut result: Vec<Vec<StackedPoint>> = self
            .series
            .iter()
            .map(|_| vec![StackedPoint::new(0.0, 0.0); n_points])
            .collect();

        let order = self.compute_order();

        for i in 0..n_points {
            let mut y0 = 0.0;
            for &series_idx in &order {
                let y = self.series[series_idx].values.get(i).copied().unwrap_or(0.0);
                result[series_idx][i] = StackedPoint::new(y0, y0 + y);
                y0 += y;
            }
        }

        self.apply_offset(&mut result, n_points);
        result
    }

    fn compute_order(&self) -> Vec<usize> {
        let n = self.series.len();
        let mut indices: Vec<usize> = (0..n).collect();

        match self.order {
            StackOrder::None => {}
            StackOrder::Ascending => {
                let sums: Vec<f64> = self.series.iter().map(|s| s.values.iter().sum()).collect();
                indices.sort_by(|&a, &b| {
                    sums[a]
                        .partial_cmp(&sums[b])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            StackOrder::Descending => {
                let sums: Vec<f64> = self.series.iter().map(|s| s.values.iter().sum()).collect();
                indices.sort_by(|&a, &b| {
                    sums[b]
                        .partial_cmp(&sums[a])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            StackOrder::InsideOut => {
                let sums: Vec<f64> = self.series.iter().map(|s| s.values.iter().sum()).collect();
                indices.sort_by(|&a, &b| {
                    sums[b]
                        .partial_cmp(&sums[a])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                let mut new_order = Vec::with_capacity(n);
                let mut top = true;
                for idx in indices {
                    if top {
                        new_order.push(idx);
                    } else {
                        new_order.insert(0, idx);
                    }
                    top = !top;
                }
                indices = new_order;
            }
            StackOrder::Reverse => {
                indices.reverse();
            }
        }

        indices
    }

    fn apply_offset(&self, result: &mut Vec<Vec<StackedPoint>>, n_points: usize) {
        match self.offset {
            StackOffset::None => {}
            StackOffset::Expand => {
                for i in 0..n_points {
                    let total: f64 = result.iter().map(|s| s[i].height()).sum();
                    if total > 0.0 {
                        for s in result.iter_mut() {
                            s[i].y0 /= total;
                            s[i].y1 /= total;
                        }
                    }
                }
            }
            StackOffset::Diverging | StackOffset::Silhouette => {
                for i in 0..n_points {
                    let max_y1 = result.iter().map(|s| s[i].y1).fold(0.0_f64, f64::max);
                    let offset = -max_y1 / 2.0;
                    for s in result.iter_mut() {
                        s[i].y0 += offset;
                        s[i].y1 += offset;
                    }
                }
            }
            StackOffset::Wiggle => {
                if result.is_empty() || n_points == 0 {
                    return;
                }
                let n = result.len();
                for i in 0..n_points {
                    let mut sum = 0.0;
                    let mut total_weight = 0.0;
                    for (j, s) in result.iter().enumerate() {
                        let height = s[i].height();
                        let weight = (n - j) as f64;
                        sum += weight * height;
                        total_weight += weight;
                    }
                    let total: f64 = result.iter().map(|s| s[i].height()).sum();
                    let offset = if total_weight > 0.0 && total > 0.0 {
                        -sum / (total_weight * 2.0)
                    } else {
                        0.0
                    };
                    for s in result.iter_mut() {
                        s[i].y0 += offset;
                        s[i].y1 += offset;
                    }
                }
            }
        }
    }

    /// Effective x positions (falls back to 0..n indices)
    fn effective_x(&self, n_points: usize) -> Vec<f64> {
        if self.x.len() >= n_points {
            self.x[..n_points].to_vec()
        } else {
            (0..n_points).map(|i| i as f64).collect()
        }
    }

    // ---- Demo data ----

    fn make_demo_data(&mut self) {
        let n = 14;
        let mut rng = DemoRng::new(7);
        self.x = (0..n).map(|i| i as f64).collect();
        let names = ["Product A", "Product B", "Product C", "Product D"];
        for (si, name) in names.iter().enumerate() {
            let base = 2.0 + si as f64 * 1.5;
            let mut v = base;
            let values: Vec<f64> = (0..n)
                .map(|i| {
                    v += (rng.next_f64() - 0.5) * 2.0;
                    if v < 0.5 {
                        v = 0.5;
                    }
                    v + (i as f64 * 0.5 + si as f64).sin() * 0.8 + 1.0
                })
                .collect();
            self.series.push(StackSeries::new(*name, values));
        }
    }

    // ---- Fitting ----

    fn fit(&mut self) {
        let stacked = self.compute_stacked();
        let n_points = stacked.first().map(|s| s.len()).unwrap_or(0);
        let xs = self.effective_x(n_points);

        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        for &x in &xs {
            x_min = x_min.min(x);
            x_max = x_max.max(x);
        }
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        for series in &stacked {
            for pt in series {
                y_min = y_min.min(pt.y0);
                y_max = y_max.max(pt.y1);
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
        if (y_max - y_min).abs() < 0.001 {
            y_max = y_min + 1.0;
        }
        self.plot_view.fit_data(x_min, x_max, y_min, y_max);
        self.fitted = true;
    }
}

impl Widget for Stackplot {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.series.is_empty() && self.demo_data {
            self.make_demo_data();
            self.fitted = false;
        }
        if !self.fitted {
            self.fit();
        }

        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx);

        let stacked = self.compute_stacked();
        let n_points = stacked.first().map(|s| s.len()).unwrap_or(0);
        if n_points >= 2 {
            let xs = self.effective_x(n_points);
            for (series_idx, series_data) in stacked.iter().enumerate() {
                let color = self.series[series_idx]
                    .color
                    .unwrap_or_else(|| cycle_color(series_idx));
                let fill = vec4(color.x, color.y, color.z, color.w * 0.85);

                // Band polygon: top edge forward, bottom edge back
                let mut px = Vec::with_capacity(n_points * 2);
                let mut py = Vec::with_capacity(n_points * 2);
                for i in 0..n_points {
                    px.push(xs[i]);
                    py.push(series_data[i].y1);
                }
                for i in (0..n_points).rev() {
                    px.push(xs[i]);
                    py.push(series_data[i].y0);
                }
                self.plot_view.fill_polygon_data(&px, &py, fill);

                // Top edge line
                if self.show_lines {
                    let tops: Vec<f64> = series_data.iter().map(|p| p.y1).collect();
                    self.plot_view.draw_polyline_data(
                        &xs,
                        &tops,
                        darken(color, 0.3),
                        1.5,
                        LineStyle::Solid,
                    );
                }
            }
        }

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
            x if x == live_id!(set_x) => {
                let xs = script_arg_f64_array(vm, &args, 0).unwrap_or_default();
                self.set_x(xs);
            }
            x if x == live_id!(add_series) => {
                let label = script_arg_string(vm, &args, 0).unwrap_or_default();
                let ys = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                let idx = self.series.len();
                self.add_series(StackSeries::new(label, ys).with_color(cycle_color(idx)));
            }
            x if x == live_id!(clear) => {
                self.clear();
            }
            x if x == live_id!(set_title) => {
                self.plot_view.title = script_arg_string(vm, &args, 0).unwrap_or_default();
            }
            x if x == live_id!(set_order) => {
                let name = script_arg_string(vm, &args, 0).unwrap_or_default();
                self.set_order(StackOrder::from_name(&name));
            }
            x if x == live_id!(set_offset) => {
                let name = script_arg_string(vm, &args, 0).unwrap_or_default();
                self.set_offset(StackOffset::from_name(&name));
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
// Streamgraph — centered (silhouette) stacked area chart
// =============================================================================

/// A single series for the streamgraph
#[derive(Clone, Debug)]
pub struct StreamSeries {
    pub name: String,
    pub values: Vec<f64>,
    pub color: Option<Vec4>,
}

impl StreamSeries {
    pub fn new(name: impl Into<String>, values: Vec<f64>) -> Self {
        Self {
            name: name.into(),
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
pub struct Streamgraph {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub series: Vec<StreamSeries>,
    #[rust]
    pub x: Vec<f64>,

    #[live(true)]
    pub demo_data: bool,

    #[rust]
    fitted: bool,
}

impl Streamgraph {
    // ---- Rust-side API (mirrors the 1.0 library surface) ----

    pub fn set_data(&mut self, series: Vec<StreamSeries>, x: Vec<f64>) {
        self.series = series;
        self.x = x;
        self.fitted = false;
    }

    pub fn add_series(&mut self, series: StreamSeries) {
        self.series.push(series);
        self.fitted = false;
    }

    pub fn set_x(&mut self, x: Vec<f64>) {
        self.x = x;
        self.fitted = false;
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn clear(&mut self) {
        self.series.clear();
        self.x.clear();
        self.fitted = false;
    }

    fn n_points(&self) -> usize {
        self.series.iter().map(|s| s.values.len()).max().unwrap_or(0)
    }

    fn effective_x(&self, n_points: usize) -> Vec<f64> {
        if self.x.len() >= n_points {
            self.x[..n_points].to_vec()
        } else {
            (0..n_points).map(|i| i as f64).collect()
        }
    }

    /// Per-point column totals and the maximum total.
    fn totals(&self, n_points: usize) -> (Vec<f64>, f64) {
        let mut totals: Vec<f64> = vec![0.0; n_points];
        for s in &self.series {
            for (i, &val) in s.values.iter().enumerate() {
                if i < totals.len() {
                    totals[i] += val;
                }
            }
        }
        let max_total = totals.iter().cloned().fold(0.0_f64, f64::max);
        (totals, max_total)
    }

    // ---- Demo data ----

    fn make_demo_data(&mut self) {
        let n = 24;
        let mut rng = DemoRng::new(11);
        self.x = (0..n).map(|i| i as f64).collect();
        let names = ["alpha", "beta", "gamma", "delta", "epsilon"];
        for (si, name) in names.iter().enumerate() {
            let phase = si as f64 * 1.3;
            let values: Vec<f64> = (0..n)
                .map(|i| {
                    let t = i as f64 * 0.35;
                    let wave = ((t + phase).sin() * 0.5 + 0.6).max(0.05);
                    wave * (2.0 + si as f64 * 0.6) + rng.next_f64() * 0.6
                })
                .collect();
            self.series.push(StreamSeries::new(*name, values));
        }
    }

    // ---- Fitting ----

    fn fit(&mut self) {
        let n_points = self.n_points();
        let xs = self.effective_x(n_points);
        let (_, max_total) = self.totals(n_points);

        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        for &x in &xs {
            x_min = x_min.min(x);
            x_max = x_max.max(x);
        }
        if !x_min.is_finite() {
            x_min = 0.0;
            x_max = 1.0;
        }
        if x_min == x_max {
            x_min -= 0.5;
            x_max += 0.5;
        }
        let y_max = if max_total > 0.0 { max_total } else { 1.0 };
        self.plot_view.fit_data(x_min, x_max, 0.0, y_max);
        self.fitted = true;
    }
}

impl Widget for Streamgraph {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.series.is_empty() && self.demo_data {
            self.make_demo_data();
            self.fitted = false;
        }
        if !self.fitted {
            self.fit();
        }

        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx);

        let n_points = self.n_points();
        let (totals, max_total) = self.totals(n_points);
        if n_points >= 2 && max_total > 0.0 {
            let xs = self.effective_x(n_points);
            // Silhouette baselines: center each column
            let baselines: Vec<f64> = totals.iter().map(|&t| (max_total - t) / 2.0).collect();
            let mut cumulative = baselines.clone();

            for series_idx in 0..self.series.len() {
                let s = self.series[series_idx].clone();
                let color = s.color.unwrap_or_else(|| cycle_color(series_idx));
                let fill = vec4(color.x, color.y, color.z, color.w * 0.85);

                let mut bottoms = Vec::with_capacity(n_points);
                let mut tops = Vec::with_capacity(n_points);
                for i in 0..n_points {
                    let val = s.values.get(i).copied().unwrap_or(0.0);
                    bottoms.push(cumulative[i]);
                    tops.push(cumulative[i] + val);
                    cumulative[i] += val;
                }

                // Band polygon: top edge forward, bottom edge back
                let mut px = Vec::with_capacity(n_points * 2);
                let mut py = Vec::with_capacity(n_points * 2);
                for i in 0..n_points {
                    px.push(xs[i]);
                    py.push(tops[i]);
                }
                for i in (0..n_points).rev() {
                    px.push(xs[i]);
                    py.push(bottoms[i]);
                }
                self.plot_view.fill_polygon_data(&px, &py, fill);
            }
        }

        // Legend
        let entries: Vec<(String, Vec4)> = self
            .series
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.name.is_empty())
            .map(|(i, s)| (s.name.clone(), s.color.unwrap_or_else(|| cycle_color(i))))
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
            x if x == live_id!(set_x) => {
                let xs = script_arg_f64_array(vm, &args, 0).unwrap_or_default();
                self.set_x(xs);
            }
            x if x == live_id!(add_series) => {
                let label = script_arg_string(vm, &args, 0).unwrap_or_default();
                let ys = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                let idx = self.series.len();
                self.add_series(StreamSeries::new(label, ys).with_color(cycle_color(idx)));
            }
            x if x == live_id!(clear) => {
                self.clear();
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
