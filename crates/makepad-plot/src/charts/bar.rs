// Bar-family charts: BarPlot (categorical bars, grouped/stacked/horizontal),
// HistogramChart (binned distribution), CandlestickChart (OHLC), WaterfallChart.
//
// Ported from the Makepad 1.0 makepad-plot library to Makepad 2.0 / Octoscript.

use crate::plot_view::*;
use crate::script_util::*;
use crate::types::*;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*

    mod.plot.BarPlotBase = #(BarPlot::register_widget(vm))

    mod.plot.BarPlot = set_type_default() do mod.plot.BarPlotBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 60.0, top: 28.0, right: 16.0, bottom: 40.0}
        show_grid: false
        show_ticks: false
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }

    mod.plot.HistogramChartBase = #(HistogramChart::register_widget(vm))

    mod.plot.HistogramChart = set_type_default() do mod.plot.HistogramChartBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 52.0, top: 28.0, right: 16.0, bottom: 34.0}
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }

    mod.plot.CandlestickChartBase = #(CandlestickChart::register_widget(vm))

    mod.plot.CandlestickChart = set_type_default() do mod.plot.CandlestickChartBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 52.0, top: 28.0, right: 16.0, bottom: 34.0}
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }

    mod.plot.WaterfallChartBase = #(WaterfallChart::register_widget(vm))

    mod.plot.WaterfallChart = set_type_default() do mod.plot.WaterfallChartBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 60.0, top: 28.0, right: 16.0, bottom: 44.0}
        show_grid: false
        show_ticks: false
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }
}

// =============================================================================
// Shared helpers: manual value-axis ticks for categorical charts
// (show_ticks is disabled in the prototypes because categories replace numeric
// ticks on one axis; the value axis is drawn by hand here.)
// =============================================================================

/// Draw horizontal grid lines + numeric labels along the (vertical) value axis.
fn draw_value_ticks_y(pv: &mut PlotView, cx: &mut Cx2d) {
    let vp = pv.viewport.clone();
    let pr = pv.plot_rect.clone();
    let font = pv.tick_font_size;
    let color = vec4(0.25, 0.25, 0.25, 1.0);
    for &t in &nice_ticks(vp.y_min, vp.y_max, 6) {
        let (_, py) = pv.tdata_to_px(0.0, t);
        pv.draw_grid_line_h_px(cx, py as f64);
        let label = format_tick_value(ScaleType::Linear, t);
        let est_w = label.len() as f64 * font as f64 * 0.55;
        let rx = pv.rect.pos.x;
        pv.draw_text_px(
            cx,
            (pr.pos.x - est_w - 6.0).max(rx),
            py as f64 - font as f64 * 0.6,
            &label,
            color,
            font,
        );
    }
}

/// Draw vertical grid lines + numeric labels along the (horizontal) value axis.
fn draw_value_ticks_x(pv: &mut PlotView, cx: &mut Cx2d) {
    let vp = pv.viewport.clone();
    let pr = pv.plot_rect.clone();
    let font = pv.tick_font_size;
    let color = vec4(0.25, 0.25, 0.25, 1.0);
    for &t in &nice_ticks(vp.x_min, vp.x_max, 6) {
        let (px, _) = pv.tdata_to_px(t, 0.0);
        pv.draw_grid_line_v_px(cx, px as f64);
        let label = format_tick_value(ScaleType::Linear, t);
        pv.draw_text_centered_px(
            cx,
            px as f64,
            pr.pos.y + pr.size.y + 12.0,
            &label,
            color,
            font,
        );
    }
}

// =============================================================================
// BarPlot — categorical bars: vertical/horizontal, grouped, stacked
// =============================================================================

/// A named series of per-category values for grouped / stacked bar charts.
#[derive(Clone, Debug)]
pub struct BarGroup {
    pub label: String,
    pub values: Vec<f64>,
    pub color: Option<Vec4>,
}

impl BarGroup {
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

/// An interval on a numerical categorical axis, including floating/stacked bars.
#[derive(Clone, Debug)]
pub struct BarInterval {
    pub x: f64,
    pub width: f64,
    pub low: f64,
    pub high: f64,
    pub color: Vec4,
    pub paint: Option<crate::ChartPaint>,
    pub radius: f32,
    /// Isometric face depth in logical units; zero selects a flat bar.
    pub depth: f32,
    pub side_colors: Option<(Vec4,Vec4)>,
}

#[derive(Script, ScriptHook, Widget)]
pub struct BarPlot {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub categories: Vec<String>,
    #[rust]
    pub values: Vec<f64>,
    #[rust]
    pub groups: Vec<BarGroup>,
    #[rust]
    pub intervals: Vec<BarInterval>,
    #[rust]
    bar_color: Option<Vec4>,

    #[live(0.8)]
    pub bar_width_ratio: f64,
    #[live(false)]
    pub horizontal: bool,
    #[live(false)]
    pub stacked: bool,
    #[live(false)]
    pub show_bar_labels: bool,
    #[live(true)]
    pub demo_data: bool,
    #[live(true)]
    pub show_category_labels: bool,
    #[live(true)]
    pub show_value_ticks: bool,

    #[rust]
    fitted: bool,
}

impl BarPlot {
    pub fn set_intervals(&mut self, intervals: Vec<BarInterval>, domain: [f64;4]) {
        self.intervals=intervals;
        self.values.clear(); self.groups.clear();
        self.plot_view.set_viewport(domain[0],domain[1],domain[2],domain[3]);
        self.fitted=true;
    }

    fn draw_intervals(&mut self) {
        for bar in &self.intervals {
            let (x0,y0)=self.plot_view.data_to_px(bar.x-bar.width*0.5,bar.high);
            let (x1,y1)=self.plot_view.data_to_px(bar.x+bar.width*0.5,bar.low);
            let (x,y,w,h)=(x0.min(x1),y0.min(y1),(x1-x0).abs(),(y1-y0).abs());
            if w<=0. || h<=0. {continue;}
            if bar.depth>0. {
                let d=bar.depth;let mid=x+w*0.5;
                let (left,right)=bar.side_colors.unwrap_or((bar.color,bar.color));
                self.plot_view.fill_polygon_px(&[(x,y+d),(mid,y+2.*d),(mid,y+h),(x,y+h-d)],left);
                self.plot_view.fill_polygon_px(&[(mid,y+2.*d),(x+w,y+d),(x+w,y+h-d),(mid,y+h)],right);
                self.plot_view.fill_polygon_px(&[(x,y+d),(mid,y),(x+w,y+d),(mid,y+2.*d)],bar.color);
            } else {
                self.plot_view.set_color(bar.color);
                if let Some(paint)=&bar.paint {paint.apply(&mut self.plot_view.draw_vector,x,y,w,h);}
                self.plot_view.draw_vector.rounded_rect(x,y,w,h,bar.radius.min(w*0.5).min(h*0.5));
                self.plot_view.draw_vector.fill();
                self.plot_view.draw_vector.cur_gradient_row_v=-1.;
            }
        }
    }
    // ---- Rust-side API (mirrors the 1.0 library surface) ----

    /// Set bar data (simple mode - single series)
    pub fn set_data(&mut self, categories: Vec<String>, values: Vec<f64>) {
        self.categories = categories;
        self.values = values;
        self.groups.clear();
        self.fitted = false;
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    /// Set bar color (for simple mode)
    pub fn set_color(&mut self, color: Vec4) {
        self.bar_color = Some(color);
    }

    /// Set horizontal orientation (barh)
    pub fn set_horizontal(&mut self, horizontal: bool) {
        self.horizontal = horizontal;
        self.fitted = false;
    }

    /// Set stacked mode
    pub fn set_stacked(&mut self, stacked: bool) {
        self.stacked = stacked;
        self.fitted = false;
    }

    /// Show bar value labels
    pub fn set_show_bar_labels(&mut self, show: bool) {
        self.show_bar_labels = show;
    }

    /// Add a bar group (for grouped/stacked bars)
    pub fn add_group(&mut self, group: BarGroup) {
        self.groups.push(group);
        self.fitted = false;
    }

    /// Set multiple groups at once
    pub fn set_groups(&mut self, categories: Vec<String>, groups: Vec<BarGroup>) {
        self.categories = categories;
        self.groups = groups;
        self.values.clear();
        self.fitted = false;
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.categories.clear();
        self.values.clear();
        self.groups.clear();
        self.fitted = false;
    }

    // ---- Internals ----

    fn num_categories(&self) -> usize {
        if !self.groups.is_empty() {
            let max_len = self.groups.iter().map(|g| g.values.len()).max().unwrap_or(0);
            self.categories.len().max(max_len)
        } else {
            self.categories.len().max(self.values.len())
        }
    }

    fn category_labels(&self) -> Vec<String> {
        let n = self.num_categories();
        (0..n)
            .map(|i| {
                self.categories
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| format!("{}", i + 1))
            })
            .collect()
    }

    fn get_value_range(&self) -> (f64, f64) {
        let max = if !self.groups.is_empty() {
            if self.stacked {
                // For stacked, sum up all groups per category
                let num_cats = self.num_categories();
                let mut max = 0.0f64;
                for cat_idx in 0..num_cats {
                    let sum: f64 = self
                        .groups
                        .iter()
                        .filter_map(|g| g.values.get(cat_idx))
                        .sum();
                    max = max.max(sum);
                }
                max
            } else {
                self.groups
                    .iter()
                    .flat_map(|g| g.values.iter())
                    .cloned()
                    .fold(0.0f64, f64::max)
            }
        } else {
            self.values.iter().cloned().fold(0.0f64, f64::max)
        };
        let max = if max <= 0.0 { 1.0 } else { max };
        (0.0, max * 1.1)
    }

    fn make_demo_data(&mut self) {
        self.categories = vec![
            "Q1".to_string(),
            "Q2".to_string(),
            "Q3".to_string(),
            "Q4".to_string(),
        ];
        self.groups = vec![
            BarGroup::new("Product A", vec![12.0, 17.0, 14.0, 21.0]),
            BarGroup::new("Product B", vec![9.0, 13.0, 16.0, 18.0]),
        ];
        self.fitted = false;
    }

    fn fit(&mut self) {
        let n = self.num_categories().max(1);
        let (_, v_max) = self.get_value_range();
        if self.horizontal {
            self.plot_view
                .set_viewport(0.0, v_max, -0.5, n as f64 - 0.5);
        } else {
            self.plot_view
                .set_viewport(-0.5, n as f64 - 0.5, 0.0, v_max);
        }
        self.fitted = true;
    }

    /// Center position of category `i` on the categorical axis (data space)
    fn cat_center(&self, i: usize) -> f64 {
        if self.horizontal {
            // First category at the top
            (self.num_categories() - 1 - i) as f64
        } else {
            i as f64
        }
    }

    fn draw_category_labels(&mut self, cx: &mut Cx2d) {
        let cats = self.category_labels();
        let pr = self.plot_view.plot_rect.clone();
        let font = self.plot_view.tick_font_size;
        let color = vec4(0.25, 0.25, 0.25, 1.0);
        for (i, cat) in cats.iter().enumerate() {
            let c = self.cat_center(i);
            if self.horizontal {
                let (_, py) = self.plot_view.data_to_px(0.0, c);
                let est_w = cat.len() as f64 * font as f64 * 0.55;
                let rx = self.plot_view.rect.pos.x;
                self.plot_view.draw_text_px(
                    cx,
                    (pr.pos.x - est_w - 6.0).max(rx),
                    py as f64 - font as f64 * 0.6,
                    cat,
                    color,
                    font,
                );
            } else {
                let (px, _) = self.plot_view.data_to_px(c, 0.0);
                self.plot_view.draw_text_centered_px(
                    cx,
                    px as f64,
                    pr.pos.y + pr.size.y + 12.0,
                    cat,
                    color,
                    font,
                );
            }
        }
    }

    fn draw_simple_bars(&mut self, cx: &mut Cx2d) {
        let values = self.values.clone();
        let color = self.bar_color.unwrap_or_else(|| cycle_color(0));
        let bw = self.bar_width_ratio;
        let font = self.plot_view.tick_font_size;
        let label_color = vec4(0.25, 0.25, 0.25, 1.0);
        for (i, &v) in values.iter().enumerate() {
            let c = self.cat_center(i);
            if self.horizontal {
                self.plot_view
                    .fill_rect_data(0.0, c - bw * 0.5, v, c + bw * 0.5, color);
                if self.show_bar_labels {
                    let (px, py) = self.plot_view.data_to_px(v, c);
                    self.plot_view.draw_text_px(
                        cx,
                        px as f64 + 6.0,
                        py as f64 - font as f64 * 0.6,
                        &format!("{:.1}", v),
                        label_color,
                        font,
                    );
                }
            } else {
                self.plot_view
                    .fill_rect_data(c - bw * 0.5, 0.0, c + bw * 0.5, v, color);
                if self.show_bar_labels {
                    let (px, py) = self.plot_view.data_to_px(c, v);
                    self.plot_view.draw_text_centered_px(
                        cx,
                        px as f64,
                        py as f64 - 8.0,
                        &format!("{:.1}", v),
                        label_color,
                        font,
                    );
                }
            }
        }
    }

    fn draw_stacked_bars(&mut self, _cx: &mut Cx2d) {
        let groups = self.groups.clone();
        let num_cats = self.num_categories();
        let bw = self.bar_width_ratio;
        for cat_idx in 0..num_cats {
            let c = self.cat_center(cat_idx);
            let mut cum = 0.0f64;
            for (gi, g) in groups.iter().enumerate() {
                if let Some(&v) = g.values.get(cat_idx) {
                    let color = g.color.unwrap_or_else(|| cycle_color(gi));
                    if self.horizontal {
                        self.plot_view
                            .fill_rect_data(cum, c - bw * 0.5, cum + v, c + bw * 0.5, color);
                    } else {
                        self.plot_view
                            .fill_rect_data(c - bw * 0.5, cum, c + bw * 0.5, cum + v, color);
                    }
                    cum += v;
                }
            }
        }
    }

    fn draw_side_by_side_bars(&mut self, _cx: &mut Cx2d) {
        let groups = self.groups.clone();
        let num_cats = self.num_categories();
        let num_groups = groups.len().max(1);
        let bw = self.bar_width_ratio;
        let gw = bw / num_groups as f64;
        for cat_idx in 0..num_cats {
            let c = self.cat_center(cat_idx);
            let start = c - bw * 0.5;
            for (gi, g) in groups.iter().enumerate() {
                if let Some(&v) = g.values.get(cat_idx) {
                    let color = g.color.unwrap_or_else(|| cycle_color(gi));
                    let b0 = start + gi as f64 * gw;
                    let b1 = b0 + gw * 0.9;
                    if self.horizontal {
                        self.plot_view.fill_rect_data(0.0, b0, v, b1, color);
                    } else {
                        self.plot_view.fill_rect_data(b0, 0.0, b1, v, color);
                    }
                }
            }
        }
    }

    fn draw_bars(&mut self, cx: &mut Cx2d) {
        if !self.intervals.is_empty() {self.draw_intervals();return;}
        if !self.groups.is_empty() {
            if self.stacked {
                self.draw_stacked_bars(cx);
            } else {
                self.draw_side_by_side_bars(cx);
            }
        } else {
            self.draw_simple_bars(cx);
        }
    }
}

impl Widget for BarPlot {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.values.is_empty() && self.groups.is_empty() && self.intervals.is_empty() && self.demo_data {
            self.make_demo_data();
        }
        if !self.fitted {
            self.fit();
        }

        self.plot_view.begin(cx, walk);

        // Manual value-axis grid + tick labels (numeric ticks are disabled in
        // the prototype since one axis is categorical).
        if self.show_value_ticks {
            if self.horizontal {draw_value_ticks_x(&mut self.plot_view, cx);}
            else {draw_value_ticks_y(&mut self.plot_view, cx);}
        }

        self.plot_view.draw_axes(cx); // border + title + axis labels

        self.draw_bars(cx);
        if self.show_category_labels {self.draw_category_labels(cx);}

        // Legend for grouped bars
        let entries: Vec<(String, Vec4)> = self
            .groups
            .iter()
            .enumerate()
            .filter(|(_, g)| !g.label.is_empty())
            .map(|(i, g)| (g.label.clone(), g.color.unwrap_or_else(|| cycle_color(i))))
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
                let labels = script_arg_string_array(vm, &args, 0).unwrap_or_default();
                let values = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                self.set_data(labels, values);
            }
            x if x == live_id!(add_group) => {
                let label = script_arg_string(vm, &args, 0).unwrap_or_default();
                let values = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                let idx = self.groups.len();
                self.add_group(BarGroup::new(label, values).with_color(cycle_color(idx)));
            }
            x if x == live_id!(set_horizontal) => {
                self.set_horizontal(script_arg_bool(vm, &args, 0).unwrap_or(false));
            }
            x if x == live_id!(set_stacked) => {
                self.set_stacked(script_arg_bool(vm, &args, 0).unwrap_or(false));
            }
            x if x == live_id!(set_show_bar_labels) => {
                self.set_show_bar_labels(script_arg_bool(vm, &args, 0).unwrap_or(false));
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

// =============================================================================
// HistogramChart — binned distribution of a set of values
// =============================================================================

#[derive(Clone, Debug)]
pub struct HistogramBin {
    pub left: f64,
    pub right: f64,
    pub count: usize,
}

#[derive(Script, ScriptHook, Widget)]
pub struct HistogramChart {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub values: Vec<f64>,
    #[rust]
    pub bins: Vec<HistogramBin>,
    #[rust]
    num_bins: Option<usize>,
    #[rust]
    bar_color: Option<Vec4>,

    #[live(true)]
    pub demo_data: bool,

    #[rust]
    fitted: bool,
}

impl HistogramChart {
    // ---- Rust-side API ----

    pub fn set_values(&mut self, values: Vec<f64>) {
        self.values = values;
        self.compute_bins();
        self.fitted = false;
    }

    pub fn set_num_bins(&mut self, num_bins: usize) {
        self.num_bins = Some(num_bins);
        if !self.values.is_empty() {
            self.compute_bins();
        }
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

    pub fn set_color(&mut self, color: Vec4) {
        self.bar_color = Some(color);
    }

    pub fn clear(&mut self) {
        self.values.clear();
        self.bins.clear();
        self.fitted = false;
    }

    // ---- Internals ----

    fn compute_bins(&mut self) {
        if self.values.is_empty() {
            self.bins.clear();
            return;
        }

        let min = self.values.iter().cloned().fold(f64::MAX, f64::min);
        let max = self.values.iter().cloned().fold(f64::MIN, f64::max);

        // Sturges' rule when no bin count was given
        let num_bins = self
            .num_bins
            .unwrap_or_else(|| {
                let n = self.values.len() as f64;
                (1.0 + 3.322 * n.log10()).ceil() as usize
            })
            .max(1);

        let bin_width = if max > min {
            (max - min) / num_bins as f64
        } else {
            1.0
        };

        self.bins = (0..num_bins)
            .map(|i| {
                let left = min + i as f64 * bin_width;
                let right = if i == num_bins - 1 {
                    max.max(left + bin_width)
                } else {
                    min + (i + 1) as f64 * bin_width
                };
                HistogramBin { left, right, count: 0 }
            })
            .collect();

        for &value in &self.values {
            let bin_idx = ((value - min) / bin_width).floor() as usize;
            let bin_idx = bin_idx.min(num_bins - 1);
            self.bins[bin_idx].count += 1;
        }
    }

    fn make_demo_data(&mut self) {
        // Approximately normal distribution: sum of 4 uniforms
        let mut rng = DemoRng::new(42);
        let values: Vec<f64> = (0..300)
            .map(|_| {
                let s = rng.next_f64() + rng.next_f64() + rng.next_f64() + rng.next_f64();
                (s - 2.0) * 2.5
            })
            .collect();
        self.set_values(values);
    }

    fn fit(&mut self) {
        if self.bins.is_empty() {
            self.plot_view.set_viewport(0.0, 1.0, 0.0, 1.0);
        } else {
            let x_min = self.bins.first().map(|b| b.left).unwrap_or(0.0);
            let x_max = self.bins.last().map(|b| b.right).unwrap_or(1.0);
            let y_max = self.bins.iter().map(|b| b.count).max().unwrap_or(1) as f64 * 1.1;
            self.plot_view.set_viewport(x_min, x_max, 0.0, y_max);
        }
        self.fitted = true;
    }

    fn draw_bars(&mut self, _cx: &mut Cx2d) {
        let color = self.bar_color.unwrap_or_else(|| cycle_color(0));
        let bins = self.bins.clone();
        for bin in &bins {
            let (px1, py0) = self.plot_view.data_to_px(bin.left, 0.0);
            let (px2, py1) = self.plot_view.data_to_px(bin.right, bin.count as f64);
            let w = (px2 - px1 - 1.0).max(1.0);
            let h = (py0 - py1).max(0.0);
            if h > 0.0 {
                self.plot_view.fill_rect_px(px1, py1, w, h, color);
            }
        }
    }
}

impl Widget for HistogramChart {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.values.is_empty() && self.demo_data {
            self.make_demo_data();
        }
        if !self.fitted {
            self.fit();
        }

        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx);
        self.draw_bars(cx);
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
            x if x == live_id!(set_values) => {
                let values = script_arg_f64_array(vm, &args, 0).unwrap_or_default();
                self.set_values(values);
            }
            x if x == live_id!(set_num_bins) => {
                if let Some(n) = script_arg_f64(vm, &args, 0) {
                    self.set_num_bins(n.max(1.0) as usize);
                }
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
// CandlestickChart — OHLC candles
// =============================================================================

/// A single OHLC candle. (Named PlotCandle to avoid colliding with
/// makepad_widgets::Candle.)
#[derive(Clone, Debug)]
pub struct PlotCandle {
    pub timestamp: f64, // X position (can be index or actual timestamp)
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: Option<f64>,
}

impl PlotCandle {
    pub fn new(timestamp: f64, open: f64, high: f64, low: f64, close: f64) -> Self {
        Self {
            timestamp,
            open,
            high,
            low,
            close,
            volume: None,
        }
    }

    pub fn with_volume(mut self, volume: f64) -> Self {
        self.volume = Some(volume);
        self
    }

    pub fn is_bullish(&self) -> bool {
        self.close >= self.open
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct CandlestickChart {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub candles: Vec<PlotCandle>,
    #[rust]
    bullish_color: Option<Vec4>,
    #[rust]
    bearish_color: Option<Vec4>,

    #[live(0.0)]
    pub candle_width: f64, // 0.0 = auto from candle count
    #[live(true)]
    pub demo_data: bool,

    #[rust]
    fitted: bool,
}

impl CandlestickChart {
    // ---- Rust-side API ----

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn set_data(&mut self, candles: Vec<PlotCandle>) {
        self.candles = candles;
        self.fitted = false;
    }

    pub fn add_candle(&mut self, candle: PlotCandle) {
        self.candles.push(candle);
        self.fitted = false;
    }

    pub fn set_colors(&mut self, bullish: Vec4, bearish: Vec4) {
        self.bullish_color = Some(bullish);
        self.bearish_color = Some(bearish);
    }

    pub fn set_candle_width(&mut self, width: f64) {
        self.candle_width = width;
    }

    pub fn clear(&mut self) {
        self.candles.clear();
        self.fitted = false;
    }

    // ---- Internals ----

    fn make_demo_data(&mut self) {
        let mut rng = DemoRng::new(7);
        let mut price = 100.0f64;
        let mut candles = Vec::with_capacity(40);
        for i in 0..40 {
            let open = price;
            let close = open + (rng.next_f64() - 0.5) * 6.0;
            let high = open.max(close) + rng.next_f64() * 2.0;
            let low = open.min(close) - rng.next_f64() * 2.0;
            candles.push(PlotCandle::new(i as f64, open, high, low, close));
            price = close;
        }
        self.candles = candles;
        self.fitted = false;
    }

    fn fit(&mut self) {
        if self.candles.is_empty() {
            self.plot_view.set_viewport(0.0, 1.0, 0.0, 1.0);
            self.fitted = true;
            return;
        }
        let x_min = self.candles.first().map(|c| c.timestamp).unwrap_or(0.0);
        let x_max = self.candles.last().map(|c| c.timestamp).unwrap_or(1.0);
        let mut y_min = f64::MAX;
        let mut y_max = f64::MIN;
        for c in &self.candles {
            y_min = y_min.min(c.low);
            y_max = y_max.max(c.high);
        }
        let y_range = (y_max - y_min).max(1e-9);
        let n = self.candles.len().max(1);
        let x_pad = if x_max > x_min {
            (x_max - x_min) / n as f64 * 0.75
        } else {
            0.5
        };
        self.plot_view.set_viewport(
            x_min - x_pad,
            x_max + x_pad,
            y_min - y_range * 0.05,
            y_max + y_range * 0.05,
        );
        self.fitted = true;
    }

    fn draw_candles(&mut self, _cx: &mut Cx2d) {
        let candles = self.candles.clone();
        if candles.is_empty() {
            return;
        }
        let bullish = self.bullish_color.unwrap_or_else(|| vec4(0.17, 0.63, 0.17, 1.0));
        let bearish = self.bearish_color.unwrap_or_else(|| vec4(0.84, 0.15, 0.16, 1.0));

        let pr = self.plot_view.plot_rect.clone();
        let cw = if self.candle_width > 0.0 {
            self.candle_width as f32
        } else {
            ((pr.size.x / candles.len() as f64 * 0.7).min(20.0).max(3.0)) as f32
        };

        for candle in &candles {
            let color = if candle.is_bullish() { bullish } else { bearish };
            let (x, high_y) = self.plot_view.data_to_px(candle.timestamp, candle.high);
            let (_, low_y) = self.plot_view.data_to_px(candle.timestamp, candle.low);
            let (_, open_y) = self.plot_view.data_to_px(candle.timestamp, candle.open);
            let (_, close_y) = self.plot_view.data_to_px(candle.timestamp, candle.close);

            // Wick (high-low line)
            self.plot_view.set_color(color);
            self.plot_view.line_px(x, high_y, x, low_y, 1.0);

            // Body
            let body_top = open_y.min(close_y);
            let body_height = (open_y - close_y).abs().max(1.0);
            self.plot_view
                .fill_rect_px(x - cw * 0.5, body_top, cw, body_height, color);
        }
    }
}

impl Widget for CandlestickChart {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.candles.is_empty() && self.demo_data {
            self.make_demo_data();
        }
        if !self.fitted {
            self.fit();
        }

        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx);
        self.draw_candles(cx);
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
                let opens = script_arg_f64_array(vm, &args, 0).unwrap_or_default();
                let highs = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                let lows = script_arg_f64_array(vm, &args, 2).unwrap_or_default();
                let closes = script_arg_f64_array(vm, &args, 3).unwrap_or_default();
                let n = opens.len().min(highs.len()).min(lows.len()).min(closes.len());
                let candles = (0..n)
                    .map(|i| PlotCandle::new(i as f64, opens[i], highs[i], lows[i], closes[i]))
                    .collect();
                self.set_data(candles);
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

// =============================================================================
// WaterfallChart — cumulative deltas with connectors and totals
// =============================================================================

#[derive(Clone, Debug)]
pub struct WaterfallEntry {
    pub label: String,
    pub value: f64,
    pub is_total: bool, // If true, shows absolute value from baseline
}

impl WaterfallEntry {
    pub fn new(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
            is_total: false,
        }
    }

    pub fn total(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
            is_total: true,
        }
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct WaterfallChart {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub entries: Vec<WaterfallEntry>,
    #[rust]
    positive_color: Option<Vec4>,
    #[rust]
    negative_color: Option<Vec4>,
    #[rust]
    total_color: Option<Vec4>,
    #[rust]
    connector_color: Option<Vec4>,

    #[live(0.7)]
    pub bar_width_ratio: f64,
    #[live(true)]
    pub demo_data: bool,

    #[rust]
    fitted: bool,
}

impl WaterfallChart {
    // ---- Rust-side API ----

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn set_data(&mut self, entries: Vec<WaterfallEntry>) {
        self.entries = entries;
        self.fitted = false;
    }

    pub fn add_entry(&mut self, entry: WaterfallEntry) {
        self.entries.push(entry);
        self.fitted = false;
    }

    pub fn set_colors(&mut self, positive: Vec4, negative: Vec4, total: Vec4) {
        self.positive_color = Some(positive);
        self.negative_color = Some(negative);
        self.total_color = Some(total);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.fitted = false;
    }

    // ---- Internals ----

    /// Per-bar (start, end, is_total, value) plus the padded value range.
    fn compute_bar_data(&self) -> (Vec<(f64, f64, bool, f64)>, f64, f64) {
        let mut cumulative = 0.0f64;
        let mut min_val = 0.0f64;
        let mut max_val = 0.0f64;
        let mut bar_data: Vec<(f64, f64, bool, f64)> = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            if entry.is_total {
                bar_data.push((0.0, entry.value, true, entry.value));
                min_val = min_val.min(0.0).min(entry.value);
                max_val = max_val.max(0.0).max(entry.value);
            } else {
                let start = cumulative;
                cumulative += entry.value;
                bar_data.push((start, cumulative, false, entry.value));
                min_val = min_val.min(start).min(cumulative);
                max_val = max_val.max(start).max(cumulative);
            }
        }
        let range = (max_val - min_val).max(1e-9);
        (bar_data, min_val - range * 0.1, max_val + range * 0.1)
    }

    fn make_demo_data(&mut self) {
        self.entries = vec![
            WaterfallEntry::new("Revenue", 420.0),
            WaterfallEntry::new("COGS", -180.0),
            WaterfallEntry::total("Gross", 240.0),
            WaterfallEntry::new("Opex", -90.0),
            WaterfallEntry::new("Tax", -30.0),
            WaterfallEntry::total("Net", 120.0),
        ];
        self.fitted = false;
    }

    fn fit(&mut self) {
        let n = self.entries.len().max(1);
        let (_, min_val, max_val) = self.compute_bar_data();
        self.plot_view
            .set_viewport(-0.5, n as f64 - 0.5, min_val, max_val);
        self.fitted = true;
    }

    fn draw_waterfall(&mut self, cx: &mut Cx2d) {
        let entries = self.entries.clone();
        if entries.is_empty() {
            return;
        }
        let (bar_data, min_val, max_val) = self.compute_bar_data();
        let positive = self.positive_color.unwrap_or_else(|| vec4(0.17, 0.63, 0.17, 1.0));
        let negative = self.negative_color.unwrap_or_else(|| vec4(0.84, 0.15, 0.16, 1.0));
        let total = self.total_color.unwrap_or_else(|| vec4(0.12, 0.47, 0.71, 1.0));
        let connector = self.connector_color.unwrap_or_else(|| vec4(0.5, 0.5, 0.5, 0.5));

        // Zero line if in range
        if min_val < 0.0 && max_val > 0.0 {
            self.plot_view
                .draw_hline(0.0, vec4(0.5, 0.5, 0.5, 0.5), 1.0, LineStyle::Solid);
        }

        let bw = self.bar_width_ratio;
        let pr = self.plot_view.plot_rect.clone();
        let font = self.plot_view.tick_font_size;
        let label_color = vec4(0.25, 0.25, 0.25, 1.0);

        let mut prev_end: Option<f64> = None;
        for (i, ((start, end, is_total, value), entry)) in
            bar_data.iter().zip(entries.iter()).enumerate()
        {
            let xc = i as f64;

            // Connector from previous bar
            if let Some(prev) = prev_end {
                if !is_total {
                    self.plot_view.draw_line_data(
                        xc - 1.0 + bw * 0.5,
                        prev,
                        xc - bw * 0.5,
                        prev,
                        connector,
                        1.0,
                        LineStyle::Solid,
                    );
                }
            }

            // Bar
            let color = if *is_total {
                total
            } else if *value >= 0.0 {
                positive
            } else {
                negative
            };
            self.plot_view
                .fill_rect_data(xc - bw * 0.5, *start, xc + bw * 0.5, *end, color);

            // Category label under the bar
            let (px, _) = self.plot_view.data_to_px(xc, 0.0);
            self.plot_view.draw_text_centered_px(
                cx,
                px as f64,
                pr.pos.y + pr.size.y + 12.0,
                &entry.label,
                label_color,
                font,
            );

            // Value label above (or below, for negatives) the bar
            let (_, py_start) = self.plot_view.data_to_px(xc, *start);
            let (_, py_end) = self.plot_view.data_to_px(xc, *end);
            let bar_top = py_start.min(py_end) as f64;
            let bar_bottom = py_start.max(py_end) as f64;
            let value_y = if *value >= 0.0 {
                bar_top - 9.0
            } else {
                bar_bottom + 9.0
            };
            self.plot_view.draw_text_centered_px(
                cx,
                px as f64,
                value_y,
                &format!("{:.0}", value),
                label_color,
                font,
            );

            prev_end = Some(*end);
        }
    }
}

impl Widget for WaterfallChart {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.entries.is_empty() && self.demo_data {
            self.make_demo_data();
        }
        if !self.fitted {
            self.fit();
        }

        self.plot_view.begin(cx, walk);
        draw_value_ticks_y(&mut self.plot_view, cx);
        self.plot_view.draw_axes(cx); // border + title (grid/ticks disabled)
        self.draw_waterfall(cx);
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
                let labels = script_arg_string_array(vm, &args, 0).unwrap_or_default();
                let values = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                let n = labels.len().min(values.len());
                let entries = (0..n)
                    .map(|i| WaterfallEntry::new(labels[i].clone(), values[i]))
                    .collect();
                self.set_data(entries);
            }
            x if x == live_id!(add_entry) => {
                let label = script_arg_string(vm, &args, 0).unwrap_or_default();
                let value = script_arg_f64(vm, &args, 1).unwrap_or(0.0);
                let is_total = script_arg_bool(vm, &args, 2).unwrap_or(false);
                self.add_entry(WaterfallEntry {
                    label,
                    value,
                    is_total,
                });
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
