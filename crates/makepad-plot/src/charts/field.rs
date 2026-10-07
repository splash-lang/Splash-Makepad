// Field / matrix charts: HeatmapChart (+ Heatmap alias), ContourPlot, QuiverPlot
//
// Ported from the Makepad 1.0 plot library to Makepad 2.0 / Octoscript.

use crate::plot_view::*;
use crate::script_util::*;
use crate::types::*;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*

    mod.plot.HeatmapChartBase = #(HeatmapChart::register_widget(vm))

    mod.plot.HeatmapChart = set_type_default() do mod.plot.HeatmapChartBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 52.0, top: 28.0, right: 64.0, bottom: 34.0}
        show_grid: false
        show_ticks: false
        colormap: "Viridis"
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }

    mod.plot.Heatmap = mod.plot.HeatmapChart

    mod.plot.ContourPlotBase = #(ContourPlot::register_widget(vm))

    mod.plot.ContourPlot = set_type_default() do mod.plot.ContourPlotBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 52.0, top: 28.0, right: 16.0, bottom: 34.0}
        show_grid: false
        colormap: "Viridis"
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }

    mod.plot.QuiverPlotBase = #(QuiverPlot::register_widget(vm))

    mod.plot.QuiverPlot = set_type_default() do mod.plot.QuiverPlotBase{
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
// HeatmapChart — 2D matrix heatmap with colormap, cell values and colorbar
// =============================================================================

#[derive(Script, ScriptHook, Widget)]
pub struct HeatmapChart {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    data: Vec<Vec<f64>>,
    #[rust]
    x_labels: Option<Vec<String>>,
    #[rust]
    y_labels: Option<Vec<String>>,
    #[rust]
    vmin: Option<f64>,
    #[rust]
    vmax: Option<f64>,

    #[live(true)]
    pub show_values: bool,
    #[live(true)]
    pub show_colorbar: bool,
    #[live]
    pub colormap: String,
    #[live(true)]
    pub demo_data: bool,

    #[rust]
    fitted: bool,
}

impl HeatmapChart {
    // ---- Rust-side API (mirrors the 1.0 library surface) ----

    pub fn set_data(&mut self, data: Vec<Vec<f64>>) {
        self.data = data;
        self.fitted = false;
    }

    pub fn set_x_labels(&mut self, labels: Vec<String>) {
        self.x_labels = Some(labels);
    }

    pub fn set_y_labels(&mut self, labels: Vec<String>) {
        self.y_labels = Some(labels);
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn set_colormap(&mut self, colormap: Colormap) {
        self.colormap = colormap.name().to_string();
    }

    pub fn set_vmin(&mut self, vmin: f64) {
        self.vmin = Some(vmin);
    }

    pub fn set_vmax(&mut self, vmax: f64) {
        self.vmax = Some(vmax);
    }

    pub fn set_show_values(&mut self, show: bool) {
        self.show_values = show;
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.x_labels = None;
        self.y_labels = None;
        self.fitted = false;
    }

    fn dims(&self) -> (usize, usize) {
        let rows = self.data.len();
        let cols = if rows > 0 { self.data[0].len() } else { 0 };
        (rows, cols)
    }

    fn get_value_range(&self) -> (f64, f64) {
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for row in &self.data {
            for &val in row {
                min = min.min(val);
                max = max.max(val);
            }
        }
        if min > max {
            min = 0.0;
            max = 1.0;
        }
        (self.vmin.unwrap_or(min), self.vmax.unwrap_or(max))
    }

    fn make_demo_data(&mut self) {
        // Smooth 2D function sampled on an 8x8 grid
        let n = 8;
        let mut data = Vec::with_capacity(n);
        for r in 0..n {
            let mut row = Vec::with_capacity(n);
            for c in 0..n {
                let x = c as f64 * 0.9;
                let y = r as f64 * 0.7;
                let bump = 5.0 * (-((x - 3.0) * (x - 3.0) + (y - 2.0) * (y - 2.0)) / 8.0).exp();
                row.push(x.sin() * y.cos() * 4.0 + bump);
            }
            data.push(row);
        }
        self.data = data;
        self.fitted = false;
    }

    fn fit(&mut self) {
        let (rows, cols) = self.dims();
        // Fit viewport exactly to matrix dimensions (no padding)
        self.plot_view
            .set_viewport(0.0, cols.max(1) as f64, 0.0, rows.max(1) as f64);
        self.fitted = true;
    }

    fn draw_cells(&mut self, cx: &mut Cx2d) {
        let (rows, cols) = self.dims();
        if rows == 0 || cols == 0 {
            return;
        }
        let cmap = Colormap::from_name(&self.colormap);
        let (vmin, vmax) = self.get_value_range();
        let range = (vmax - vmin).max(1e-10);

        // Estimate cell size in px to decide whether value text fits
        let (px0, py0) = self.plot_view.data_to_px(0.0, rows as f64);
        let (px1, py1) = self.plot_view.data_to_px(1.0, (rows - 1) as f64);
        let cell_w_px = (px1 - px0).abs();
        let cell_h_px = (py1 - py0).abs();
        let can_show_values = self.show_values && cell_w_px >= 24.0 && cell_h_px >= 14.0;

        for row_idx in 0..rows {
            for col_idx in 0..cols {
                let value = self.data[row_idx].get(col_idx).copied().unwrap_or(0.0);
                let t = (value - vmin) / range;
                let color = cmap.sample(t);

                // Row 0 at the top (matrix convention): data y decreases with row
                let y_top = (rows - row_idx) as f64;
                let y_bot = (rows - row_idx - 1) as f64;
                let x_lo = col_idx as f64;
                let x_hi = (col_idx + 1) as f64;
                self.plot_view.fill_rect_data(x_lo, y_bot, x_hi, y_top, color);

                if can_show_values {
                    let text_color = if t > 0.5 {
                        vec4(0.0, 0.0, 0.0, 1.0)
                    } else {
                        vec4(1.0, 1.0, 1.0, 1.0)
                    };
                    let (cpx, cpy) = self
                        .plot_view
                        .data_to_px((x_lo + x_hi) * 0.5, (y_bot + y_top) * 0.5);
                    let label = format!("{:.1}", value);
                    self.plot_view.draw_text_centered_px(
                        cx,
                        cpx as f64,
                        cpy as f64,
                        &label,
                        text_color,
                        9.0,
                    );
                }
            }
        }
    }

    fn draw_axis_labels(&mut self, cx: &mut Cx2d) {
        let (rows, cols) = self.dims();
        if rows == 0 || cols == 0 {
            return;
        }
        let text_color = vec4(0.3, 0.3, 0.3, 1.0);
        let font_size = self.plot_view.tick_font_size;
        let pr = self.plot_view.plot_rect().clone();

        // Column tick labels (below the plot)
        for c in 0..cols {
            let label = match &self.x_labels {
                Some(labels) => match labels.get(c) {
                    Some(l) => l.clone(),
                    None => continue,
                },
                None => format!("{}", c),
            };
            let (px, _) = self.plot_view.data_to_px(c as f64 + 0.5, 0.0);
            self.plot_view.draw_text_centered_px(
                cx,
                px as f64,
                pr.pos.y + pr.size.y + 12.0,
                &label,
                text_color,
                font_size,
            );
        }

        // Row tick labels (left of the plot); row 0 at the top
        for r in 0..rows {
            let label = match &self.y_labels {
                Some(labels) => match labels.get(r) {
                    Some(l) => l.clone(),
                    None => continue,
                },
                None => format!("{}", r),
            };
            let (_, py) = self.plot_view.data_to_px(0.0, (rows - r) as f64 - 0.5);
            let est_w = label.len() as f64 * font_size as f64 * 0.55;
            let rect_x = self.plot_view.rect.pos.x;
            self.plot_view.draw_text_px(
                cx,
                (pr.pos.x - est_w - 8.0).max(rect_x),
                py as f64 - font_size as f64 * 0.6,
                &label,
                text_color,
                font_size,
            );
        }
    }

    fn draw_colorbar(&mut self, cx: &mut Cx2d) {
        let cmap = Colormap::from_name(&self.colormap);
        let pr = self.plot_view.plot_rect().clone();
        let bar_width = 12.0f64;
        let bar_x = pr.pos.x + pr.size.x + 10.0;
        let bar_top = pr.pos.y;
        let bar_height = pr.size.y;

        let steps = 50usize;
        let step_height = bar_height / steps as f64;
        for i in 0..steps {
            let t = 1.0 - i as f64 / (steps - 1) as f64;
            let color = cmap.sample(t);
            self.plot_view.fill_rect_px(
                bar_x as f32,
                (bar_top + i as f64 * step_height) as f32,
                bar_width as f32,
                (step_height + 1.0) as f32,
                color,
            );
        }

        let (vmin, vmax) = self.get_value_range();
        let text_color = vec4(0.3, 0.3, 0.3, 1.0);
        let font_size = self.plot_view.tick_font_size;
        let tx = bar_x + bar_width + 3.0;
        let labels = [
            (bar_top, format!("{:.1}", vmax)),
            (
                bar_top + bar_height * 0.5,
                format!("{:.1}", (vmin + vmax) * 0.5),
            ),
            (bar_top + bar_height, format!("{:.1}", vmin)),
        ];
        for (y, label) in labels {
            self.plot_view.draw_text_px(
                cx,
                tx,
                y - font_size as f64 * 0.6,
                &label,
                text_color,
                font_size,
            );
        }
    }
}

impl Widget for HeatmapChart {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.data.is_empty() && self.demo_data {
            self.make_demo_data();
        }
        if !self.fitted {
            self.fit();
        }

        self.plot_view.begin(cx, walk);
        self.draw_cells(cx);
        self.plot_view.draw_axes(cx); // grid/ticks off: border + title/labels only
        self.draw_axis_labels(cx);
        if self.show_colorbar && !self.data.is_empty() {
            self.draw_colorbar(cx);
        }
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
                let data = script_arg_f64_matrix(vm, &args, 0).unwrap_or_default();
                self.set_data(data);
            }
            x if x == live_id!(set_colormap) => {
                self.colormap = script_arg_string(vm, &args, 0).unwrap_or_default();
            }
            x if x == live_id!(set_show_values) => {
                self.show_values = script_arg_bool(vm, &args, 0).unwrap_or(true);
            }
            x if x == live_id!(set_title) => {
                self.plot_view.title = script_arg_string(vm, &args, 0).unwrap_or_default();
            }
            x if x == live_id!(set_x_labels) => {
                self.x_labels = script_arg_string_array(vm, &args, 0);
            }
            x if x == live_id!(set_y_labels) => {
                self.y_labels = script_arg_string_array(vm, &args, 0);
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
// ContourPlot — contour lines from a z-matrix via marching squares
// =============================================================================

#[derive(Script, ScriptHook, Widget)]
pub struct ContourPlot {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    data: Vec<Vec<f64>>,

    #[live(10.0)]
    pub num_levels: f64,
    #[live(false)]
    pub filled: bool,
    #[live]
    pub colormap: String,
    #[live(true)]
    pub demo_data: bool,

    #[rust]
    fitted: bool,
}

impl ContourPlot {
    // ---- Rust-side API (mirrors the 1.0 library surface) ----

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn set_data(&mut self, data: Vec<Vec<f64>>) {
        self.data = data;
        self.fitted = false;
    }

    pub fn set_filled(&mut self, filled: bool) {
        self.filled = filled;
    }

    pub fn set_num_levels(&mut self, n: usize) {
        self.num_levels = n.max(1) as f64;
    }

    pub fn set_colormap(&mut self, colormap: Colormap) {
        self.colormap = colormap.name().to_string();
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.fitted = false;
    }

    fn dims(&self) -> (usize, usize) {
        let rows = self.data.len();
        let cols = if rows > 0 { self.data[0].len() } else { 0 };
        (rows, cols)
    }

    fn make_demo_data(&mut self) {
        // Matlab-style "peaks" function sampled on a 40x40 grid over [-3, 3]
        let n = 40;
        let mut data = Vec::with_capacity(n);
        for r in 0..n {
            let mut row = Vec::with_capacity(n);
            let y = -3.0 + 6.0 * r as f64 / (n - 1) as f64;
            for c in 0..n {
                let x = -3.0 + 6.0 * c as f64 / (n - 1) as f64;
                let z = 3.0 * (1.0 - x).powi(2) * (-x * x - (y + 1.0).powi(2)).exp()
                    - 10.0 * (x / 5.0 - x.powi(3) - y.powi(5)) * (-x * x - y * y).exp()
                    - (1.0 / 3.0) * (-(x + 1.0).powi(2) - y * y).exp();
                row.push(z);
            }
            data.push(row);
        }
        self.data = data;
        self.fitted = false;
    }

    fn fit(&mut self) {
        let (rows, cols) = self.dims();
        // Fit viewport exactly to matrix grid dimensions
        self.plot_view.set_viewport(
            0.0,
            (cols.max(2) - 1) as f64,
            0.0,
            (rows.max(2) - 1) as f64,
        );
        self.fitted = true;
    }

    fn draw_contours(&mut self, _cx: &mut Cx2d) {
        let (rows, cols) = self.dims();
        if rows < 2 || cols < 2 {
            return;
        }
        let cmap = Colormap::from_name(&self.colormap);
        let (mut v_min, mut v_max) = (f64::MAX, f64::MIN);
        for row in &self.data {
            for &v in row {
                v_min = v_min.min(v);
                v_max = v_max.max(v);
            }
        }
        let v_range = (v_max - v_min).max(1e-10);

        // Grid row r maps to data y = rows-1-r so row 0 renders at the top,
        // matching the matrix convention of the old widget.
        let data_y = |r: usize| (rows - 1 - r) as f64;

        if self.filled {
            for row in 0..rows - 1 {
                for col in 0..cols - 1 {
                    let avg = (self.data[row][col]
                        + self.data[row][col + 1]
                        + self.data[row + 1][col]
                        + self.data[row + 1][col + 1])
                        / 4.0;
                    let color = cmap.sample((avg - v_min) / v_range);
                    self.plot_view.fill_rect_data(
                        col as f64,
                        data_y(row + 1),
                        (col + 1) as f64,
                        data_y(row),
                        color,
                    );
                }
            }
        }

        // Marching squares in pixel space (ported from the 1.0 algorithm)
        let n_levels = self.num_levels.max(1.0) as usize;
        for lvl in 1..=n_levels {
            let level = v_min + lvl as f64 * (v_max - v_min) / (n_levels + 1) as f64;
            let color = if self.filled {
                vec4(0.2, 0.2, 0.2, 0.8)
            } else {
                cmap.sample((level - v_min) / v_range)
            };
            self.plot_view.set_color(color);
            for row in 0..rows - 1 {
                for col in 0..cols - 1 {
                    let (v00, v10, v01, v11) = (
                        self.data[row][col],
                        self.data[row][col + 1],
                        self.data[row + 1][col],
                        self.data[row + 1][col + 1],
                    );
                    let case = ((v00 >= level) as u8)
                        | (((v10 >= level) as u8) << 1)
                        | (((v01 >= level) as u8) << 2)
                        | (((v11 >= level) as u8) << 3);
                    if case == 0 || case == 15 {
                        continue;
                    }
                    // Cell corner pixel coords (x0,y0 = top-left of cell on screen)
                    let (x0, y0) = self.plot_view.data_to_px(col as f64, data_y(row));
                    let (x1, y1) = self
                        .plot_view
                        .data_to_px((col + 1) as f64, data_y(row + 1));
                    let cell_w = x1 - x0;
                    let cell_h = y1 - y0;
                    let interp = |a: f64, b: f64| -> f32 {
                        if (b - a).abs() < 1e-10 {
                            0.5
                        } else {
                            (((level - a) / (b - a)) as f32).clamp(0.0, 1.0)
                        }
                    };
                    let tx = x0 + interp(v00, v10) * cell_w; // top edge crossing
                    let bx = x0 + interp(v01, v11) * cell_w; // bottom edge crossing
                    let ly = y0 + interp(v00, v01) * cell_h; // left edge crossing
                    let ry = y0 + interp(v10, v11) * cell_h; // right edge crossing
                    match case {
                        1 | 14 => self.plot_view.line_px(x0, ly, tx, y0, 1.5),
                        2 | 13 => self.plot_view.line_px(tx, y0, x0 + cell_w, ry, 1.5),
                        3 | 12 => self.plot_view.line_px(x0, ly, x0 + cell_w, ry, 1.5),
                        4 | 11 => self.plot_view.line_px(x0, ly, bx, y0 + cell_h, 1.5),
                        // 5 = TL+BL, 10 = TR+BR: vertical column — top edge to bottom edge
                        5 | 10 => self.plot_view.line_px(tx, y0, bx, y0 + cell_h, 1.5),
                        7 | 8 => self.plot_view.line_px(bx, y0 + cell_h, x0 + cell_w, ry, 1.5),
                        // 6 = TR+BL, 9 = TL+BR: saddles — two opposite corner segments
                        6 => {
                            self.plot_view.line_px(tx, y0, x0 + cell_w, ry, 1.5);
                            self.plot_view.line_px(x0, ly, bx, y0 + cell_h, 1.5);
                        }
                        9 => {
                            self.plot_view.line_px(x0, ly, tx, y0, 1.5);
                            self.plot_view.line_px(bx, y0 + cell_h, x0 + cell_w, ry, 1.5);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

impl Widget for ContourPlot {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.data.is_empty() && self.demo_data {
            self.make_demo_data();
        }
        if !self.fitted {
            self.fit();
        }

        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx);
        self.draw_contours(cx);
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
                let data = script_arg_f64_matrix(vm, &args, 0).unwrap_or_default();
                self.set_data(data);
            }
            x if x == live_id!(set_num_levels) => {
                if let Some(n) = script_arg_f64(vm, &args, 0) {
                    self.num_levels = n.max(1.0);
                }
            }
            x if x == live_id!(set_filled) => {
                self.filled = script_arg_bool(vm, &args, 0).unwrap_or(false);
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

// =============================================================================
// QuiverPlot — vector field arrows, colored by magnitude
// =============================================================================

#[derive(Script, ScriptHook, Widget)]
pub struct QuiverPlot {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    x: Vec<f64>,
    #[rust]
    y: Vec<f64>,
    #[rust]
    u: Vec<f64>,
    #[rust]
    v: Vec<f64>,

    // 0.0 = auto scale (arrows sized relative to the data range)
    #[live(0.0)]
    pub scale: f64,
    #[live]
    pub colormap: String,
    #[live(true)]
    pub demo_data: bool,

    #[rust]
    fitted: bool,
}

impl QuiverPlot {
    // ---- Rust-side API (mirrors the 1.0 library surface) ----

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn set_data(&mut self, x: Vec<f64>, y: Vec<f64>, u: Vec<f64>, v: Vec<f64>) {
        self.x = x;
        self.y = y;
        self.u = u;
        self.v = v;
        self.fitted = false;
    }

    pub fn set_scale(&mut self, scale: f64) {
        self.scale = scale;
    }

    pub fn clear(&mut self) {
        self.x.clear();
        self.y.clear();
        self.u.clear();
        self.v.clear();
        self.fitted = false;
    }

    fn make_demo_data(&mut self) {
        // Swirling vortex field on a 13x13 grid over [-2, 2]
        let n = 13;
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        let mut us = Vec::new();
        let mut vs = Vec::new();
        for r in 0..n {
            let y = -2.0 + 4.0 * r as f64 / (n - 1) as f64;
            for c in 0..n {
                let x = -2.0 + 4.0 * c as f64 / (n - 1) as f64;
                let rr = x * x + y * y;
                let damp = (-rr / 4.0).exp();
                xs.push(x);
                ys.push(y);
                us.push(-y * damp);
                vs.push(x * damp);
            }
        }
        self.x = xs;
        self.y = ys;
        self.u = us;
        self.v = vs;
        self.fitted = false;
    }

    fn fit(&mut self) {
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        for &x in &self.x {
            x_min = x_min.min(x);
            x_max = x_max.max(x);
        }
        for &y in &self.y {
            y_min = y_min.min(y);
            y_max = y_max.max(y);
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

    fn draw_arrows(&mut self, _cx: &mut Cx2d) {
        let n = self
            .x
            .len()
            .min(self.y.len())
            .min(self.u.len())
            .min(self.v.len());
        if n == 0 {
            return;
        }
        let cmap = Colormap::from_name(&self.colormap);
        let max_mag = self
            .u
            .iter()
            .zip(self.v.iter())
            .map(|(&u, &v)| (u * u + v * v).sqrt())
            .fold(0.0f64, f64::max);

        // Auto scale: longest arrow spans ~10% of the smaller data range
        let vp = self.plot_view.viewport.clone();
        let scale = if self.scale > 0.0 {
            self.scale
        } else if max_mag > 0.0 {
            0.1 * vp.x_range().min(vp.y_range()) / max_mag
        } else {
            1.0
        };

        for i in 0..n {
            let mag = (self.u[i] * self.u[i] + self.v[i] * self.v[i]).sqrt();
            let t = if max_mag > 0.0 { mag / max_mag } else { 0.0 };
            let color = cmap.sample(t);
            let x1 = self.x[i];
            let y1 = self.y[i];
            let x2 = x1 + self.u[i] * scale;
            let y2 = y1 + self.v[i] * scale;
            self.plot_view.draw_arrow_data(x1, y1, x2, y2, color, 1.5, 5.0);
        }
    }
}

impl Widget for QuiverPlot {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.x.is_empty() && self.demo_data {
            self.make_demo_data();
        }
        if !self.fitted {
            self.fit();
        }

        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx);
        self.draw_arrows(cx);
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
                let us = script_arg_f64_array(vm, &args, 2).unwrap_or_default();
                let vs = script_arg_f64_array(vm, &args, 3).unwrap_or_default();
                self.set_data(xs, ys, us, vs);
            }
            x if x == live_id!(set_scale) => {
                if let Some(s) = script_arg_f64(vm, &args, 0) {
                    self.scale = s;
                }
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
