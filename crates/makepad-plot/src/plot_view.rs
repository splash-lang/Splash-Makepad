// PlotView — core cartesian plot widget for makepad-plot (Makepad 2.0 / Octoscript)
//
// All cartesian charts embed a PlotView via #[deref] and use its coordinate
// transforms + DrawVector helpers, in the same way makepad's built-in charts
// embed ChartView.

use crate::types::*;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*

    mod.plot.PlotViewBase = #(PlotView::register_widget(vm))

    mod.plot.PlotView = set_type_default() do mod.plot.PlotViewBase{
        width: Fill
        height: Fill

        draw_bg +: {
            draw_depth: 0.0
            color: #xffffff
        }

        draw_grid +: {
            // Keep every plot layer inside the paint-order (zbias) band so
            // chrome drawn after the board wins the depth test; the internal
            // layering still increases per layer.
            draw_depth: 0.0001
            color: #xe0e0e0
        }

        draw_vector +: {
            draw_depth: 0.0002
        }

        draw_text +: {
            draw_depth: 0.0003
            color: #x333333
            text_style: theme.font_regular{}
        }
    }
}

/// Data-space viewport (in scale-transformed coordinates)
#[derive(Clone, Debug)]
pub struct PlotViewport {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl Default for PlotViewport {
    fn default() -> Self {
        Self {
            x_min: 0.0,
            x_max: 1.0,
            y_min: 0.0,
            y_max: 1.0,
        }
    }
}

impl PlotViewport {
    pub fn x_range(&self) -> f64 {
        self.x_max - self.x_min
    }
    pub fn y_range(&self) -> f64 {
        self.y_max - self.y_min
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct PlotView {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[redraw]
    #[live]
    pub draw_bg: DrawColor,
    #[live]
    pub draw_grid: DrawColor,
    #[live]
    pub draw_vector: DrawVector,
    #[live]
    pub draw_text: DrawText,
    /// Embedded source designs may intentionally paint strokes beyond their
    /// geometric frame. Standalone charts keep their existing clipping.
    #[live(true)]
    pub clip_plot: bool,

    // Titles & labels
    #[live]
    pub title: String,
    #[live]
    pub xlabel: String,
    #[live]
    pub ylabel: String,
    #[live(14.0)]
    pub title_font_size: f32,
    #[live(11.0)]
    pub label_font_size: f32,
    #[live(9.0)]
    pub tick_font_size: f32,

    // Axis configuration
    #[live]
    pub x_scale: ScaleType,
    #[live]
    pub y_scale: ScaleType,
    #[live(true)]
    pub show_grid: bool,
    #[live(true)]
    pub show_border: bool,
    #[live(true)]
    pub show_ticks: bool,
    #[live]
    pub legend: LegendPosition,

    // Colors
    #[live]
    pub grid_color: Vec4,
    #[live]
    pub text_color: Vec4,
    #[live]
    pub border_color: Vec4,

    // Interaction
    #[live(false)]
    pub interactive: bool,
    /// Extra space when fitting data; measured UI charts can request exact domains.
    #[live(0.05)]
    pub data_padding: f64,

    #[live]
    pub plot_margin: Inset,

    // Runtime state
    #[rust]
    pub viewport: PlotViewport,
    #[rust]
    pub rect: Rect,
    #[rust]
    pub plot_rect: Rect,
    #[rust]
    drag_start_abs: Option<DVec2>,
    #[rust]
    drag_start_viewport: PlotViewport,
    #[rust]
    dash_offset: f64,
    // Text is queued during the draw pass and flushed in end() AFTER the vector
    // layer: text glyph quads write depth across their whole quad, so any vector
    // geometry drawn (flushed) after a text call gets depth-rejected underneath
    // the glyphs, punching background-colored holes into fills.
    #[rust]
    text_queue: Vec<QueuedText>,
}

#[derive(Clone)]
struct QueuedText {
    x: f64,
    y: f64,
    text: String,
    color: Vec4,
    size: f32,
}

impl Widget for PlotView {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        self.handle_plot_event(cx, event);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // A bare PlotView draws its background, grid and decorations
        self.begin(cx, walk);
        self.draw_axes(cx);
        self.end(cx);
        DrawStep::done()
    }
}

impl PlotView {
    // ---- Event handling (pan / zoom, shared by all embedding charts) ----

    pub fn handle_plot_event(&mut self, cx: &mut Cx, event: &Event) {
        if !self.interactive {
            return;
        }
        match event.hits_with_capture_overload(cx, self.draw_bg.area(), true) {
            Hit::FingerDown(fe) if fe.is_primary_hit() => {
                self.drag_start_abs = Some(fe.abs);
                self.drag_start_viewport = self.viewport.clone();
                cx.set_cursor(MouseCursor::Grabbing);
            }
            Hit::FingerMove(fe) => {
                if let Some(start_abs) = self.drag_start_abs {
                    let delta = fe.abs - start_abs;
                    let pr = &self.plot_rect;
                    if pr.size.x > 0.0 && pr.size.y > 0.0 {
                        let dx = delta.x / pr.size.x * self.drag_start_viewport.x_range();
                        let dy = delta.y / pr.size.y * self.drag_start_viewport.y_range();
                        self.viewport.x_min = self.drag_start_viewport.x_min - dx;
                        self.viewport.x_max = self.drag_start_viewport.x_max - dx;
                        self.viewport.y_min = self.drag_start_viewport.y_min + dy;
                        self.viewport.y_max = self.drag_start_viewport.y_max + dy;
                    }
                    self.redraw(cx);
                }
            }
            Hit::FingerUp(_) => {
                self.drag_start_abs = None;
                cx.set_cursor(MouseCursor::Grab);
            }
            Hit::FingerHoverIn(_) => {
                cx.set_cursor(MouseCursor::Grab);
            }
            Hit::FingerScroll(fs) => {
                let scroll = if fs.scroll.y.abs() > f64::EPSILON {
                    fs.scroll.y
                } else {
                    fs.scroll.x
                };
                self.zoom_at(cx, scroll, fs.abs);
            }
            _ => {}
        }
    }

    fn zoom_at(&mut self, cx: &mut Cx, scroll: f64, abs: DVec2) {
        let factor = if scroll > 0.0 { 0.9 } else { 1.0 / 0.9 };
        let pr = &self.plot_rect;
        if pr.size.x <= 0.0 || pr.size.y <= 0.0 {
            return;
        }
        let frac_x = ((abs.x - pr.pos.x) / pr.size.x).clamp(0.0, 1.0);
        let frac_y = 1.0 - ((abs.y - pr.pos.y) / pr.size.y).clamp(0.0, 1.0);
        let data_x = self.viewport.x_min + frac_x * self.viewport.x_range();
        let data_y = self.viewport.y_min + frac_y * self.viewport.y_range();
        let new_x_range = self.viewport.x_range() * factor;
        let new_y_range = self.viewport.y_range() * factor;
        self.viewport.x_min = data_x - frac_x * new_x_range;
        self.viewport.x_max = data_x + (1.0 - frac_x) * new_x_range;
        self.viewport.y_min = data_y - frac_y * new_y_range;
        self.viewport.y_max = data_y + (1.0 - frac_y) * new_y_range;
        self.redraw(cx);
    }

    // ---- Session ----

    fn compute_plot_rect(&mut self) {
        let m = &self.plot_margin;
        self.plot_rect = Rect {
            pos: DVec2 {
                x: self.rect.pos.x + m.left,
                y: self.rect.pos.y + m.top,
            },
            size: DVec2 {
                x: (self.rect.size.x - m.left - m.right).max(1.0),
                y: (self.rect.size.y - m.top - m.bottom).max(1.0),
            },
        };
    }

    /// Begin a drawing session: draws the background and opens the vector layer.
    pub fn begin(&mut self, cx: &mut Cx2d, walk: Walk) {
        self.rect = cx.walk_turtle(walk);
        self.compute_plot_rect();
        self.draw_bg.draw_abs(cx, self.rect);
        cx.begin_turtle(
            Walk {
                abs_pos: Some(self.rect.pos),
                width: Size::Fixed(self.rect.size.x),
                height: Size::Fixed(self.rect.size.y),
                margin: Inset::default(),
                metrics: Metrics::default(),
                ..Walk::default()
            },
            Layout {
                clip_x: self.clip_plot,
                clip_y: self.clip_plot,
                padding: self.plot_margin,
                ..Layout::default()
            },
        );
        self.draw_vector.begin();
        self.dash_offset = 0.0;
        self.text_queue.clear();
    }

    /// End the drawing session: flushes the vector layer, then draws all queued
    /// text on top of it.
    pub fn end(&mut self, cx: &mut Cx2d) {
        self.draw_vector.end(cx);
        let queue = std::mem::take(&mut self.text_queue);
        for q in queue {
            self.draw_text.color = q.color;
            self.draw_text.text_style.font_size = q.size;
            self.draw_text.draw_abs(cx, dvec2(q.x, q.y), &q.text);
        }
        cx.end_turtle();
    }

    // ---- Viewport management ----

    pub fn set_viewport(&mut self, x_min: f64, x_max: f64, y_min: f64, y_max: f64) {
        self.viewport = PlotViewport {
            x_min,
            x_max,
            y_min,
            y_max,
        };
    }

    /// Fit the viewport to raw (untransformed) data ranges, with padding.
    /// Applies the axis scale transforms.
    pub fn fit_data(&mut self, x_min: f64, x_max: f64, y_min: f64, y_max: f64) {
        let tx_min = self.x_scale.transform(x_min);
        let tx_max = self.x_scale.transform(x_max);
        let ty_min = self.y_scale.transform(y_min);
        let ty_max = self.y_scale.transform(y_max);
        let x_pad = (tx_max - tx_min).abs().max(1e-9) * self.data_padding;
        let y_pad = (ty_max - ty_min).abs().max(1e-9) * self.data_padding;
        self.viewport = PlotViewport {
            x_min: tx_min - x_pad,
            x_max: tx_max + x_pad,
            y_min: ty_min - y_pad,
            y_max: ty_max + y_pad,
        };
    }

    /// Fit the viewport to a set of series (uses raw data, applies scales).
    pub fn fit_series(&mut self, series: &[Series]) {
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        for s in series {
            for &x in &s.x {
                if x < x_min {
                    x_min = x;
                }
                if x > x_max {
                    x_max = x;
                }
            }
            for &y in &s.y {
                if y < y_min {
                    y_min = y;
                }
                if y > y_max {
                    y_max = y;
                }
            }
        }
        if !x_min.is_finite() || !y_min.is_finite() {
            return;
        }
        if x_min == x_max {
            x_min -= 0.5;
            x_max += 0.5;
        }
        if y_min == y_max {
            y_min -= 0.5;
            y_max += 0.5;
        }
        self.fit_data(x_min, x_max, y_min, y_max);
    }

    // ---- Coordinate transforms ----

    /// Raw data coordinates → pixels (applies axis scale transforms)
    pub fn data_to_px(&self, x: f64, y: f64) -> (f32, f32) {
        let tx = self.x_scale.transform(x);
        let ty = self.y_scale.transform(y);
        self.tdata_to_px(tx, ty)
    }

    /// Transformed data coordinates → pixels
    pub fn tdata_to_px(&self, tx: f64, ty: f64) -> (f32, f32) {
        let pr = &self.plot_rect;
        let vp = &self.viewport;
        let xr = if vp.x_range().abs() < 1e-12 { 1.0 } else { vp.x_range() };
        let yr = if vp.y_range().abs() < 1e-12 { 1.0 } else { vp.y_range() };
        let px = pr.pos.x + (tx - vp.x_min) / xr * pr.size.x;
        let py = pr.pos.y + (1.0 - (ty - vp.y_min) / yr) * pr.size.y;
        (px as f32, py as f32)
    }

    /// Pixels → raw data coordinates (inverts axis scale transforms)
    pub fn px_to_data(&self, px: f32, py: f32) -> (f64, f64) {
        let pr = &self.plot_rect;
        let vp = &self.viewport;
        let tx = vp.x_min + (px as f64 - pr.pos.x) / pr.size.x * vp.x_range();
        let ty = vp.y_min + (1.0 - (py as f64 - pr.pos.y) / pr.size.y) * vp.y_range();
        (self.x_scale.inverse(tx), self.y_scale.inverse(ty))
    }

    pub fn plot_rect(&self) -> &Rect {
        &self.plot_rect
    }

    pub fn viewport(&self) -> &PlotViewport {
        &self.viewport
    }

    // ---- Axes / grid / decorations ----

    /// Draw grid lines, tick labels, border, title and axis labels.
    pub fn draw_axes(&mut self, cx: &mut Cx2d) {
        let vp = self.viewport.clone();

        if self.show_grid || self.show_ticks {
            // X ticks: generate in raw space from the visible (transformed) range
            let raw_x_min = self.x_scale.inverse(vp.x_min);
            let raw_x_max = self.x_scale.inverse(vp.x_max);
            let x_ticks = match self.x_scale {
                ScaleType::Linear => nice_ticks(vp.x_min, vp.x_max, 8),
                _ => self
                    .x_scale
                    .generate_ticks(raw_x_min.min(raw_x_max), raw_x_max.max(raw_x_min), 8)
                    .iter()
                    .map(|v| self.x_scale.transform(*v))
                    .collect(),
            };
            for &tx in &x_ticks {
                let (px, _) = self.tdata_to_px(tx, 0.0);
                if self.show_grid {
                    self.draw_grid_line_v_px(cx, px as f64);
                }
                if self.show_ticks {
                    let raw = self.x_scale.inverse(tx);
                    let label = self.format_x_tick(raw);
                    let est_w = label.len() as f64 * self.tick_font_size as f64 * 0.5;
                    let color = self.text_color_or_default();
                    let size = self.tick_font_size;
                    self.draw_text_px(
                        cx,
                        px as f64 - est_w * 0.5,
                        self.plot_rect.pos.y + self.plot_rect.size.y + 4.0,
                        &label,
                        color,
                        size,
                    );
                }
            }

            // Y ticks
            let raw_y_min = self.y_scale.inverse(vp.y_min);
            let raw_y_max = self.y_scale.inverse(vp.y_max);
            let y_ticks = match self.y_scale {
                ScaleType::Linear => nice_ticks(vp.y_min, vp.y_max, 6),
                _ => self
                    .y_scale
                    .generate_ticks(raw_y_min.min(raw_y_max), raw_y_max.max(raw_y_min), 6)
                    .iter()
                    .map(|v| self.y_scale.transform(*v))
                    .collect(),
            };
            for &ty in &y_ticks {
                let (_, py) = self.tdata_to_px(0.0, ty);
                if self.show_grid {
                    self.draw_grid_line_h_px(cx, py as f64);
                }
                if self.show_ticks {
                    let raw = self.y_scale.inverse(ty);
                    let label = self.format_y_tick(raw);
                    let est_w = label.len() as f64 * self.tick_font_size as f64 * 0.55;
                    let color = self.text_color_or_default();
                    let size = self.tick_font_size;
                    self.draw_text_px(
                        cx,
                        (self.plot_rect.pos.x - est_w - 6.0).max(self.rect.pos.x),
                        py as f64 - self.tick_font_size as f64 * 0.6,
                        &label,
                        color,
                        size,
                    );
                }
            }
        }

        if self.show_border {
            self.draw_plot_border(cx);
        }

        // Title (centered top)
        if !self.title.is_empty() {
            let title = self.title.clone();
            let est_w = title.len() as f64 * self.title_font_size as f64 * 0.5;
            let color = self.text_color_or_default();
            let size = self.title_font_size;
            self.draw_text_px(
                cx,
                self.plot_rect.pos.x + self.plot_rect.size.x * 0.5 - est_w * 0.5,
                self.rect.pos.y + 4.0,
                &title,
                color,
                size,
            );
        }

        // X label (centered bottom)
        if !self.xlabel.is_empty() {
            let xlabel = self.xlabel.clone();
            let est_w = xlabel.len() as f64 * self.label_font_size as f64 * 0.5;
            let color = self.text_color_or_default();
            let size = self.label_font_size;
            self.draw_text_px(
                cx,
                self.plot_rect.pos.x + self.plot_rect.size.x * 0.5 - est_w * 0.5,
                self.rect.pos.y + self.rect.size.y - self.label_font_size as f64 - 6.0,
                &xlabel,
                color,
                size,
            );
        }

        // Y label (top-left, horizontal)
        if !self.ylabel.is_empty() {
            let ylabel = self.ylabel.clone();
            let color = self.text_color_or_default();
            let size = self.label_font_size;
            self.draw_text_px(
                cx,
                self.rect.pos.x + 4.0,
                self.rect.pos.y + 4.0,
                &ylabel,
                color,
                size,
            );
        }
    }

    fn format_x_tick(&self, v: f64) -> String {
        format_tick_value(self.x_scale, v)
    }

    fn format_y_tick(&self, v: f64) -> String {
        format_tick_value(self.y_scale, v)
    }

    fn text_color_or_default(&self) -> Vec4 {
        if self.text_color.w > 0.0 {
            self.text_color
        } else {
            vec4(0.2, 0.2, 0.2, 1.0)
        }
    }

    fn grid_color_or_default(&self) -> Vec4 {
        if self.grid_color.w > 0.0 {
            self.grid_color
        } else {
            vec4(0.88, 0.88, 0.88, 1.0)
        }
    }

    fn border_color_or_default(&self) -> Vec4 {
        if self.border_color.w > 0.0 {
            self.border_color
        } else {
            vec4(0.6, 0.6, 0.6, 1.0)
        }
    }

    pub fn draw_grid_line_h_px(&mut self, cx: &mut Cx2d, py: f64) {
        let pr = &self.plot_rect;
        if py < pr.pos.y || py > pr.pos.y + pr.size.y {
            return;
        }
        self.draw_grid.color = self.grid_color_or_default();
        self.draw_grid.draw_abs(
            cx,
            Rect {
                pos: DVec2 { x: pr.pos.x, y: py },
                size: DVec2 { x: pr.size.x, y: 1.0 },
            },
        );
    }

    pub fn draw_grid_line_v_px(&mut self, cx: &mut Cx2d, px: f64) {
        let pr = &self.plot_rect;
        if px < pr.pos.x || px > pr.pos.x + pr.size.x {
            return;
        }
        self.draw_grid.color = self.grid_color_or_default();
        self.draw_grid.draw_abs(
            cx,
            Rect {
                pos: DVec2 { x: px, y: pr.pos.y },
                size: DVec2 { x: 1.0, y: pr.size.y },
            },
        );
    }

    pub fn draw_plot_border(&mut self, cx: &mut Cx2d) {
        let pr = self.plot_rect;
        self.draw_grid.color = self.border_color_or_default();
        for r in [
            Rect { pos: pr.pos, size: DVec2 { x: pr.size.x, y: 1.0 } },
            Rect {
                pos: DVec2 { x: pr.pos.x, y: pr.pos.y + pr.size.y },
                size: DVec2 { x: pr.size.x, y: 1.0 },
            },
            Rect { pos: pr.pos, size: DVec2 { x: 1.0, y: pr.size.y } },
            Rect {
                pos: DVec2 { x: pr.pos.x + pr.size.x, y: pr.pos.y },
                size: DVec2 { x: 1.0, y: pr.size.y },
            },
        ] {
            self.draw_grid.draw_abs(cx, r);
        }
    }

    // ---- Vector drawing helpers (pixel space) ----

    pub fn set_color(&mut self, color: Vec4) {
        self.draw_vector.set_color(color.x, color.y, color.z, color.w);
    }

    /// Straight solid line in pixel space
    pub fn line_px(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, width: f32) {
        self.draw_vector.move_to(x1, y1);
        self.draw_vector.line_to(x2, y2);
        self.draw_vector.stroke(width);
    }

    /// Styled line in pixel space (dash marching for non-solid styles)
    pub fn line_styled_px(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        width: f32,
        style: LineStyle,
    ) {
        match style {
            LineStyle::Solid => self.line_px(x1, y1, x2, y2, width),
            _ => {
                let pattern: &[f32] = match style {
                    LineStyle::Dashed => &[8.0, 5.0],
                    LineStyle::Dotted => &[1.5, 4.0],
                    LineStyle::DashDot => &[8.0, 4.0, 1.5, 4.0],
                    LineStyle::Solid => unreachable!(),
                };
                let dx = x2 - x1;
                let dy = y2 - y1;
                let len = (dx * dx + dy * dy).sqrt();
                if len < 1e-6 {
                    return;
                }
                let ux = dx / len;
                let uy = dy / len;
                let mut t = 0.0f32;
                let mut i = 0usize;
                while t < len {
                    let seg = pattern[i % pattern.len()];
                    let is_draw = i % 2 == 0;
                    let end = (t + seg).min(len);
                    if is_draw {
                        self.draw_vector.move_to(x1 + ux * t, y1 + uy * t);
                        self.draw_vector.line_to(x1 + ux * end, y1 + uy * end);
                        self.draw_vector.stroke(width);
                    }
                    t = end;
                    i += 1;
                }
            }
        }
    }

    /// Styled polyline through data-space points
    pub fn draw_polyline_data(
        &mut self,
        xs: &[f64],
        ys: &[f64],
        color: Vec4,
        width: f32,
        style: LineStyle,
    ) {
        if xs.len() < 2 || ys.len() < 2 {
            return;
        }
        self.set_color(color);
        let n = xs.len().min(ys.len());
        match style {
            LineStyle::Solid => {
                let (px, py) = self.data_to_px(xs[0], ys[0]);
                self.draw_vector.move_to(px, py);
                for i in 1..n {
                    let (px, py) = self.data_to_px(xs[i], ys[i]);
                    self.draw_vector.line_to(px, py);
                }
                self.draw_vector.stroke(width);
            }
            _ => {
                for i in 1..n {
                    let (px1, py1) = self.data_to_px(xs[i - 1], ys[i - 1]);
                    let (px2, py2) = self.data_to_px(xs[i], ys[i]);
                    self.line_styled_px(px1, py1, px2, py2, width, style);
                }
            }
        }
    }

    /// Line between two data-space points
    pub fn draw_line_data(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        color: Vec4,
        width: f32,
        style: LineStyle,
    ) {
        self.set_color(color);
        let (px1, py1) = self.data_to_px(x1, y1);
        let (px2, py2) = self.data_to_px(x2, y2);
        self.line_styled_px(px1, py1, px2, py2, width, style);
    }

    /// Filled polygon from data-space points
    pub fn fill_polygon_data(&mut self, xs: &[f64], ys: &[f64], color: Vec4) {
        let n = xs.len().min(ys.len());
        if n < 3 {
            return;
        }
        self.set_color(color);
        let (px, py) = self.data_to_px(xs[0], ys[0]);
        self.draw_vector.move_to(px, py);
        for i in 1..n {
            let (px, py) = self.data_to_px(xs[i], ys[i]);
            self.draw_vector.line_to(px, py);
        }
        self.draw_vector.close();
        self.draw_vector.fill();
    }

    /// Filled polygon in pixel space
    pub fn fill_polygon_gradient(&mut self, xs: &[f64], ys: &[f64], color: Vec4) {
        use makepad_widgets::makepad_draw::vector::{GradientStop, VectorPaint};
        let count=xs.len().min(ys.len());
        if count<3 {return;}
        let top=self.plot_rect.pos.y as f32;
        let bottom=(self.plot_rect.pos.y+self.plot_rect.size.y) as f32;
        let stops=vec![
            GradientStop{offset:0.,color:[color.x*color.w,color.y*color.w,color.z*color.w,color.w],straight_rgb:None},
            GradientStop{offset:1.,color:[0.;4],straight_rgb:None},
        ];
        self.draw_vector.cur_gradient_row_v=self.draw_vector.add_gradient_row(&stops);
        self.draw_vector.set_paint(VectorPaint::linear_gradient(0.,top,0.,bottom,stops));
        let (px,py)=self.data_to_px(xs[0],ys[0]);
        self.draw_vector.move_to(px,py);
        for i in 1..count {
            let (px,py)=self.data_to_px(xs[i],ys[i]);
            self.draw_vector.line_to(px,py);
        }
        self.draw_vector.close();
        self.draw_vector.fill();
        self.draw_vector.cur_gradient_row_v=-1.0;
    }

    /// Filled polygon in pixel space
    pub fn fill_polygon_px(&mut self, pts: &[(f32, f32)], color: Vec4) {
        if pts.len() < 3 {
            return;
        }
        self.set_color(color);
        self.draw_vector.move_to(pts[0].0, pts[0].1);
        for p in &pts[1..] {
            self.draw_vector.line_to(p.0, p.1);
        }
        self.draw_vector.close();
        self.draw_vector.fill();
    }

    /// Stroked polygon outline in pixel space
    pub fn stroke_polygon_px(&mut self, pts: &[(f32, f32)], color: Vec4, width: f32) {
        if pts.len() < 2 {
            return;
        }
        self.set_color(color);
        self.draw_vector.move_to(pts[0].0, pts[0].1);
        for p in &pts[1..] {
            self.draw_vector.line_to(p.0, p.1);
        }
        self.draw_vector.close();
        self.draw_vector.stroke(width);
    }

    /// Axis-aligned filled rectangle between two data-space corners
    pub fn fill_rect_data(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, color: Vec4) {
        let (px1, py1) = self.data_to_px(x1, y1);
        let (px2, py2) = self.data_to_px(x2, y2);
        let x = px1.min(px2);
        let y = py1.min(py2);
        let w = (px2 - px1).abs().max(1.0);
        let h = (py2 - py1).abs().max(1.0);
        self.set_color(color);
        self.draw_vector.rect(x, y, w, h);
        self.draw_vector.fill();
    }

    /// Filled rectangle in pixel space
    pub fn fill_rect_px(&mut self, x: f32, y: f32, w: f32, h: f32, color: Vec4) {
        self.set_color(color);
        self.draw_vector.rect(x, y, w, h);
        self.draw_vector.fill();
    }

    /// Filled circle in pixel space
    pub fn fill_circle_px(&mut self, cx_px: f32, cy_px: f32, r: f32, color: Vec4) {
        self.set_color(color);
        self.draw_vector.circle(cx_px, cy_px, r);
        self.draw_vector.fill();
    }

    /// Circle outline in pixel space
    pub fn stroke_circle_px(&mut self, cx_px: f32, cy_px: f32, r: f32, color: Vec4, width: f32) {
        self.set_color(color);
        self.draw_vector.circle(cx_px, cy_px, r);
        self.draw_vector.stroke(width);
    }

    /// Marker at a data-space point
    pub fn draw_marker_data(
        &mut self,
        x: f64,
        y: f64,
        size: f32,
        style: MarkerStyle,
        color: Vec4,
    ) {
        let (px, py) = self.data_to_px(x, y);
        self.draw_marker_px(px, py, size, style, color);
    }

    /// Marker at a pixel-space point. `size` is the marker radius in px.
    pub fn draw_marker_px(
        &mut self,
        px: f32,
        py: f32,
        size: f32,
        style: MarkerStyle,
        color: Vec4,
    ) {
        let r = size;
        self.set_color(color);
        match style {
            MarkerStyle::None => {}
            MarkerStyle::Circle => {
                self.draw_vector.circle(px, py, r);
                self.draw_vector.fill();
            }
            MarkerStyle::Square => {
                self.draw_vector.rect(px - r, py - r, r * 2.0, r * 2.0);
                self.draw_vector.fill();
            }
            MarkerStyle::TriangleUp => {
                self.draw_vector.move_to(px, py - r);
                self.draw_vector.line_to(px + r, py + r);
                self.draw_vector.line_to(px - r, py + r);
                self.draw_vector.close();
                self.draw_vector.fill();
            }
            MarkerStyle::TriangleDown => {
                self.draw_vector.move_to(px, py + r);
                self.draw_vector.line_to(px + r, py - r);
                self.draw_vector.line_to(px - r, py - r);
                self.draw_vector.close();
                self.draw_vector.fill();
            }
            MarkerStyle::Diamond => {
                self.draw_vector.move_to(px, py - r);
                self.draw_vector.line_to(px + r, py);
                self.draw_vector.line_to(px, py + r);
                self.draw_vector.line_to(px - r, py);
                self.draw_vector.close();
                self.draw_vector.fill();
            }
            MarkerStyle::Cross => {
                let d = r * 0.7071;
                self.draw_vector.move_to(px - d, py - d);
                self.draw_vector.line_to(px + d, py + d);
                self.draw_vector.stroke(r * 0.4);
                self.draw_vector.move_to(px - d, py + d);
                self.draw_vector.line_to(px + d, py - d);
                self.draw_vector.stroke(r * 0.4);
            }
            MarkerStyle::Plus => {
                self.draw_vector.move_to(px - r, py);
                self.draw_vector.line_to(px + r, py);
                self.draw_vector.stroke(r * 0.4);
                self.draw_vector.move_to(px, py - r);
                self.draw_vector.line_to(px, py + r);
                self.draw_vector.stroke(r * 0.4);
            }
            MarkerStyle::Star => {
                // 5-point star
                let mut first = true;
                for i in 0..10 {
                    let angle = -std::f32::consts::FRAC_PI_2
                        + i as f32 * std::f32::consts::PI / 5.0;
                    let radius = if i % 2 == 0 { r } else { r * 0.45 };
                    let sx = px + radius * angle.cos();
                    let sy = py + radius * angle.sin();
                    if first {
                        self.draw_vector.move_to(sx, sy);
                        first = false;
                    } else {
                        self.draw_vector.line_to(sx, sy);
                    }
                }
                self.draw_vector.close();
                self.draw_vector.fill();
            }
        }
    }

    /// Filled pie slice / arc sector in pixel space, angles in radians.
    /// `inner_ratio` of 0.0 draws a full slice; > 0.0 draws a donut segment.
    pub fn fill_arc_px(
        &mut self,
        cx_px: f32,
        cy_px: f32,
        radius: f32,
        inner_ratio: f32,
        start_angle: f32,
        end_angle: f32,
        color: Vec4,
    ) {
        let span = end_angle - start_angle;
        if span.abs() < 1e-6 {
            return;
        }
        let steps = ((span.abs() / 0.06).ceil() as usize).clamp(2, 256);
        self.set_color(color);
        let inner_r = radius * inner_ratio;
        // Outer arc forward
        for i in 0..=steps {
            let a = start_angle + span * (i as f32 / steps as f32);
            let x = cx_px + radius * a.cos();
            let y = cy_px + radius * a.sin();
            if i == 0 {
                self.draw_vector.move_to(x, y);
            } else {
                self.draw_vector.line_to(x, y);
            }
        }
        if inner_r > 0.5 {
            // Inner arc backward
            for i in (0..=steps).rev() {
                let a = start_angle + span * (i as f32 / steps as f32);
                let x = cx_px + inner_r * a.cos();
                let y = cy_px + inner_r * a.sin();
                self.draw_vector.line_to(x, y);
            }
        } else {
            self.draw_vector.line_to(cx_px, cy_px);
        }
        self.draw_vector.close();
        self.draw_vector.fill();
    }

    /// Stroked arc (polyline approximation) in pixel space
    pub fn stroke_arc_px(
        &mut self,
        cx_px: f32,
        cy_px: f32,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        color: Vec4,
        width: f32,
    ) {
        let span = end_angle - start_angle;
        if span.abs() < 1e-6 {
            return;
        }
        let steps = ((span.abs() / 0.06).ceil() as usize).clamp(2, 256);
        self.set_color(color);
        for i in 0..=steps {
            let a = start_angle + span * (i as f32 / steps as f32);
            let x = cx_px + radius * a.cos();
            let y = cy_px + radius * a.sin();
            if i == 0 {
                self.draw_vector.move_to(x, y);
            } else {
                self.draw_vector.line_to(x, y);
            }
        }
        self.draw_vector.stroke(width);
    }

    /// Arrow between two data-space points
    pub fn draw_arrow_data(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        color: Vec4,
        width: f32,
        head_size: f32,
    ) {
        let (px1, py1) = self.data_to_px(x1, y1);
        let (px2, py2) = self.data_to_px(x2, y2);
        self.set_color(color);
        self.line_px(px1, py1, px2, py2, width);
        // Arrow head
        let dx = px2 - px1;
        let dy = py2 - py1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1e-6 {
            return;
        }
        let ux = dx / len;
        let uy = dy / len;
        let bx = px2 - ux * head_size;
        let by = py2 - uy * head_size;
        let wx = -uy * head_size * 0.5;
        let wy = ux * head_size * 0.5;
        self.draw_vector.move_to(px2, py2);
        self.draw_vector.line_to(bx + wx, by + wy);
        self.draw_vector.line_to(bx - wx, by - wy);
        self.draw_vector.close();
        self.draw_vector.fill();
    }

    /// Horizontal reference line across the plot at data-space y
    pub fn draw_hline(&mut self, y: f64, color: Vec4, width: f32, style: LineStyle) {
        let ty = self.y_scale.transform(y);
        let (_, py) = self.tdata_to_px(0.0, ty);
        let pr = self.plot_rect;
        self.set_color(color);
        self.line_styled_px(
            pr.pos.x as f32,
            py,
            (pr.pos.x + pr.size.x) as f32,
            py,
            width,
            style,
        );
    }

    /// Vertical reference line across the plot at data-space x
    pub fn draw_vline(&mut self, x: f64, color: Vec4, width: f32, style: LineStyle) {
        let tx = self.x_scale.transform(x);
        let (px, _) = self.tdata_to_px(tx, 0.0);
        let pr = self.plot_rect;
        self.set_color(color);
        self.line_styled_px(
            px,
            pr.pos.y as f32,
            px,
            (pr.pos.y + pr.size.y) as f32,
            width,
            style,
        );
    }

    // ---- Text ----

    /// Draw text at an absolute pixel position with color and size.
    /// Queued and rendered above the vector layer when `end()` runs.
    pub fn draw_text_px(&mut self, _cx: &mut Cx2d, x: f64, y: f64, text: &str, color: Vec4, size: f32) {
        self.text_queue.push(QueuedText {
            x,
            y,
            text: text.to_string(),
            color,
            size,
        });
    }

    /// Draw text roughly centered at an absolute pixel position
    pub fn draw_text_centered_px(
        &mut self,
        cx: &mut Cx2d,
        x: f64,
        y: f64,
        text: &str,
        color: Vec4,
        size: f32,
    ) {
        let est_w = text.len() as f64 * size as f64 * 0.5;
        self.draw_text_px(cx, x - est_w * 0.5, y - size as f64 * 0.6, text, color, size);
    }

    /// Draw text at a data-space position
    pub fn draw_text_data(
        &mut self,
        cx: &mut Cx2d,
        x: f64,
        y: f64,
        text: &str,
        color: Vec4,
        size: f32,
    ) {
        let (px, py) = self.data_to_px(x, y);
        self.draw_text_px(cx, px as f64, py as f64, text, color, size);
    }

    // ---- Legend ----

    /// Draw a legend box with colored swatches. Call after chart contents.
    pub fn draw_legend(&mut self, cx: &mut Cx2d, entries: &[(String, Vec4)]) {
        if entries.is_empty() || self.legend == LegendPosition::None {
            return;
        }
        let font_size = self.tick_font_size.max(9.0);
        let row_h = font_size as f64 + 6.0;
        let swatch = 10.0f64;
        let pad = 8.0f64;
        let max_label_len = entries.iter().map(|(l, _)| l.len()).max().unwrap_or(0);
        let box_w = pad * 2.0 + swatch + 6.0 + max_label_len as f64 * font_size as f64 * 0.55;
        let box_h = pad * 2.0 + row_h * entries.len() as f64;

        let pr = self.plot_rect;
        let (bx, by) = match self.legend {
            LegendPosition::TopRight => (pr.pos.x + pr.size.x - box_w - 8.0, pr.pos.y + 8.0),
            LegendPosition::TopLeft => (pr.pos.x + 8.0, pr.pos.y + 8.0),
            LegendPosition::BottomRight => (
                pr.pos.x + pr.size.x - box_w - 8.0,
                pr.pos.y + pr.size.y - box_h - 8.0,
            ),
            LegendPosition::BottomLeft => (pr.pos.x + 8.0, pr.pos.y + pr.size.y - box_h - 8.0),
            LegendPosition::None => return,
        };

        // Background + border
        self.fill_rect_px(bx as f32, by as f32, box_w as f32, box_h as f32, vec4(1.0, 1.0, 1.0, 0.85));
        self.set_color(self.border_color_or_default());
        self.draw_vector.rect(bx as f32, by as f32, box_w as f32, box_h as f32);
        self.draw_vector.stroke(1.0);

        let text_color = self.text_color_or_default();
        for (i, (label, color)) in entries.iter().enumerate() {
            let row_y = by + pad + i as f64 * row_h;
            self.fill_rect_px(
                (bx + pad) as f32,
                (row_y + row_h * 0.5 - swatch * 0.5) as f32,
                swatch as f32,
                swatch as f32,
                *color,
            );
            self.draw_text_px(
                cx,
                bx + pad + swatch + 6.0,
                row_y + row_h * 0.5 - font_size as f64 * 0.7,
                label,
                text_color,
                font_size,
            );
        }
    }
}

/// "Nice" tick generation for linear axes (1/2/5 steps)
pub fn nice_ticks(min: f64, max: f64, target_count: usize) -> Vec<f64> {
    let range = max - min;
    if range <= 0.0 || !range.is_finite() {
        return vec![];
    }
    let rough_step = range / target_count as f64;
    let mag = 10.0_f64.powf(rough_step.log10().floor());
    let norm = rough_step / mag;
    let nice_step = if norm <= 1.5 {
        1.0
    } else if norm <= 3.5 {
        2.0
    } else if norm <= 7.5 {
        5.0
    } else {
        10.0
    } * mag;

    let start = (min / nice_step).ceil() * nice_step;
    let mut ticks = Vec::new();
    let mut v = start;
    while v <= max {
        ticks.push(v);
        v += nice_step;
        if ticks.len() > 100 {
            break;
        }
    }
    ticks
}

/// Format a tick value with sensible precision
pub fn format_tick_value(scale: ScaleType, v: f64) -> String {
    match scale {
        ScaleType::Linear => {
            let mut v = v;
            if v.abs() < 1e-9 {
                v = 0.0;
            }
            let a = v.abs();
            if a >= 1e6 || (a > 0.0 && a < 1e-3) {
                format!("{:.1e}", v)
            } else if a >= 100.0 {
                format!("{:.0}", v)
            } else if a >= 1.0 {
                let s = format!("{:.1}", v);
                if s.ends_with(".0") {
                    s[..s.len() - 2].to_string()
                } else {
                    s
                }
            } else {
                format!("{:.2}", v)
            }
        }
        _ => scale.format_tick(v),
    }
}
