// Polar-family charts: PolarPlot (polar coordinates) and RadarChart (spider chart)
//
// Both are NON-cartesian: the prototypes disable the cartesian grid / ticks /
// border and the widgets draw their own polar grids in pixel space, centered
// in plot_view.plot_rect(). draw_axes is still called for the title.

use crate::plot_view::*;
use crate::script_util::*;
use crate::types::*;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*

    mod.plot.PolarPlotBase = #(PolarPlot::register_widget(vm))

    mod.plot.PolarPlot = set_type_default() do mod.plot.PolarPlotBase{
        width: Fill
        height: Fill
        show_grid: false
        show_ticks: false
        show_border: false
        plot_margin: Inset{left: 8.0, top: 28.0, right: 8.0, bottom: 8.0}
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }

    mod.plot.RadarChartBase = #(RadarChart::register_widget(vm))

    mod.plot.RadarChart = set_type_default() do mod.plot.RadarChartBase{
        width: Fill
        height: Fill
        show_grid: false
        show_ticks: false
        show_border: false
        plot_margin: Inset{left: 8.0, top: 28.0, right: 8.0, bottom: 8.0}
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }
}

// =============================================================================
// PolarPlot
// =============================================================================

/// One data series on a polar plot (theta in radians, r in data units)
#[derive(Clone, Debug)]
pub struct PolarSeries {
    pub label: String,
    pub theta: Vec<f64>,
    pub r: Vec<f64>,
    pub color: Option<Vec4>,
    pub marker_style: MarkerStyle,
    pub fill: bool,
}

impl PolarSeries {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            theta: Vec::new(),
            r: Vec::new(),
            color: None,
            marker_style: MarkerStyle::None,
            fill: false,
        }
    }
    pub fn with_data(mut self, theta: Vec<f64>, r: Vec<f64>) -> Self {
        self.theta = theta;
        self.r = r;
        self
    }
    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = Some(color);
        self
    }
    pub fn with_fill(mut self, fill: bool) -> Self {
        self.fill = fill;
        self
    }
    pub fn with_marker(mut self, style: MarkerStyle) -> Self {
        self.marker_style = style;
        self
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct PolarPlot {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub series: Vec<PolarSeries>,
    #[rust]
    r_max: Option<f64>,
    #[live(2.0)]
    pub line_width: f64,
    #[live(true)]
    pub demo_data: bool,
}

impl PolarPlot {
    // ---- Rust-side API (mirrors the 1.0 library surface) ----

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn add_series(&mut self, series: PolarSeries) {
        self.series.push(series);
    }

    pub fn set_r_max(&mut self, r_max: f64) {
        self.r_max = Some(r_max);
    }

    pub fn clear(&mut self) {
        self.series.clear();
        self.r_max = None;
    }

    fn get_r_max(&self) -> f64 {
        let m = self.r_max.unwrap_or_else(|| {
            self.series
                .iter()
                .flat_map(|s| s.r.iter())
                .cloned()
                .fold(0.0f64, f64::max)
                * 1.1
        });
        if m > 0.0 {
            m
        } else {
            1.0
        }
    }

    // ---- Demo data ----

    fn make_demo_data(&mut self) {
        let n = 120;
        let thetas: Vec<f64> = (0..=n)
            .map(|i| i as f64 / n as f64 * std::f64::consts::TAU)
            .collect();
        let rose: Vec<f64> = thetas.iter().map(|t| (3.0 * t).sin().abs()).collect();
        let cardioid: Vec<f64> = thetas.iter().map(|t| 0.5 * (1.0 + t.cos())).collect();
        self.series.push(
            PolarSeries::new("|sin(3t)|")
                .with_data(thetas.clone(), rose)
                .with_color(cycle_color(0))
                .with_fill(true),
        );
        self.series.push(
            PolarSeries::new("cardioid")
                .with_data(thetas, cardioid)
                .with_color(cycle_color(1)),
        );
    }

    // ---- Geometry ----

    fn polar_geometry(&self) -> (f64, f64, f64) {
        let pr = self.plot_view.plot_rect();
        let cx = pr.pos.x + pr.size.x * 0.5;
        let cy = pr.pos.y + pr.size.y * 0.5;
        // Leave room around the circle for the angle labels
        let radius = ((pr.size.x.min(pr.size.y)) * 0.5 - 22.0).max(10.0);
        (cx, cy, radius)
    }

    /// theta/r (math convention: theta counter-clockwise from +x) → pixels
    fn polar_to_px(&self, cx: f64, cy: f64, radius: f64, theta: f64, r: f64, r_max: f64) -> (f32, f32) {
        let nr = (r / r_max) * radius;
        (
            (cx + nr * theta.cos()) as f32,
            (cy - nr * theta.sin()) as f32,
        )
    }

    fn draw_polar_grid(&mut self, cx2d: &mut Cx2d) {
        let (cx, cy, radius) = self.polar_geometry();
        let r_max = self.get_r_max();
        let grid_color = vec4(0.85, 0.85, 0.85, 1.0);
        let levels = 5usize;

        // Concentric grid circles + r-axis tick labels
        for i in 1..=levels {
            let frac = i as f64 / levels as f64;
            let ring = (frac * radius) as f32;
            if i < levels {
                self.plot_view
                    .stroke_circle_px(cx as f32, cy as f32, ring, grid_color, 1.0);
            }
            // r tick label along the +x axis, just above the spoke
            let label = format_tick_value(ScaleType::Linear, frac * r_max);
            self.plot_view.draw_text_centered_px(
                cx2d,
                cx + ring as f64,
                cy - 9.0,
                &label,
                vec4(0.45, 0.45, 0.45, 1.0),
                self.plot_view.tick_font_size,
            );
        }

        // Angular spokes every 30 degrees
        for i in 0..12 {
            let t = i as f64 * std::f64::consts::PI / 6.0;
            self.plot_view.set_color(grid_color);
            self.plot_view.line_px(
                cx as f32,
                cy as f32,
                (cx + radius * t.cos()) as f32,
                (cy - radius * t.sin()) as f32,
                1.0,
            );
        }

        // Outer boundary circle (darker) — drawn as two half arcs
        let border = vec4(0.3, 0.3, 0.3, 1.0);
        self.plot_view.stroke_arc_px(
            cx as f32,
            cy as f32,
            radius as f32,
            0.0,
            std::f32::consts::PI,
            border,
            1.5,
        );
        self.plot_view.stroke_arc_px(
            cx as f32,
            cy as f32,
            radius as f32,
            std::f32::consts::PI,
            std::f32::consts::TAU,
            border,
            1.5,
        );

        // Angle labels at 0/90/180/270 degrees
        for &deg in &[0u32, 90, 180, 270] {
            let t = deg as f64 * std::f64::consts::PI / 180.0;
            let lr = radius + 13.0;
            self.plot_view.draw_text_centered_px(
                cx2d,
                cx + lr * t.cos(),
                cy - lr * t.sin(),
                &format!("{}°", deg),
                vec4(0.3, 0.3, 0.3, 1.0),
                self.plot_view.tick_font_size,
            );
        }
    }

    fn draw_data(&mut self, _cx2d: &mut Cx2d) {
        let (cx, cy, radius) = self.polar_geometry();
        let r_max = self.get_r_max();
        let series = self.series.clone();
        let line_width = self.line_width as f32;
        for (idx, s) in series.iter().enumerate() {
            let n = s.theta.len().min(s.r.len());
            if n == 0 {
                continue;
            }
            let color = s.color.unwrap_or_else(|| cycle_color(idx));
            let pts: Vec<(f32, f32)> = (0..n)
                .map(|i| self.polar_to_px(cx, cy, radius, s.theta[i], s.r[i], r_max))
                .collect();

            // Optional semi-transparent fill of the closed curve
            if s.fill && n >= 3 {
                let fill_color = vec4(color.x, color.y, color.z, 0.25);
                self.plot_view.fill_polygon_px(&pts, fill_color);
            }

            // Closed polyline (last point connects back to first, as in 1.0)
            self.plot_view.set_color(color);
            for i in 0..n {
                let next = (i + 1) % n;
                self.plot_view.line_px(
                    pts[i].0,
                    pts[i].1,
                    pts[next].0,
                    pts[next].1,
                    line_width,
                );
            }

            // Markers
            if s.marker_style != MarkerStyle::None {
                for &(px, py) in &pts {
                    self.plot_view
                        .draw_marker_px(px, py, 5.0, s.marker_style, color);
                }
            }
        }
    }
}

impl Widget for PolarPlot {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.series.is_empty() && self.demo_data {
            self.make_demo_data();
        }
        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx); // grid/ticks/border disabled → title only
        self.draw_polar_grid(cx);
        self.draw_data(cx);

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
                let thetas = script_arg_f64_array(vm, &args, 0).unwrap_or_default();
                let rs = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                self.series.clear();
                self.series.push(PolarSeries::new("").with_data(thetas, rs));
            }
            x if x == live_id!(add_series) => {
                let label = script_arg_string(vm, &args, 0).unwrap_or_default();
                let thetas = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                let rs = script_arg_f64_array(vm, &args, 2).unwrap_or_default();
                let idx = self.series.len();
                self.series.push(
                    PolarSeries::new(label)
                        .with_data(thetas, rs)
                        .with_color(cycle_color(idx)),
                );
            }
            x if x == live_id!(clear) => {
                self.clear();
            }
            x if x == live_id!(set_title) => {
                self.plot_view.title = script_arg_string(vm, &args, 0).unwrap_or_default();
            }
            x if x == live_id!(set_r_max) => {
                if let Some(m) = script_arg_f64(vm, &args, 0) {
                    self.set_r_max(m);
                }
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
// RadarChart
// =============================================================================

/// One data series on a radar (spider) chart — one value per axis
#[derive(Clone, Debug)]
pub struct RadarSeries {
    pub label: String,
    pub values: Vec<f64>,
    pub color: Vec4,
    pub fill_alpha: f64,
}

impl RadarSeries {
    pub fn new(label: impl Into<String>, values: Vec<f64>) -> Self {
        Self {
            label: label.into(),
            values,
            color: vec4(0.12, 0.47, 0.71, 1.0),
            fill_alpha: 0.3,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn with_fill_alpha(mut self, alpha: f64) -> Self {
        self.fill_alpha = alpha;
        self
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct RadarChart {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub axes: Vec<String>,
    #[rust]
    pub axis_vectors: Vec<(f64,f64)>,
    #[rust]
    pub fill_paints: Vec<Option<crate::ChartPaint>>,
    #[rust]
    pub axis_center: (f64,f64),
    #[live(2.0)]
    pub polygon_width: f64,
    #[live(true)]
    pub show_spokes: bool,
    #[live(4.0)]
    pub vertex_radius: f64,
    #[rust]
    pub series: Vec<RadarSeries>,
    #[rust]
    max_value: f64,
    #[rust]
    grid_levels: usize,
    #[live(true)]
    pub show_radar_grid: bool,
    #[live(true)]
    pub demo_data: bool,
}

impl RadarChart {
    // ---- Rust-side API (mirrors the 1.0 library surface) ----

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn set_axes(&mut self, axes: Vec<String>) {
        self.axes = axes;
    }

    pub fn add_series(&mut self, series: RadarSeries) {
        self.series.push(series);
    }

    pub fn set_max_value(&mut self, max: f64) {
        self.max_value = max;
    }

    pub fn set_show_grid(&mut self, show: bool) {
        self.show_radar_grid = show;
    }

    pub fn set_grid_levels(&mut self, levels: usize) {
        self.grid_levels = levels;
    }

    pub fn clear(&mut self) {
        self.series.clear();
        self.fill_paints.clear();
    }

    fn compute_max(&self) -> f64 {
        if self.max_value > 0.0 {
            return self.max_value;
        }
        let max = self
            .series
            .iter()
            .flat_map(|s| s.values.iter())
            .cloned()
            .fold(0.0f64, f64::max);
        if max > 0.0 {
            max * 1.1
        } else {
            1.0
        }
    }

    // ---- Demo data ----

    fn make_demo_data(&mut self) {
        self.axes = vec![
            "Speed".into(),
            "Power".into(),
            "Range".into(),
            "Durability".into(),
            "Precision".into(),
            "Agility".into(),
        ];
        let mut rng = DemoRng::new(7);
        let a: Vec<f64> = (0..6).map(|_| 45.0 + rng.next_f64() * 50.0).collect();
        let b: Vec<f64> = (0..6).map(|_| 30.0 + rng.next_f64() * 60.0).collect();
        self.series.push(
            RadarSeries::new("Model A", a).with_color(cycle_color(0)),
        );
        self.series.push(
            RadarSeries::new("Model B", b).with_color(cycle_color(1)),
        );
    }

    // ---- Geometry ----

    fn radar_geometry(&self) -> (f64, f64, f64) {
        let pr = self.plot_view.plot_rect();
        let cx = pr.pos.x + pr.size.x * 0.5;
        let cy = pr.pos.y + pr.size.y * 0.5;
        // Leave room around the polygon for axis labels
        let radius = ((pr.size.x.min(pr.size.y)) * 0.5 - 42.0).max(30.0);
        (cx, cy, radius)
    }

    fn axis_angle(i: usize, num_axes: usize) -> f64 {
        -std::f64::consts::FRAC_PI_2 + i as f64 * std::f64::consts::TAU / num_axes as f64
    }

    fn draw_radar(&mut self, cx2d: &mut Cx2d) {
        let num_axes = self.axes.len();
        let (cx, cy, radius) = self.radar_geometry();

        if num_axes < 3 {
            self.plot_view.draw_text_centered_px(
                cx2d,
                cx,
                cy,
                "Need at least 3 axes",
                vec4(0.4, 0.4, 0.4, 1.0),
                self.plot_view.label_font_size,
            );
            return;
        }

        let max_val = self.compute_max();
        let grid_levels = if self.grid_levels > 0 { self.grid_levels } else { 5 };

        // Concentric polygon grid
        if self.show_radar_grid {
            let grid_color = vec4(0.8, 0.8, 0.8, 0.7);
            for level in 1..=grid_levels {
                let r = radius * level as f64 / grid_levels as f64;
                let pts: Vec<(f32, f32)> = (0..num_axes)
                    .map(|i| {
                        let a = Self::axis_angle(i, num_axes);
                        ((cx + r * a.cos()) as f32, (cy + r * a.sin()) as f32)
                    })
                    .collect();
                self.plot_view.stroke_polygon_px(&pts, grid_color, 1.0);
            }
        }

        // Axis spokes + labels
        let axes = self.axes.clone();
        let axis_color = vec4(0.5, 0.5, 0.5, 1.0);
        let text_color = vec4(0.25, 0.25, 0.25, 1.0);
        let font_size = self.plot_view.tick_font_size.max(9.0);
        for (i, axis_name) in axes.iter().enumerate().filter(|_|self.show_spokes) {
            let a = Self::axis_angle(i, num_axes);
            let ex = cx + radius * a.cos();
            let ey = cy + radius * a.sin();
            self.plot_view.set_color(axis_color);
            self.plot_view
                .line_px(cx as f32, cy as f32, ex as f32, ey as f32, 1.0);

            // Label anchored away from the polygon
            let lx = cx + (radius + 14.0) * a.cos();
            let ly = cy + (radius + 14.0) * a.sin();
            let est_w = axis_name.len() as f64 * font_size as f64 * 0.5;
            let (tx, ty) = if a.cos().abs() < 0.1 {
                // Above / below: centered horizontally
                if a.sin() < 0.0 {
                    (lx - est_w * 0.5, ly - font_size as f64)
                } else {
                    (lx - est_w * 0.5, ly)
                }
            } else if a.cos() > 0.0 {
                // Right side: text starts at the point
                (lx, ly - font_size as f64 * 0.6)
            } else {
                // Left side: text ends at the point
                (lx - est_w, ly - font_size as f64 * 0.6)
            };
            self.plot_view
                .draw_text_px(cx2d, tx, ty, axis_name, text_color, font_size);
        }

        // Series polygons
        let series = self.series.clone();
        for (si,s) in series.iter().enumerate() {
            if s.values.len() != num_axes {
                continue;
            }
            let pts: Vec<(f32, f32)> = s
                .values
                .iter()
                .enumerate()
                .map(|(i, &val)| {
                    if let Some(&(ax,ay))=self.axis_vectors.get(i) {
                        let pr=self.plot_view.plot_rect();let t=(val/max_val).min(1.0);
                        return ((pr.pos.x+pr.size.x*(self.axis_center.0+ax*t)) as f32,
                                (pr.pos.y+pr.size.y*(self.axis_center.1+ay*t)) as f32);
                    }
                    let a = Self::axis_angle(i, num_axes);
                    let r = (val / max_val).min(1.0) * radius;
                    ((cx + r * a.cos()) as f32, (cy + r * a.sin()) as f32)
                })
                .collect();

            // Semi-transparent fill
            if s.fill_alpha > 0.0 {
                let fill_color = vec4(s.color.x, s.color.y, s.color.z, s.fill_alpha as f32);
                if let Some(Some(paint))=self.fill_paints.get(si) {
                    let pr=*self.plot_view.plot_rect();
                    let d=&mut self.plot_view.draw_vector;d.clear();
                    paint.apply(d,pr.pos.x as f32,pr.pos.y as f32,pr.size.x as f32,pr.size.y as f32);
                    for (i,&(x,y)) in pts.iter().enumerate() {if i==0 {d.move_to(x,y)}else{d.line_to(x,y)}}
                    d.close();d.fill();d.cur_gradient_row_v=-1.;
                }else{self.plot_view.fill_polygon_px(&pts, fill_color);}
            }

            // Outline
            if self.polygon_width>0. {self.plot_view.stroke_polygon_px(&pts, s.color, self.polygon_width as f32);}

            // Vertex points
            for &(px, py) in &pts {
                if self.vertex_radius>0. {self.plot_view.fill_circle_px(px, py, self.vertex_radius as f32, s.color);}
            }
        }
    }
}

impl Widget for RadarChart {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.series.is_empty() && self.demo_data {
            self.make_demo_data();
        }
        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx); // grid/ticks/border disabled → title only
        self.draw_radar(cx);

        let entries: Vec<(String, Vec4)> = self
            .series
            .iter()
            .filter(|s| !s.label.is_empty())
            .map(|s| (s.label.clone(), s.color))
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
            x if x == live_id!(set_axes) => {
                self.axes = script_arg_string_array(vm, &args, 0).unwrap_or_default();
            }
            x if x == live_id!(add_series) => {
                let label = script_arg_string(vm, &args, 0).unwrap_or_default();
                let values = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                let idx = self.series.len();
                self.series
                    .push(RadarSeries::new(label, values).with_color(cycle_color(idx)));
            }
            x if x == live_id!(clear) => {
                self.clear();
            }
            x if x == live_id!(set_title) => {
                self.plot_view.title = script_arg_string(vm, &args, 0).unwrap_or_default();
            }
            x if x == live_id!(set_max_value) => {
                if let Some(m) = script_arg_f64(vm, &args, 0) {
                    self.set_max_value(m);
                }
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
