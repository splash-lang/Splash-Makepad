// Line-family charts: LinePlot (exemplar for the Octoscript port)
//
// Additional line-family widgets (StepPlot, AreaChart, StemPlot, Stackplot,
// Streamgraph, LinePlotDual) follow the same pattern below.

use crate::plot_view::*;
use crate::script_util::*;
use crate::types::*;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*

    mod.plot.LinePlotBase = #(LinePlot::register_widget(vm))

    mod.plot.LinePlot = set_type_default() do mod.plot.LinePlotBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 52.0, top: 28.0, right: 16.0, bottom: 34.0}
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct LinePlot {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub series: Vec<Series>,
    #[rust]
    pub fill_regions: Vec<FillRegion>,
    #[rust]
    pub fill_paints: Vec<Option<crate::ChartPaint>>,
    #[rust]
    pub series_paints: Vec<Option<crate::ChartPaint>>,
    #[rust]
    pub annotations: Vec<TextAnnotation>,
    #[rust]
    pub arrow_annotations: Vec<ArrowAnnotation>,
    #[rust]
    pub vlines: Vec<VLine>,
    #[rust]
    pub hlines: Vec<HLine>,
    #[rust]
    pub vspans: Vec<VSpan>,
    #[rust]
    pub hspans: Vec<HSpan>,

    // Explicit axis limits (raw data space); None = auto-fit
    #[rust]
    x_lim: Option<(f64, f64)>,
    #[rust]
    y_lim: Option<(f64, f64)>,

    #[live(true)]
    pub show_points: bool,
    #[live(4.0)]
    pub point_radius: f64,
    #[live(2.0)]
    pub line_width: f64,
    #[live(true)]
    pub demo_data: bool,
    #[live(false)]
    pub gradient_fill: bool,

    #[rust]
    fitted: bool,
}

impl LinePlot {
    // ---- Rust-side API (mirrors the 1.0 library surface) ----

    pub fn add_series(&mut self, series: Series) {
        self.series.push(series);
        self.fitted = false;
    }

    pub fn clear(&mut self) {
        self.series.clear();
        self.fill_regions.clear();
        self.fill_paints.clear();self.series_paints.clear();
        self.annotations.clear();
        self.arrow_annotations.clear();
        self.vlines.clear();
        self.hlines.clear();
        self.vspans.clear();
        self.hspans.clear();
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

    pub fn set_show_points(&mut self, show: bool) {
        self.show_points = show;
    }

    pub fn set_line_width(&mut self, width: f64) {
        self.line_width = width;
    }

    pub fn fill_between(&mut self, x: Vec<f64>, y1: Vec<f64>, y2: Vec<f64>, color: Vec4) {
        self.fill_regions.push(FillRegion { x, y1, y2, color });
    }

    pub fn fill_between_baseline(&mut self, x: Vec<f64>, y: Vec<f64>, baseline: f64, color: Vec4) {
        let y2 = vec![baseline; x.len()];
        self.fill_regions.push(FillRegion { x, y1: y, y2, color });
    }

    pub fn annotate(&mut self, text: impl Into<String>, x: f64, y: f64, color: Vec4, font_size: f64) {
        self.annotations.push(TextAnnotation {
            text: text.into(),
            x,
            y,
            color,
            font_size,
            is_math: false,
        });
    }

    pub fn axvline(&mut self, x: f64, color: Vec4, line_width: f64, line_style: LineStyle) {
        self.vlines.push(VLine { x, color, line_width, line_style });
    }

    pub fn axhline(&mut self, y: f64, color: Vec4, line_width: f64, line_style: LineStyle) {
        self.hlines.push(HLine { y, color, line_width, line_style });
    }

    pub fn axvspan(&mut self, x1: f64, x2: f64, color: Vec4) {
        self.vspans.push(VSpan { x1, x2, color });
    }

    pub fn axhspan(&mut self, y1: f64, y2: f64, color: Vec4) {
        self.hspans.push(HSpan { y1, y2, color });
    }

    pub fn add_arrow(&mut self, arrow: ArrowAnnotation) {
        self.arrow_annotations.push(arrow);
    }

    pub fn set_legend(&mut self, position: LegendPosition) {
        self.plot_view.legend = position;
    }

    pub fn set_x_scale(&mut self, scale: ScaleType) {
        self.plot_view.x_scale = scale;
        self.fitted = false;
    }

    pub fn set_y_scale(&mut self, scale: ScaleType) {
        self.plot_view.y_scale = scale;
        self.fitted = false;
    }

    pub fn set_interactive(&mut self, interactive: bool) {
        self.plot_view.interactive = interactive;
    }

    pub fn reset_view(&mut self) {
        self.fitted = false;
    }

    // ---- Demo data ----

    fn make_demo_data(&mut self) {
        let n = 100;
        let xs: Vec<f64> = (0..n).map(|i| i as f64 * 0.1).collect();
        let sin: Vec<f64> = xs.iter().map(|x| x.sin()).collect();
        let cos: Vec<f64> = xs.iter().map(|x| (x * 0.7).cos() * 0.8).collect();
        self.series.push(
            Series::new("sin(x)")
                .with_data(xs.clone(), sin)
                .with_color(cycle_color(0)),
        );
        self.series.push(
            Series::new("0.8 cos(0.7x)")
                .with_data(xs, cos)
                .with_color(cycle_color(1))
                .with_line_style(LineStyle::Dashed),
        );
    }

    // ---- Fitting ----

    fn fit(&mut self) {
        // Collect ranges from series + fill regions
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        for s in &self.series {
            for &x in &s.x {
                x_min = x_min.min(x);
                x_max = x_max.max(x);
            }
            for (i, &y) in s.y.iter().enumerate() {
                let lo = y - s.yerr_minus.as_ref().and_then(|e| e.get(i).copied()).unwrap_or(0.0);
                let hi = y + s.yerr_plus.as_ref().and_then(|e| e.get(i).copied()).unwrap_or(0.0);
                y_min = y_min.min(lo);
                y_max = y_max.max(hi);
            }
        }
        for f in &self.fill_regions {
            for &x in &f.x {
                x_min = x_min.min(x);
                x_max = x_max.max(x);
            }
            for &y in f.y1.iter().chain(f.y2.iter()) {
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

    fn draw_series(&mut self, _cx: &mut Cx2d) {
        for si in 0..self.series.len() {
            let s = self.series[si].clone();
            let color = s.color.unwrap_or_else(|| cycle_color(si));
            let width = s.line_width.unwrap_or(self.line_width) as f32;

            if let Some(Some(paint))=self.series_paints.get(si) {
                let pts:Vec<_>=s.x.iter().zip(&s.y).map(|(&x,&y)|self.plot_view.data_to_px(x,y)).collect();
                let pr=*self.plot_view.plot_rect();
                let d=&mut self.plot_view.draw_vector;
                d.path.clear();
                for (i,&(x,y)) in pts.iter().enumerate() {if i==0 {d.move_to(x,y)}else{d.line_to(x,y)}}
                paint.apply(d,pr.pos.x as f32,pr.pos.y as f32,pr.size.x as f32,pr.size.y as f32);
                d.stroke(width);
            } else {
                // Step interpolation if requested
                match s.step_style {
                    StepStyle::None => {
                        self.plot_view
                            .draw_polyline_data(&s.x, &s.y, color, width, s.line_style);
                    }
                    _ => {
                        let (sx, sy) = step_points(&s.x, &s.y, s.step_style);
                        self.plot_view
                            .draw_polyline_data(&sx, &sy, color, width, s.line_style);
                    }
                }
            }

            // Error bars
            let n = s.x.len().min(s.y.len());
            for i in 0..n {
                let x = s.x[i];
                let y = s.y[i];
                if let (Some(em), Some(ep)) = (&s.yerr_minus, &s.yerr_plus) {
                    let lo = y - em.get(i).copied().unwrap_or(0.0);
                    let hi = y + ep.get(i).copied().unwrap_or(0.0);
                    self.plot_view
                        .draw_line_data(x, lo, x, hi, color, 1.0, LineStyle::Solid);
                    // caps
                    let (px, plo) = self.plot_view.data_to_px(x, lo);
                    let (_, phi) = self.plot_view.data_to_px(x, hi);
                    self.plot_view.set_color(color);
                    self.plot_view.line_px(px - 3.0, plo, px + 3.0, plo, 1.0);
                    self.plot_view.line_px(px - 3.0, phi, px + 3.0, phi, 1.0);
                }
                if let (Some(em), Some(ep)) = (&s.xerr_minus, &s.xerr_plus) {
                    let lo = x - em.get(i).copied().unwrap_or(0.0);
                    let hi = x + ep.get(i).copied().unwrap_or(0.0);
                    self.plot_view
                        .draw_line_data(lo, y, hi, y, color, 1.0, LineStyle::Solid);
                    let (plo, py) = self.plot_view.data_to_px(lo, y);
                    let (phi, _) = self.plot_view.data_to_px(hi, y);
                    self.plot_view.set_color(color);
                    self.plot_view.line_px(plo, py - 3.0, plo, py + 3.0, 1.0);
                    self.plot_view.line_px(phi, py - 3.0, phi, py + 3.0, 1.0);
                }
            }

            // Markers
            let marker = if s.marker_style == MarkerStyle::None && self.show_points {
                MarkerStyle::None // only draw markers when explicitly requested
            } else {
                s.marker_style
            };
            if marker != MarkerStyle::None {
                let msize = s.marker_size.unwrap_or(self.point_radius) as f32;
                for i in 0..n {
                    self.plot_view
                        .draw_marker_data(s.x[i], s.y[i], msize, marker, color);
                }
            }
        }
    }
}

/// Expand points into step-style points
pub fn step_points(x: &[f64], y: &[f64], style: StepStyle) -> (Vec<f64>, Vec<f64>) {
    let n = x.len().min(y.len());
    let mut sx = Vec::with_capacity(n * 2);
    let mut sy = Vec::with_capacity(n * 2);
    if n == 0 {
        return (sx, sy);
    }
    sx.push(x[0]);
    sy.push(y[0]);
    for i in 1..n {
        match style {
            StepStyle::Pre => {
                sx.push(x[i - 1]);
                sy.push(y[i]);
            }
            StepStyle::Post => {
                sx.push(x[i]);
                sy.push(y[i - 1]);
            }
            StepStyle::Mid => {
                let mid = (x[i - 1] + x[i]) * 0.5;
                sx.push(mid);
                sy.push(y[i - 1]);
                sx.push(mid);
                sy.push(y[i]);
            }
            StepStyle::None => {}
        }
        sx.push(x[i]);
        sy.push(y[i]);
    }
    (sx, sy)
}

impl Widget for LinePlot {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.series.is_empty() && self.fill_regions.is_empty() && self.demo_data {
            self.make_demo_data();
        }
        if !self.fitted {
            self.fit();
        }

        self.plot_view.begin(cx, walk);

        // Spans below everything
        let vspans = self.vspans.clone();
        for sp in vspans {
            let vp = self.plot_view.viewport.clone();
            let y_lo = self.plot_view.y_scale.inverse(vp.y_min);
            let y_hi = self.plot_view.y_scale.inverse(vp.y_max);
            self.plot_view.fill_rect_data(sp.x1, y_lo, sp.x2, y_hi, sp.color);
        }
        let hspans = self.hspans.clone();
        for sp in hspans {
            let vp = self.plot_view.viewport.clone();
            let x_lo = self.plot_view.x_scale.inverse(vp.x_min);
            let x_hi = self.plot_view.x_scale.inverse(vp.x_max);
            self.plot_view.fill_rect_data(x_lo, sp.y1, x_hi, sp.y2, sp.color);
        }

        self.plot_view.draw_axes(cx);

        // Fill regions (under lines)
        let fills = self.fill_regions.clone();
        for (index,f) in fills.into_iter().enumerate() {
            let n = f.x.len().min(f.y1.len()).min(f.y2.len());
            if n >= 2 {
                let mut xs = Vec::with_capacity(n * 2);
                let mut ys = Vec::with_capacity(n * 2);
                for i in 0..n {
                    xs.push(f.x[i]);
                    ys.push(f.y1[i]);
                }
                for i in (0..n).rev() {
                    xs.push(f.x[i]);
                    ys.push(f.y2[i]);
                }
                if let Some(Some(paint))=self.fill_paints.get(index) {
                    let pr=*self.plot_view.plot_rect();
                    paint.apply(&mut self.plot_view.draw_vector,pr.pos.x as f32,pr.pos.y as f32,pr.size.x as f32,pr.size.y as f32);
                    for i in 0..xs.len() {
                        let (px,py)=self.plot_view.data_to_px(xs[i],ys[i]);
                        if i==0 {self.plot_view.draw_vector.move_to(px,py);}else{self.plot_view.draw_vector.line_to(px,py);}
                    }
                    self.plot_view.draw_vector.close();self.plot_view.draw_vector.fill();
                    self.plot_view.draw_vector.cur_gradient_row_v=-1.;
                } else if self.gradient_fill {
                    self.plot_view.fill_polygon_gradient(&xs, &ys, f.color);
                } else {
                    self.plot_view.fill_polygon_data(&xs, &ys, f.color);
                }
            }
        }

        self.draw_series(cx);

        // Reference lines
        let vlines = self.vlines.clone();
        for l in vlines {
            self.plot_view
                .draw_vline(l.x, l.color, l.line_width as f32, l.line_style);
        }
        let hlines = self.hlines.clone();
        for l in hlines {
            self.plot_view
                .draw_hline(l.y, l.color, l.line_width as f32, l.line_style);
        }

        // Arrows
        let arrows = self.arrow_annotations.clone();
        for a in arrows {
            self.plot_view.draw_arrow_data(
                a.start_x,
                a.start_y,
                a.end_x,
                a.end_y,
                a.color,
                a.line_width as f32,
                a.head_size as f32,
            );
            if let Some(text) = &a.text {
                self.plot_view.draw_text_data(
                    cx,
                    a.start_x,
                    a.start_y,
                    text,
                    a.color,
                    10.0,
                );
            }
        }

        // Text annotations
        let notes = self.annotations.clone();
        for a in notes {
            self.plot_view
                .draw_text_data(cx, a.x, a.y, &a.text, a.color, a.font_size as f32);
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
            x if x == live_id!(set_data) => {
                let xs = script_arg_f64_array(vm, &args, 0).unwrap_or_default();
                let ys = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                self.series.clear();
                self.series.push(Series::new("").with_data(xs, ys));
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
            x if x == live_id!(set_xlim) => {
                if let (Some(lo), Some(hi)) = (
                    script_arg_f64(vm, &args, 0),
                    script_arg_f64(vm, &args, 1),
                ) {
                    self.set_xlim(lo, hi);
                }
            }
            x if x == live_id!(set_ylim) => {
                if let (Some(lo), Some(hi)) = (
                    script_arg_f64(vm, &args, 0),
                    script_arg_f64(vm, &args, 1),
                ) {
                    self.set_ylim(lo, hi);
                }
            }
            x if x == live_id!(set_interactive) => {
                self.plot_view.interactive = script_arg_bool(vm, &args, 0).unwrap_or(true);
            }
            x if x == live_id!(reset_view) => {
                self.fitted = false;
            }
            x if x == live_id!(axvline) => {
                if let Some(xv) = script_arg_f64(vm, &args, 0) {
                    self.axvline(xv, vec4(0.8, 0.2, 0.2, 1.0), 1.5, LineStyle::Dashed);
                }
            }
            x if x == live_id!(axhline) => {
                if let Some(yv) = script_arg_f64(vm, &args, 0) {
                    self.axhline(yv, vec4(0.8, 0.2, 0.2, 1.0), 1.5, LineStyle::Dashed);
                }
            }
            x if x == live_id!(annotate) => {
                let text = script_arg_string(vm, &args, 0).unwrap_or_default();
                let ax = script_arg_f64(vm, &args, 1).unwrap_or(0.0);
                let ay = script_arg_f64(vm, &args, 2).unwrap_or(0.0);
                self.annotate(text, ax, ay, vec4(0.2, 0.2, 0.2, 1.0), 11.0);
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
