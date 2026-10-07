// 3D charts: View3D projection helper, Surface3D, Scatter3D, Line3D
//
// Ported from the Makepad 1.0 plot library to Makepad 2.0 / Octoscript.
// These are non-cartesian widgets: the PlotView is used only as a canvas
// (background, title via draw_axes, pixel-space vector helpers) and all 3D
// geometry is projected manually into plot_rect pixel space.

use crate::plot_view::*;
use crate::script_util::*;
use crate::types::*;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*

    mod.plot.Surface3DBase = #(Surface3D::register_widget(vm))

    mod.plot.Surface3D = set_type_default() do mod.plot.Surface3DBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 8.0, top: 28.0, right: 8.0, bottom: 8.0}
        show_grid: false
        show_ticks: false
        show_border: false
        colormap: "Viridis"
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }

    mod.plot.Scatter3DBase = #(Scatter3D::register_widget(vm))

    mod.plot.Scatter3D = set_type_default() do mod.plot.Scatter3DBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 8.0, top: 28.0, right: 8.0, bottom: 8.0}
        show_grid: false
        show_ticks: false
        show_border: false
        colormap: "Viridis"
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }

    mod.plot.Line3DBase = #(Line3D::register_widget(vm))

    mod.plot.Line3D = set_type_default() do mod.plot.Line3DBase{
        width: Fill
        height: Fill
        plot_margin: Inset{left: 8.0, top: 28.0, right: 8.0, bottom: 8.0}
        show_grid: false
        show_ticks: false
        show_border: false
        draw_bg +: { draw_depth: 0.0, color: #xffffff }
        draw_grid +: { draw_depth: 0.0001, color: #xe4e4e4 }
        draw_vector +: { draw_depth: 0.0002 }
        draw_text +: { draw_depth: 0.0003, color: #x333333, text_style: theme.font_regular{} }
    }
}

// =============================================================================
// View3D - azimuth/elevation 3D -> 2D projection helper
// =============================================================================

#[derive(Clone, Debug)]
pub struct View3D {
    pub azimuth: f64,   // Horizontal rotation (degrees)
    pub elevation: f64, // Vertical rotation (degrees)
    pub distance: f64,  // Distance from origin (perspective)
}

impl Default for View3D {
    fn default() -> Self {
        Self {
            azimuth: -60.0,
            elevation: 30.0,
            distance: 3.0,
        }
    }
}

impl View3D {
    pub fn new() -> Self {
        Self::default()
    }

    /// Project a normalized 3D point ([-1,1] cube) to 2D view coordinates
    pub fn project(&self, x: f64, y: f64, z: f64) -> (f64, f64) {
        let az = self.azimuth.to_radians();
        let el = self.elevation.to_radians();

        // Rotate around Z axis (azimuth)
        let x1 = x * az.cos() - y * az.sin();
        let y1 = x * az.sin() + y * az.cos();
        let z1 = z;

        // Rotate around X axis (elevation)
        let x2 = x1;
        let y2 = y1 * el.cos() - z1 * el.sin();
        let z2 = y1 * el.sin() + z1 * el.cos();

        // Simple perspective projection
        let perspective = self.distance / (self.distance + y2 + 2.0);
        (x2 * perspective, z2 * perspective)
    }

    /// Depth for painter's-algorithm sorting (larger = further away)
    pub fn depth(&self, x: f64, y: f64, z: f64) -> f64 {
        let az = self.azimuth.to_radians();
        let el = self.elevation.to_radians();
        let y1 = x * az.sin() + y * az.cos();
        y1 * el.cos() - z * el.sin()
    }

    /// Project into pixel space around a screen center with a pixel scale
    pub fn project_px(
        &self,
        x: f64,
        y: f64,
        z: f64,
        cx_center: f64,
        cy_center: f64,
        scale: f64,
    ) -> (f32, f32) {
        let (sx, sy) = self.project(x, y, z);
        ((cx_center + sx * scale) as f32, (cy_center - sy * scale) as f32)
    }
}

/// Shared drag-to-rotate / scroll-to-zoom interaction for the 3D widgets.
/// Returns true when the widget needs a redraw.
fn handle_rotate_event(
    cx: &mut Cx,
    event: &Event,
    area: Area,
    view3d: &mut View3D,
    zoom: &mut f64,
    drag_start: &mut Option<DVec2>,
    start_azimuth: &mut f64,
    start_elevation: &mut f64,
) -> bool {
    match event.hits_with_capture_overload(cx, area, true) {
        Hit::FingerDown(fe) if fe.is_primary_hit() => {
            *drag_start = Some(fe.abs);
            *start_azimuth = view3d.azimuth;
            *start_elevation = view3d.elevation;
            false
        }
        Hit::FingerMove(fe) => {
            if let Some(start) = *drag_start {
                let delta = fe.abs - start;
                // Horizontal drag changes azimuth, vertical changes elevation
                view3d.azimuth = *start_azimuth + delta.x * 0.5;
                view3d.elevation = (*start_elevation - delta.y * 0.5).clamp(-89.0, 89.0);
                true
            } else {
                false
            }
        }
        Hit::FingerUp(_) => {
            *drag_start = None;
            false
        }
        Hit::FingerScroll(fs) => {
            if *zoom == 0.0 {
                *zoom = 1.0;
            }
            let zoom_delta = 1.0 + fs.scroll.y * 0.001;
            // DrawVector geometry is not clipped to the widget cell by the engine,
            // so keep zoom modest to avoid heavy spill into neighboring widgets.
            *zoom = (*zoom * zoom_delta).clamp(0.5, 1.6);
            true
        }
        _ => false,
    }
}

const AXIS_COLOR: Vec4 = Vec4 { x: 0.5, y: 0.5, z: 0.5, w: 0.8 };

// =============================================================================
// Surface3D - 3D surface from a z matrix, colored by height
// =============================================================================

#[derive(Script, ScriptHook, Widget)]
pub struct Surface3D {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    z_data: Vec<Vec<f64>>,
    #[rust]
    z_range: (f64, f64),
    #[rust]
    view3d: View3D,

    #[live]
    pub colormap: String,
    #[live(true)]
    pub show_surface: bool,
    #[live(true)]
    pub show_wireframe: bool,
    #[live(1.0)]
    pub zoom: f64,
    #[live(true)]
    pub demo_data: bool,

    #[rust]
    drag_start: Option<DVec2>,
    #[rust]
    start_azimuth: f64,
    #[rust]
    start_elevation: f64,
}

impl Surface3D {
    // ---- Rust-side API (mirrors the 1.0 library surface) ----

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn set_data(&mut self, z: Vec<Vec<f64>>) {
        if z.is_empty() || z[0].is_empty() {
            return;
        }
        let mut z_min = f64::MAX;
        let mut z_max = f64::MIN;
        for row in &z {
            for &val in row {
                if val < z_min {
                    z_min = val;
                }
                if val > z_max {
                    z_max = val;
                }
            }
        }
        self.z_data = z;
        self.z_range = (z_min, z_max);
    }

    pub fn set_view(&mut self, view: View3D) {
        self.view3d = view;
    }

    pub fn set_azimuth(&mut self, az: f64) {
        self.view3d.azimuth = az;
    }

    pub fn set_elevation(&mut self, el: f64) {
        self.view3d.elevation = el;
    }

    pub fn set_colormap(&mut self, name: impl Into<String>) {
        self.colormap = name.into();
    }

    pub fn set_wireframe(&mut self, show: bool) {
        self.show_wireframe = show;
    }

    pub fn set_surface(&mut self, show: bool) {
        self.show_surface = show;
    }

    pub fn clear(&mut self) {
        self.z_data.clear();
    }

    fn normalize_z(&self, z: f64) -> f64 {
        if self.z_range.1 == self.z_range.0 {
            return 0.5;
        }
        (z - self.z_range.0) / (self.z_range.1 - self.z_range.0)
    }

    fn make_demo_data(&mut self) {
        // Ripple surface: sin(r) falling off from the center
        let n = 24usize;
        let mut z = Vec::with_capacity(n);
        for i in 0..n {
            let mut row = Vec::with_capacity(n);
            for j in 0..n {
                let x = (j as f64 / (n - 1) as f64) * 6.0 - 3.0;
                let y = (i as f64 / (n - 1) as f64) * 6.0 - 3.0;
                let r = (x * x + y * y).sqrt();
                row.push((r * 1.8).sin() * (-r * 0.25).exp());
            }
            z.push(row);
        }
        self.set_data(z);
    }

    fn draw_surface(&mut self, _cx: &mut Cx2d) {
        let rows = self.z_data.len();
        if rows < 2 || self.z_data[0].len() < 2 {
            return;
        }
        let cols = self.z_data[0].len();

        let pr = self.plot_view.plot_rect;
        let cx_center = pr.pos.x + pr.size.x * 0.5;
        let cy_center = pr.pos.y + pr.size.y * 0.5;
        let zoom = if self.zoom == 0.0 { 1.0 } else { self.zoom };
        let scale = pr.size.x.min(pr.size.y) * 0.27 * zoom;

        // Normalize coordinates to [-1, 1]
        let x_scale = 2.0 / (cols - 1).max(1) as f64;
        let y_scale = 2.0 / (rows - 1).max(1) as f64;
        let z_scale = if self.z_range.1 != self.z_range.0 {
            1.5 / (self.z_range.1 - self.z_range.0)
        } else {
            1.0
        };
        let z_offset = (self.z_range.0 + self.z_range.1) * 0.5;

        let cmap = Colormap::from_name(&self.colormap);
        let view3d = self.view3d.clone();
        let show_surface = self.show_surface;
        let mut show_wireframe = self.show_wireframe;
        if !show_surface && !show_wireframe {
            show_wireframe = true;
        }
        let wire_color = if show_surface {
            vec4(0.0, 0.0, 0.0, 0.5)
        } else {
            vec4(0.2, 0.4, 0.8, 1.0)
        };

        // Project a grid corner to pixel space
        let project_corner = |zd: &Vec<Vec<f64>>, ci: usize, cj: usize| -> (f32, f32) {
            let x = cj as f64 * x_scale - 1.0;
            let y = ci as f64 * y_scale - 1.0;
            let z = (zd[ci][cj] - z_offset) * z_scale;
            view3d.project_px(x, y, z, cx_center, cy_center, scale)
        };

        // Collect all quads with their depth (painter's algorithm)
        let mut quads: Vec<(f64, usize, usize, Vec4)> = Vec::new();
        for i in 0..rows - 1 {
            for j in 0..cols - 1 {
                let avg_z = (self.z_data[i][j]
                    + self.z_data[i + 1][j]
                    + self.z_data[i][j + 1]
                    + self.z_data[i + 1][j + 1])
                    * 0.25;
                let t = self.normalize_z(avg_z);
                let color = cmap.sample(t);

                let cx_q = (j as f64 + 0.5) * x_scale - 1.0;
                let cy_q = (i as f64 + 0.5) * y_scale - 1.0;
                let cz_q = (avg_z - z_offset) * z_scale;
                let depth = view3d.depth(cx_q, cy_q, cz_q);
                quads.push((depth, i, j, color));
            }
        }

        // Sort back to front (larger depth = further away)
        quads.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Draw quads (fill + optional wireframe edges) in sorted order
        for (_, i, j, color) in quads {
            let corners = [(j, i), (j + 1, i), (j + 1, i + 1), (j, i + 1)];
            let mut pts = [(0.0f32, 0.0f32); 4];
            for (k, &(cj, ci)) in corners.iter().enumerate() {
                pts[k] = project_corner(&self.z_data, ci, cj);
            }
            if show_surface {
                self.plot_view.fill_polygon_px(&pts, color);
            }
            if show_wireframe {
                self.plot_view.stroke_polygon_px(&pts, wire_color, 1.0);
            }
        }

        // Draw the 3D axis box edges (three principal axes through the origin)
        self.plot_view.set_color(AXIS_COLOR);
        let axis_len = 1.2;
        let axes = [
            ((-axis_len, 0.0, 0.0), (axis_len, 0.0, 0.0)),
            ((0.0, -axis_len, 0.0), (0.0, axis_len, 0.0)),
            ((0.0, 0.0, -axis_len * 0.5), (0.0, 0.0, axis_len)),
        ];
        for ((x0, y0, z0), (x1, y1, z1)) in axes {
            let (px0, py0) = view3d.project_px(x0, y0, z0, cx_center, cy_center, scale);
            let (px1, py1) = view3d.project_px(x1, y1, z1, cx_center, cy_center, scale);
            self.plot_view.line_px(px0, py0, px1, py1, 1.0);
        }
    }
}

impl Widget for Surface3D {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        if handle_rotate_event(
            cx,
            event,
            self.plot_view.draw_bg.area(),
            &mut self.view3d,
            &mut self.zoom,
            &mut self.drag_start,
            &mut self.start_azimuth,
            &mut self.start_elevation,
        ) {
            self.plot_view.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.z_data.is_empty() && self.demo_data {
            self.make_demo_data();
        }
        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx); // grid/ticks/border disabled: title only
        self.draw_surface(cx);
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
                let z = script_arg_f64_matrix(vm, &args, 0).unwrap_or_default();
                self.z_data.clear();
                self.set_data(z);
            }
            x if x == live_id!(set_colormap) => {
                self.colormap = script_arg_string(vm, &args, 0).unwrap_or_default();
            }
            x if x == live_id!(set_rotation) => {
                if let Some(az) = script_arg_f64(vm, &args, 0) {
                    self.view3d.azimuth = az;
                }
                if let Some(el) = script_arg_f64(vm, &args, 1) {
                    self.view3d.elevation = el.clamp(-89.0, 89.0);
                }
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
// Scatter3D - depth-sorted 3D scatter points
// =============================================================================

#[derive(Clone, Debug)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub color: Option<Vec4>,
    pub size: Option<f64>,
}

impl Point3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z, color: None, size: None }
    }
    pub fn with_color(mut self, c: Vec4) -> Self {
        self.color = Some(c);
        self
    }
    pub fn with_size(mut self, s: f64) -> Self {
        self.size = Some(s);
        self
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct Scatter3D {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    points: Vec<Point3D>,
    #[rust]
    view3d: View3D,
    #[rust]
    x_range: (f64, f64),
    #[rust]
    y_range: (f64, f64),
    #[rust]
    z_range: (f64, f64),

    #[live]
    pub colormap: String,
    #[live(true)]
    pub color_by_z: bool,
    #[live(4.0)]
    pub point_size: f64,
    #[live(1.0)]
    pub zoom: f64,
    #[live(true)]
    pub demo_data: bool,

    #[rust]
    drag_start: Option<DVec2>,
    #[rust]
    start_azimuth: f64,
    #[rust]
    start_elevation: f64,
}

impl Scatter3D {
    // ---- Rust-side API ----

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn set_data(&mut self, x: Vec<f64>, y: Vec<f64>, z: Vec<f64>) {
        self.points.clear();
        let n = x.len().min(y.len()).min(z.len());
        for i in 0..n {
            self.points.push(Point3D::new(x[i], y[i], z[i]));
        }
        self.auto_range();
    }

    pub fn add_point(&mut self, p: Point3D) {
        self.points.push(p);
        self.auto_range();
    }

    pub fn set_point_size(&mut self, s: f64) {
        self.point_size = s;
    }

    pub fn set_view(&mut self, view: View3D) {
        self.view3d = view;
    }

    pub fn set_azimuth(&mut self, az: f64) {
        self.view3d.azimuth = az;
    }

    pub fn set_elevation(&mut self, el: f64) {
        self.view3d.elevation = el;
    }

    pub fn clear(&mut self) {
        self.points.clear();
    }

    fn auto_range(&mut self) {
        if self.points.is_empty() {
            return;
        }
        let mut x_min = f64::MAX;
        let mut x_max = f64::MIN;
        let mut y_min = f64::MAX;
        let mut y_max = f64::MIN;
        let mut z_min = f64::MAX;
        let mut z_max = f64::MIN;
        for p in &self.points {
            x_min = x_min.min(p.x);
            x_max = x_max.max(p.x);
            y_min = y_min.min(p.y);
            y_max = y_max.max(p.y);
            z_min = z_min.min(p.z);
            z_max = z_max.max(p.z);
        }
        let pad_x = (x_max - x_min).max(0.1) * 0.1;
        let pad_y = (y_max - y_min).max(0.1) * 0.1;
        let pad_z = (z_max - z_min).max(0.1) * 0.1;
        self.x_range = (x_min - pad_x, x_max + pad_x);
        self.y_range = (y_min - pad_y, y_max + pad_y);
        self.z_range = (z_min - pad_z, z_max + pad_z);
    }

    fn make_demo_data(&mut self) {
        // Helix point cloud with deterministic jitter
        let mut rng = DemoRng::new(7);
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        let mut zs = Vec::new();
        for i in 0..160 {
            let t = i as f64 * 0.08;
            let r = 0.6 + t * 0.10;
            xs.push(r * (t * 2.2).cos() + (rng.next_f64() - 0.5) * 0.25);
            ys.push(r * (t * 2.2).sin() + (rng.next_f64() - 0.5) * 0.25);
            zs.push(t * 0.35 + (rng.next_f64() - 0.5) * 0.2);
        }
        self.set_data(xs, ys, zs);
    }

    fn draw_points(&mut self, _cx: &mut Cx2d) {
        if self.points.is_empty() {
            return;
        }
        let pr = self.plot_view.plot_rect;
        let cx_center = pr.pos.x + pr.size.x * 0.5;
        let cy_center = pr.pos.y + pr.size.y * 0.5;
        let zoom = if self.zoom == 0.0 { 1.0 } else { self.zoom };
        let scale = pr.size.x.min(pr.size.y) * 0.27 * zoom;

        // Normalize to [-1, 1]
        let x_scale = if self.x_range.1 != self.x_range.0 {
            2.0 / (self.x_range.1 - self.x_range.0)
        } else {
            1.0
        };
        let y_scale = if self.y_range.1 != self.y_range.0 {
            2.0 / (self.y_range.1 - self.y_range.0)
        } else {
            1.0
        };
        let z_scale = if self.z_range.1 != self.z_range.0 {
            2.0 / (self.z_range.1 - self.z_range.0)
        } else {
            1.0
        };
        let x_off = (self.x_range.0 + self.x_range.1) * 0.5;
        let y_off = (self.y_range.0 + self.y_range.1) * 0.5;
        let z_off = (self.z_range.0 + self.z_range.1) * 0.5;

        let view3d = self.view3d.clone();
        let cmap = Colormap::from_name(&self.colormap);
        let default_color = cycle_color(0);
        let default_size = if self.point_size <= 0.0 { 4.0 } else { self.point_size };
        let color_by_z = self.color_by_z;
        let (zr0, zr1) = self.z_range;

        // Sort points back to front, tracking min/max depth for size cueing
        let mut min_depth = f64::MAX;
        let mut max_depth = f64::MIN;
        let mut sorted: Vec<(f64, usize)> = Vec::with_capacity(self.points.len());
        for (i, p) in self.points.iter().enumerate() {
            let x = (p.x - x_off) * x_scale;
            let y = (p.y - y_off) * y_scale;
            let z = (p.z - z_off) * z_scale;
            let d = view3d.depth(x, y, z);
            min_depth = min_depth.min(d);
            max_depth = max_depth.max(d);
            sorted.push((d, i));
        }
        sorted.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let depth_range = (max_depth - min_depth).max(1e-9);

        // Draw the 3D axes behind the points
        self.plot_view.set_color(AXIS_COLOR);
        let axis_len = 1.2;
        let axes = [
            ((-axis_len, 0.0, 0.0), (axis_len, 0.0, 0.0)),
            ((0.0, -axis_len, 0.0), (0.0, axis_len, 0.0)),
            ((0.0, 0.0, -axis_len * 0.5), (0.0, 0.0, axis_len)),
        ];
        for ((x0, y0, z0), (x1, y1, z1)) in axes {
            let (px0, py0) = view3d.project_px(x0, y0, z0, cx_center, cy_center, scale);
            let (px1, py1) = view3d.project_px(x1, y1, z1, cx_center, cy_center, scale);
            self.plot_view.line_px(px0, py0, px1, py1, 1.0);
        }

        // Draw points back to front; nearer points draw larger
        for (depth, idx) in sorted {
            let p = self.points[idx].clone();
            let x = (p.x - x_off) * x_scale;
            let y = (p.y - y_off) * y_scale;
            let z = (p.z - z_off) * z_scale;
            let (px, py) = view3d.project_px(x, y, z, cx_center, cy_center, scale);

            let color = p.color.unwrap_or_else(|| {
                if color_by_z {
                    let t = if zr1 != zr0 { (p.z - zr0) / (zr1 - zr0) } else { 0.5 };
                    cmap.sample(t)
                } else {
                    default_color
                }
            });
            // Depth cue: near points (small depth) slightly larger
            let near_t = 1.0 - (depth - min_depth) / depth_range;
            let size = p.size.unwrap_or(default_size) * (0.7 + 0.5 * near_t);
            self.plot_view.fill_circle_px(px, py, size as f32, color);
        }
    }
}

impl Widget for Scatter3D {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        if handle_rotate_event(
            cx,
            event,
            self.plot_view.draw_bg.area(),
            &mut self.view3d,
            &mut self.zoom,
            &mut self.drag_start,
            &mut self.start_azimuth,
            &mut self.start_elevation,
        ) {
            self.plot_view.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.points.is_empty() && self.demo_data {
            self.make_demo_data();
        }
        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx); // title only
        self.draw_points(cx);
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
                let zs = script_arg_f64_array(vm, &args, 2).unwrap_or_default();
                self.set_data(xs, ys, zs);
            }
            x if x == live_id!(set_rotation) => {
                if let Some(az) = script_arg_f64(vm, &args, 0) {
                    self.view3d.azimuth = az;
                }
                if let Some(el) = script_arg_f64(vm, &args, 1) {
                    self.view3d.elevation = el.clamp(-89.0, 89.0);
                }
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
// Line3D - 3D polyline series
// =============================================================================

#[derive(Clone, Debug)]
pub struct Line3DSeries {
    pub label: String,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub z: Vec<f64>,
    pub color: Vec4,
    pub width: f64,
}

impl Line3DSeries {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            x: Vec::new(),
            y: Vec::new(),
            z: Vec::new(),
            color: cycle_color(0),
            width: 1.5,
        }
    }
    pub fn with_data(mut self, x: Vec<f64>, y: Vec<f64>, z: Vec<f64>) -> Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }
    pub fn with_color(mut self, c: Vec4) -> Self {
        self.color = c;
        self
    }
    pub fn with_width(mut self, w: f64) -> Self {
        self.width = w;
        self
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct Line3D {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    series: Vec<Line3DSeries>,
    #[rust]
    view3d: View3D,
    #[rust]
    x_range: (f64, f64),
    #[rust]
    y_range: (f64, f64),
    #[rust]
    z_range: (f64, f64),

    #[live(1.0)]
    pub zoom: f64,
    #[live(true)]
    pub demo_data: bool,

    #[rust]
    drag_start: Option<DVec2>,
    #[rust]
    start_azimuth: f64,
    #[rust]
    start_elevation: f64,
}

impl Line3D {
    // ---- Rust-side API ----

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn add_series(&mut self, s: Line3DSeries) {
        self.series.push(s);
        self.auto_range();
    }

    pub fn set_data(&mut self, x: Vec<f64>, y: Vec<f64>, z: Vec<f64>) {
        self.series.clear();
        self.add_series(Line3DSeries::new("").with_data(x, y, z));
    }

    pub fn set_view(&mut self, view: View3D) {
        self.view3d = view;
    }

    pub fn set_azimuth(&mut self, az: f64) {
        self.view3d.azimuth = az;
    }

    pub fn set_elevation(&mut self, el: f64) {
        self.view3d.elevation = el;
    }

    pub fn clear(&mut self) {
        self.series.clear();
    }

    fn auto_range(&mut self) {
        let mut x_min = f64::MAX;
        let mut x_max = f64::MIN;
        let mut y_min = f64::MAX;
        let mut y_max = f64::MIN;
        let mut z_min = f64::MAX;
        let mut z_max = f64::MIN;
        for s in &self.series {
            for &v in &s.x {
                x_min = x_min.min(v);
                x_max = x_max.max(v);
            }
            for &v in &s.y {
                y_min = y_min.min(v);
                y_max = y_max.max(v);
            }
            for &v in &s.z {
                z_min = z_min.min(v);
                z_max = z_max.max(v);
            }
        }
        if x_min != f64::MAX {
            let pad_x = (x_max - x_min).max(0.1) * 0.1;
            let pad_y = (y_max - y_min).max(0.1) * 0.1;
            let pad_z = (z_max - z_min).max(0.1) * 0.1;
            self.x_range = (x_min - pad_x, x_max + pad_x);
            self.y_range = (y_min - pad_y, y_max + pad_y);
            self.z_range = (z_min - pad_z, z_max + pad_z);
        }
    }

    fn make_demo_data(&mut self) {
        // Helix + Lissajous curve demo
        let n = 200;
        let mut hx = Vec::with_capacity(n);
        let mut hy = Vec::with_capacity(n);
        let mut hz = Vec::with_capacity(n);
        let mut lx = Vec::with_capacity(n);
        let mut ly = Vec::with_capacity(n);
        let mut lz = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f64 / (n - 1) as f64 * 12.0;
            hx.push(t.cos());
            hy.push(t.sin());
            hz.push(t * 0.15 - 0.9);
            lx.push((t * 0.75).sin() * 1.2);
            ly.push((t * 0.5).cos() * 1.2);
            lz.push((t * 0.9).sin() * 0.6);
        }
        self.add_series(
            Line3DSeries::new("helix")
                .with_data(hx, hy, hz)
                .with_color(cycle_color(0)),
        );
        self.add_series(
            Line3DSeries::new("lissajous")
                .with_data(lx, ly, lz)
                .with_color(cycle_color(1)),
        );
    }

    fn draw_series(&mut self, _cx: &mut Cx2d) {
        let pr = self.plot_view.plot_rect;
        let cx_center = pr.pos.x + pr.size.x * 0.5;
        let cy_center = pr.pos.y + pr.size.y * 0.5;
        let zoom = if self.zoom == 0.0 { 1.0 } else { self.zoom };
        let scale = pr.size.x.min(pr.size.y) * 0.27 * zoom;

        // Normalize to [-1, 1]
        let x_scale = if self.x_range.1 != self.x_range.0 {
            2.0 / (self.x_range.1 - self.x_range.0)
        } else {
            1.0
        };
        let y_scale = if self.y_range.1 != self.y_range.0 {
            2.0 / (self.y_range.1 - self.y_range.0)
        } else {
            1.0
        };
        let z_scale = if self.z_range.1 != self.z_range.0 {
            2.0 / (self.z_range.1 - self.z_range.0)
        } else {
            1.0
        };
        let x_off = (self.x_range.0 + self.x_range.1) * 0.5;
        let y_off = (self.y_range.0 + self.y_range.1) * 0.5;
        let z_off = (self.z_range.0 + self.z_range.1) * 0.5;

        let view3d = self.view3d.clone();

        // Draw the 3D axes first
        self.plot_view.set_color(AXIS_COLOR);
        let axis_len = 1.2;
        let axes = [
            ((-axis_len, 0.0, 0.0), (axis_len, 0.0, 0.0)),
            ((0.0, -axis_len, 0.0), (0.0, axis_len, 0.0)),
            ((0.0, 0.0, -axis_len * 0.5), (0.0, 0.0, axis_len)),
        ];
        for ((x0, y0, z0), (x1, y1, z1)) in axes {
            let (px0, py0) = view3d.project_px(x0, y0, z0, cx_center, cy_center, scale);
            let (px1, py1) = view3d.project_px(x1, y1, z1, cx_center, cy_center, scale);
            self.plot_view.line_px(px0, py0, px1, py1, 1.0);
        }

        // Draw the series polylines
        for si in 0..self.series.len() {
            let s = self.series[si].clone();
            let n = s.x.len().min(s.y.len()).min(s.z.len());
            if n < 2 {
                continue;
            }
            self.plot_view.set_color(s.color);
            for i in 0..n {
                let x = (s.x[i] - x_off) * x_scale;
                let y = (s.y[i] - y_off) * y_scale;
                let z = (s.z[i] - z_off) * z_scale;
                let (px, py) = view3d.project_px(x, y, z, cx_center, cy_center, scale);
                if i == 0 {
                    self.plot_view.draw_vector.move_to(px, py);
                } else {
                    self.plot_view.draw_vector.line_to(px, py);
                }
            }
            self.plot_view.draw_vector.stroke(s.width as f32);
        }
    }
}

impl Widget for Line3D {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        if handle_rotate_event(
            cx,
            event,
            self.plot_view.draw_bg.area(),
            &mut self.view3d,
            &mut self.zoom,
            &mut self.drag_start,
            &mut self.start_azimuth,
            &mut self.start_elevation,
        ) {
            self.plot_view.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.series.is_empty() && self.demo_data {
            self.make_demo_data();
        }
        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx); // title only
        self.draw_series(cx);

        // Legend for labeled series
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
            x if x == live_id!(set_data) => {
                let xs = script_arg_f64_array(vm, &args, 0).unwrap_or_default();
                let ys = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                let zs = script_arg_f64_array(vm, &args, 2).unwrap_or_default();
                self.set_data(xs, ys, zs);
            }
            x if x == live_id!(add_series) => {
                let label = script_arg_string(vm, &args, 0).unwrap_or_default();
                let xs = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                let ys = script_arg_f64_array(vm, &args, 2).unwrap_or_default();
                let zs = script_arg_f64_array(vm, &args, 3).unwrap_or_default();
                let idx = self.series.len();
                self.add_series(
                    Line3DSeries::new(label)
                        .with_data(xs, ys, zs)
                        .with_color(cycle_color(idx)),
                );
            }
            x if x == live_id!(set_rotation) => {
                if let Some(az) = script_arg_f64(vm, &args, 0) {
                    self.view3d.azimuth = az;
                }
                if let Some(el) = script_arg_f64(vm, &args, 1) {
                    self.view3d.elevation = el.clamp(-89.0, 89.0);
                }
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
