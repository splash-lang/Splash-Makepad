// Circular / meter charts: PieChart, DonutChart, GaugeChart, FunnelChart
//
// All four are NON-cartesian widgets: they draw in pixel space centered in
// plot_view.plot_rect(), with grid/ticks/border disabled. draw_axes is still
// called so the title renders.

use crate::plot_view::*;
use crate::script_util::*;
use crate::types::*;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*

    set_type_default() do #(DrawDonutArc::script_shader(vm)) {
        ..mod.draw.DrawQuad
        draw_depth: 0.0
        ink: #ffffff
        ink2: #ffffff
        pixel: fn() {
            let p=(self.pos-vec2(0.5))*self.rect_size
            let radius=min(self.rect_size.x,self.rect_size.y)*0.5
            let inner=radius*self.inner_ratio
            let angle=atan2(p.y,p.x)
            let travel=modf((angle-self.start)*self.direction+12.566370614,6.283185307)
            let center_radius=(radius+inner)*0.5
            let half_width=(radius-inner)*0.5
            let radial=abs(length(p)-center_radius)-half_width
            let edge=min(travel,self.sweep-travel)*center_radius
            var dist=max(radial,-edge)
            if self.rounded>0.5 && self.sweep<6.2831 {
                let a=self.start+self.direction*self.sweep
                let cap1=length(p-vec2(cos(self.start),sin(self.start))*center_radius)-half_width
                let cap2=length(p-vec2(cos(a),sin(a))*center_radius)-half_width
                dist=min(dist,min(cap1,cap2))
            }
            var coverage=1.0-smoothstep(-0.5,0.5,dist)
            if self.dash_length>0.0 && self.dash_gap>0.0 {
                let cycle=self.dash_length+self.dash_gap
                let phase=modf(travel*center_radius+cycle*0.5,cycle)-cycle*0.5
                coverage=coverage*(1.0-smoothstep(self.dash_length*0.5-0.5,self.dash_length*0.5+0.5,abs(phase)))
            }
            var t=0.0
            if self.gradient_mode>2.5 {
                t=length(self.pos-self.gradient_from)/max(length(self.gradient_to-self.gradient_from),0.000001)
            } else if self.gradient_mode>1.5 {
                let angular_p=(self.pos-self.angular_center)*self.rect_size
                t=modf(atan2(angular_p.y,angular_p.x)+6.283185307,6.283185307)/6.283185307
            } else if self.gradient_mode>0.5 {
                let d=self.gradient_to-self.gradient_from
                t=dot(self.pos-self.gradient_from,d)/max(dot(d,d),0.000001)
            }
            let f=clamp((t-self.stop_start)/max(self.stop_end-self.stop_start,0.000001),0.0,1.0)
            var c=mix(self.ink,self.ink2,f)
            if self.three_stops>0.5 {
                if t<self.stop_mid {c=mix(self.ink,self.ink_mid,clamp((t-self.stop_start)/max(self.stop_mid-self.stop_start,0.000001),0.0,1.0))}
                else {c=mix(self.ink_mid,self.ink2,clamp((t-self.stop_mid)/max(self.stop_end-self.stop_mid,0.000001),0.0,1.0))}
            }
            if self.gradient_mode>1.5 && self.gradient_mode<2.5 {
                if t<self.stop_start {t=t+1.0}
                if t>self.stop_end {
                    let wrap=(t-self.stop_end)/max(1.0+self.stop_start-self.stop_end,0.000001)
                    c=mix(self.ink2,self.ink,clamp(wrap,0.0,1.0))
                }
            }
            if c.a*coverage<0.000001 {discard()}
            return vec4(c.rgb*c.a*coverage,c.a*coverage)
        }
    }

    mod.plot.PieChartBase = #(PieChart::register_widget(vm))

    mod.plot.PieChart = set_type_default() do mod.plot.PieChartBase{
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

    mod.plot.DonutChartBase = #(DonutChart::register_widget(vm))

    mod.plot.DonutChart = set_type_default() do mod.plot.DonutChartBase{
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

    mod.plot.GaugeChartBase = #(GaugeChart::register_widget(vm))

    mod.plot.GaugeChart = set_type_default() do mod.plot.GaugeChartBase{
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

    mod.plot.FunnelChartBase = #(FunnelChart::register_widget(vm))

    mod.plot.FunnelChart = set_type_default() do mod.plot.FunnelChartBase{
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

#[derive(Script, ScriptHook)]
#[repr(C)]
struct DrawDonutArc {
    #[deref] draw_super: DrawQuad,
    #[live] ink: Vec4,
    #[live] ink2: Vec4,
    #[live] ink_mid: Vec4,
    #[live] stop_mid: f32,
    #[live] three_stops: f32,
    #[live] inner_ratio: f32,
    #[live] start: f32,
    #[live] sweep: f32,
    #[live] direction: f32,
    #[live] rounded: f32,
    #[live] dash_length: f32,
    #[live] dash_gap: f32,
    #[live] gradient_mode: f32,
    #[live] gradient_from: Vec2,
    #[live] angular_center: Vec2,
    #[live] gradient_to: Vec2,
    #[live] stop_start: f32,
    #[live] stop_end: f32,
}

#[derive(Clone, Debug)]
pub struct DonutArcStyle {
    pub center: (f64,f64),
    pub radius_scale: f64,
    pub inner: f64,
    pub start: f64,
    pub direction: f64,
    pub rounded: bool,
    pub dash_length: f32,
    pub dash_gap: f32,
}

// =============================================================================
// PieChart
// =============================================================================

/// A single pie slice entry
#[derive(Clone, Debug)]
pub struct PieSlice {
    pub label: String,
    pub value: f64,
    pub color: Option<Vec4>,
}

impl PieSlice {
    pub fn new(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
            color: None,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = Some(color);
        self
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct PieChart {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub slices: Vec<PieSlice>,

    #[live(0.8)]
    pub radius_ratio: f64,
    #[live(false)]
    pub show_labels: bool,
    #[live(true)]
    pub show_percentages: bool,
    #[live(true)]
    pub demo_data: bool,
}

impl PieChart {
    // ---- Rust-side API (mirrors the 1.0 library surface) ----

    pub fn add_slice(&mut self, slice: PieSlice) {
        self.slices.push(slice);
    }

    pub fn set_slices(&mut self, slices: Vec<PieSlice>) {
        self.slices = slices;
    }

    pub fn set_data(&mut self, labels: Vec<String>, values: Vec<f64>) {
        self.slices = labels
            .into_iter()
            .zip(values)
            .map(|(l, v)| PieSlice::new(l, v))
            .collect();
    }

    pub fn clear(&mut self) {
        self.slices.clear();
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn set_show_percentages(&mut self, show: bool) {
        self.show_percentages = show;
    }

    pub fn set_show_labels(&mut self, show: bool) {
        self.show_labels = show;
    }

    pub fn set_legend(&mut self, position: LegendPosition) {
        self.plot_view.legend = position;
    }

    fn make_demo_data(&mut self) {
        self.slices = vec![
            PieSlice::new("Product A", 35.0),
            PieSlice::new("Product B", 25.0),
            PieSlice::new("Product C", 20.0),
            PieSlice::new("Product D", 12.0),
            PieSlice::new("Other", 8.0),
        ];
    }

    fn draw_pie(&mut self, cx: &mut Cx2d) {
        let total: f64 = self.slices.iter().map(|s| s.value.max(0.0)).sum();
        if total <= 0.0 {
            return;
        }

        let pr = *self.plot_view.plot_rect();
        let cx0 = pr.pos.x + pr.size.x * 0.5;
        let cy0 = pr.pos.y + pr.size.y * 0.5;
        // Leave room for external labels when they are shown
        let label_room = if self.show_labels { 0.82 } else { 1.0 };
        let radius = (pr.size.x.min(pr.size.y) * 0.5) * self.radius_ratio * label_room;
        if radius < 2.0 {
            return;
        }

        let slices = self.slices.clone();
        let mut start_angle = -std::f64::consts::FRAC_PI_2;

        for (idx, slice) in slices.iter().enumerate() {
            let v = slice.value.max(0.0);
            let slice_angle = (v / total) * std::f64::consts::TAU;
            let end_angle = start_angle + slice_angle;
            let color = slice.color.unwrap_or_else(|| cycle_color(idx));

            self.plot_view.fill_arc_px(
                cx0 as f32,
                cy0 as f32,
                radius as f32,
                0.0,
                start_angle as f32,
                end_angle as f32,
                color,
            );

            let mid_angle = start_angle + slice_angle * 0.5;

            // Percentage inside the slice
            if self.show_percentages && slice_angle > 0.05 {
                let label_radius = radius * 0.65;
                let lx = cx0 + mid_angle.cos() * label_radius;
                let ly = cy0 + mid_angle.sin() * label_radius;
                let percentage = (v / total) * 100.0;
                let text = format!("{:.1}%", percentage);
                self.plot_view
                    .draw_text_centered_px(cx, lx, ly, &text, vec4(1.0, 1.0, 1.0, 1.0), 10.0);
            }

            // External label
            if self.show_labels && slice_angle > 0.02 {
                let label_radius = radius + 10.0;
                let lx = cx0 + mid_angle.cos() * label_radius;
                let ly = cy0 + mid_angle.sin() * label_radius;
                let font_size = 10.0f32;
                let text_color = vec4(0.3, 0.3, 0.3, 1.0);
                if mid_angle.cos() >= 0.0 {
                    self.plot_view.draw_text_px(
                        cx,
                        lx,
                        ly - font_size as f64 * 0.6,
                        &slice.label,
                        text_color,
                        font_size,
                    );
                } else {
                    let est_w = slice.label.len() as f64 * font_size as f64 * 0.5;
                    self.plot_view.draw_text_px(
                        cx,
                        lx - est_w,
                        ly - font_size as f64 * 0.6,
                        &slice.label,
                        text_color,
                        font_size,
                    );
                }
            }

            start_angle = end_angle;
        }
    }
}

impl Widget for PieChart {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.slices.is_empty() && self.demo_data {
            self.make_demo_data();
        }

        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx); // title only (grid/ticks/border disabled)
        self.draw_pie(cx);

        let entries: Vec<(String, Vec4)> = self
            .slices
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
                let labels = script_arg_string_array(vm, &args, 0).unwrap_or_default();
                let values = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                self.set_data(labels, values);
            }
            x if x == live_id!(add_slice) => {
                let label = script_arg_string(vm, &args, 0).unwrap_or_default();
                let value = script_arg_f64(vm, &args, 1).unwrap_or(0.0);
                self.add_slice(PieSlice::new(label, value));
            }
            x if x == live_id!(set_show_percentages) => {
                self.show_percentages = script_arg_bool(vm, &args, 0).unwrap_or(true);
            }
            x if x == live_id!(set_show_labels) => {
                self.show_labels = script_arg_bool(vm, &args, 0).unwrap_or(true);
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

// =============================================================================
// DonutChart
// =============================================================================

/// A single donut slice entry
#[derive(Clone, Debug)]
pub struct DonutSlice {
    pub label: String,
    pub value: f64,
    pub color: Option<Vec4>,
}

impl DonutSlice {
    pub fn new(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
            color: None,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = Some(color);
        self
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct DonutChart {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub slices: Vec<DonutSlice>,
    #[rust]
    pub slice_paints: Vec<Option<crate::ChartPaint>>,
    #[rust]
    pub independent_arcs: Vec<DonutArcStyle>,
    #[live]
    draw_arc: DrawDonutArc,
    #[live(false)]
    pub styled_arcs: bool,
    #[live(false)]
    pub rounded_caps: bool,
    #[live(-1.5707963267948966)]
    pub start_angle: f64,
    #[live(1.0)]
    pub direction: f64,
    #[live(0.95)]
    pub radius_scale: f64,
    #[live]
    pub track_color: Vec4,
    #[rust]
    center_label: String,

    #[live(true)]
    pub show_center_label: bool,
    #[live(0.0)]
    pub slice_gap: f64,
    #[live(0.55)]
    pub inner_radius: f64,
    #[live(false)]
    pub show_labels: bool,
    #[live(true)]
    pub show_percentages: bool,
    #[live(true)]
    pub demo_data: bool,
}

impl DonutChart {
    // ---- Rust-side API ----

    pub fn set_data(&mut self, labels: Vec<String>, values: Vec<f64>) {
        self.slices = labels
            .into_iter()
            .zip(values)
            .map(|(l, v)| DonutSlice::new(l, v))
            .collect();
    }

    pub fn set_slices(&mut self, slices: Vec<DonutSlice>) {
        self.slices = slices;
    }

    pub fn add_slice(&mut self, slice: DonutSlice) {
        self.slices.push(slice);
    }

    pub fn set_inner_radius_ratio(&mut self, ratio: f64) {
        self.inner_radius = ratio.clamp(0.0, 0.9);
    }

    pub fn set_center_label(&mut self, label: impl Into<String>) {
        self.center_label = label.into();
    }

    pub fn set_show_labels(&mut self, show: bool) {
        self.show_labels = show;
    }

    pub fn set_show_percentages(&mut self, show: bool) {
        self.show_percentages = show;
    }

    pub fn clear(&mut self) {
        self.slices.clear();
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    fn make_demo_data(&mut self) {
        self.slices = vec![
            DonutSlice::new("Rust", 42.0),
            DonutSlice::new("Go", 23.0),
            DonutSlice::new("Python", 19.0),
            DonutSlice::new("C++", 16.0),
        ];
    }

    fn draw_donut(&mut self, cx: &mut Cx2d) {
        let total: f64 = self.slices.iter().map(|s| s.value.max(0.0)).sum();
        if total <= 0.0 {
            return;
        }

        let pr = *self.plot_view.plot_rect();
        let cx0 = pr.pos.x + pr.size.x * 0.5;
        let cy0 = pr.pos.y + pr.size.y * 0.5;
        let label_room = if self.show_labels || self.show_percentages {
            0.8
        } else {
            self.radius_scale
        };
        let outer_radius = (pr.size.x.min(pr.size.y) * 0.5) * label_room;
        if outer_radius < 2.0 {
            return;
        }
        let inner_ratio = self.inner_radius.clamp(0.0, 0.999) as f32;

        let slices = self.slices.clone();
        let mut start_angle = self.start_angle;

        if self.styled_arcs && self.track_color.w>0. {
            let draw=&mut self.draw_arc;
            draw.ink=self.track_color;draw.ink2=self.track_color;draw.inner_ratio=inner_ratio;
            draw.start=0.;draw.sweep=std::f32::consts::TAU;draw.direction=1.;draw.rounded=0.;
            draw.gradient_mode=0.;draw.dash_length=0.;draw.dash_gap=0.;draw.three_stops=0.;draw.stop_start=0.;draw.stop_end=1.;
            draw.draw_abs(cx,Rect{pos:dvec2(cx0-outer_radius,cy0-outer_radius),size:dvec2(outer_radius*2.,outer_radius*2.)});
        }

        for (i, slice) in slices.iter().enumerate() {
            let v = slice.value.max(0.0);
            let arc=self.independent_arcs.get(i);
            let sweep_angle = if arc.is_some() {v.clamp(0.,1.)*std::f64::consts::TAU} else {(v/total)*std::f64::consts::TAU};
            let (cx0,cy0,outer_radius,inner_ratio,direction,rounded)=if let Some(arc)=arc {
                start_angle=arc.start;
                (pr.pos.x+pr.size.x*arc.center.0,pr.pos.y+pr.size.y*arc.center.1,
                 pr.size.x.min(pr.size.y)*0.5*arc.radius_scale,arc.inner as f32,arc.direction,arc.rounded)
            }else{(cx0,cy0,outer_radius,inner_ratio,self.direction,self.rounded_caps)};
            let end_angle = start_angle + sweep_angle*direction;
            let color = slice.color.unwrap_or_else(|| cycle_color(i));

            if self.styled_arcs && sweep_angle>1e-6 {
                let draw=&mut self.draw_arc;
                draw.ink=color;draw.ink2=color;
                draw.inner_ratio=inner_ratio;draw.start=start_angle as f32;
                draw.sweep=sweep_angle as f32;draw.direction=direction as f32;
                draw.dash_length=arc.map(|a|a.dash_length).unwrap_or(0.);draw.dash_gap=arc.map(|a|a.dash_gap).unwrap_or(0.);
                draw.rounded=if rounded {1.}else{0.};draw.gradient_mode=0.;draw.three_stops=0.;
                draw.stop_start=0.;draw.stop_end=1.;
                if let Some(Some(paint))=self.slice_paints.get(i) {
                    if let (Some(first),Some(last))=(paint.stops.first(),paint.stops.last()) {
                        draw.ink=first.1;draw.ink2=last.1;
                        if paint.stops.len()==3 {draw.ink_mid=paint.stops[1].1;draw.stop_mid=paint.stops[1].0;draw.three_stops=1.;}draw.stop_start=first.0;draw.stop_end=last.0;
                        draw.gradient_mode=if paint.angular {2.}else if paint.radial {3.}else{1.};
                        draw.angular_center=vec2(paint.angular_center.0,paint.angular_center.1);
                        draw.gradient_from=vec2(paint.from.0,paint.from.1);
                        draw.gradient_to=vec2(paint.to.0,paint.to.1);
                    }
                }
                draw.draw_abs(cx,Rect{pos:dvec2(cx0-outer_radius,cy0-outer_radius),size:dvec2(outer_radius*2.,outer_radius*2.)});
            } else if !self.styled_arcs {self.plot_view.fill_arc_px(
                cx0 as f32,
                cy0 as f32,
                outer_radius as f32,
                inner_ratio,
                (start_angle+self.slice_gap/outer_radius*0.5) as f32,
                (end_angle-self.slice_gap/outer_radius*0.5) as f32,
                color,
            );}

            // External labels / percentages
            if (self.show_labels || self.show_percentages) && sweep_angle > 0.02 {
                let mid_angle = start_angle + sweep_angle * 0.5;
                let label_radius = outer_radius + 12.0;
                let lx = cx0 + label_radius * mid_angle.cos();
                let ly = cy0 + label_radius * mid_angle.sin();

                let label_text = if self.show_percentages {
                    let pct = (v / total) * 100.0;
                    if self.show_labels {
                        format!("{} ({:.1}%)", slice.label, pct)
                    } else {
                        format!("{:.1}%", pct)
                    }
                } else {
                    slice.label.clone()
                };

                let font_size = 10.0f32;
                let text_color = vec4(0.3, 0.3, 0.3, 1.0);
                if mid_angle.cos() >= 0.0 {
                    self.plot_view.draw_text_px(
                        cx,
                        lx,
                        ly - font_size as f64 * 0.6,
                        &label_text,
                        text_color,
                        font_size,
                    );
                } else {
                    let est_w = label_text.len() as f64 * font_size as f64 * 0.5;
                    self.plot_view.draw_text_px(
                        cx,
                        lx - est_w,
                        ly - font_size as f64 * 0.6,
                        &label_text,
                        text_color,
                        font_size,
                    );
                }
            }

            start_angle = end_angle;
        }

        if !self.show_center_label {return;}
        // Center text: explicit center label or the total
        let center_text = if self.center_label.is_empty() {
            let rounded = format!("{:.1}", total);
            if rounded.ends_with(".0") {
                format!("{:.0}", total)
            } else {
                rounded
            }
        } else {
            self.center_label.clone()
        };
        self.plot_view.draw_text_centered_px(
            cx,
            cx0,
            cy0,
            &center_text,
            vec4(0.2, 0.2, 0.2, 1.0),
            14.0,
        );
    }
}

impl Widget for DonutChart {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.slices.is_empty() && self.demo_data {
            self.make_demo_data();
        }

        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx); // title only
        self.draw_donut(cx);

        let entries: Vec<(String, Vec4)> = self
            .slices
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
                let labels = script_arg_string_array(vm, &args, 0).unwrap_or_default();
                let values = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                self.set_data(labels, values);
            }
            x if x == live_id!(add_slice) => {
                let label = script_arg_string(vm, &args, 0).unwrap_or_default();
                let value = script_arg_f64(vm, &args, 1).unwrap_or(0.0);
                self.add_slice(DonutSlice::new(label, value));
            }
            x if x == live_id!(set_show_percentages) => {
                self.show_percentages = script_arg_bool(vm, &args, 0).unwrap_or(true);
            }
            x if x == live_id!(set_show_labels) => {
                self.show_labels = script_arg_bool(vm, &args, 0).unwrap_or(true);
            }
            x if x == live_id!(set_inner_radius) => {
                let ratio = script_arg_f64(vm, &args, 0).unwrap_or(0.55);
                self.set_inner_radius_ratio(ratio);
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

// =============================================================================
// GaugeChart
// =============================================================================

#[derive(Script, ScriptHook, Widget)]
pub struct GaugeChart {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    value: f64,
    #[rust]
    min_value: f64,
    #[rust]
    max_value: f64,
    #[rust]
    thresholds: Vec<(f64, Vec4)>, // (value, color) pairs, ascending
    #[rust]
    unit: String,
    #[rust]
    has_value: bool,

    #[live(20.0)]
    pub arc_width: f64,
    #[live(true)]
    pub show_value: bool,
    #[live(true)]
    pub demo_data: bool,
}

impl GaugeChart {
    // ---- Rust-side API ----

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn set_value(&mut self, value: f64) {
        self.value = value;
        self.has_value = true;
    }

    pub fn set_range(&mut self, min: f64, max: f64) {
        self.min_value = min;
        self.max_value = max;
    }

    pub fn set_thresholds(&mut self, thresholds: Vec<(f64, Vec4)>) {
        self.thresholds = thresholds;
    }

    pub fn set_unit(&mut self, unit: impl Into<String>) {
        self.unit = unit.into();
    }

    pub fn set_show_value(&mut self, show: bool) {
        self.show_value = show;
    }

    pub fn set_arc_width(&mut self, width: f64) {
        self.arc_width = width;
    }

    fn get_color_for_value(&self, value: f64, thresholds: &[(f64, Vec4)]) -> Vec4 {
        if thresholds.is_empty() {
            return vec4(0.12, 0.47, 0.71, 1.0);
        }
        for &(threshold, color) in thresholds.iter().rev() {
            if value >= threshold {
                return color;
            }
        }
        thresholds
            .first()
            .map(|&(_, c)| c)
            .unwrap_or(vec4(0.12, 0.47, 0.71, 1.0))
    }

    fn draw_gauge(&mut self, cx: &mut Cx2d) {
        // Defaults
        let (min_v, max_v) = if self.max_value == 0.0 && self.min_value == 0.0 {
            (0.0, 100.0)
        } else {
            (self.min_value, self.max_value)
        };
        let range = (max_v - min_v).abs().max(1e-9);
        let thresholds = if self.thresholds.is_empty() {
            vec![
                (min_v, vec4(0.17, 0.63, 0.17, 1.0)),               // Green
                (min_v + 0.6 * range, vec4(1.0, 0.65, 0.0, 1.0)),   // Orange
                (min_v + 0.8 * range, vec4(0.84, 0.15, 0.16, 1.0)), // Red
            ]
        } else {
            self.thresholds.clone()
        };
        let arc_width = if self.arc_width <= 0.0 { 20.0 } else { self.arc_width };

        let pr = *self.plot_view.plot_rect();
        let cx0 = pr.pos.x + pr.size.x * 0.5;
        let cy0 = pr.pos.y + pr.size.y * 0.58;
        let radius = (pr.size.x.min(pr.size.y) * 0.5 - arc_width).max(20.0);

        // Gauge sweeps 270 degrees over the top: lower-left -> top -> lower-right
        // (screen coordinates, y down).
        let start_angle = std::f64::consts::PI * 0.75; // lower-left
        let end_angle = std::f64::consts::PI * 2.25; // lower-right
        let total_angle = end_angle - start_angle;

        // Background arc
        self.plot_view.stroke_arc_px(
            cx0 as f32,
            cy0 as f32,
            radius as f32,
            start_angle as f32,
            end_angle as f32,
            vec4(0.85, 0.85, 0.85, 1.0),
            arc_width as f32,
        );

        // Value arc
        let value_ratio = ((self.value - min_v) / range).clamp(0.0, 1.0);
        let value_angle = start_angle + value_ratio * total_angle;
        let color = self.get_color_for_value(self.value, &thresholds);
        if value_ratio > 1e-4 {
            self.plot_view.stroke_arc_px(
                cx0 as f32,
                cy0 as f32,
                radius as f32,
                start_angle as f32,
                value_angle as f32,
                color,
                arc_width as f32,
            );
        }

        // Needle
        let needle_length = (radius - arc_width * 0.5 - 5.0).max(10.0);
        let nx = cx0 + needle_length * value_angle.cos();
        let ny = cy0 + needle_length * value_angle.sin();
        self.plot_view.set_color(vec4(0.2, 0.2, 0.2, 1.0));
        self.plot_view
            .line_px(cx0 as f32, cy0 as f32, nx as f32, ny as f32, 3.0);

        // Center hub
        self.plot_view
            .fill_circle_px(cx0 as f32, cy0 as f32, 7.0, vec4(0.3, 0.3, 0.3, 1.0));

        // Value text
        if self.show_value {
            let value_text = if self.unit.is_empty() {
                format!("{:.1}", self.value)
            } else {
                format!("{:.1}{}", self.value, self.unit)
            };
            self.plot_view.draw_text_centered_px(
                cx,
                cx0,
                cy0 + 30.0,
                &value_text,
                vec4(0.2, 0.2, 0.2, 1.0),
                14.0,
            );
        }

        // Min / max labels at the arc ends
        let min_x = cx0 + radius * start_angle.cos();
        let min_y = cy0 + radius * start_angle.sin() + 14.0;
        let max_x = cx0 + radius * end_angle.cos();
        let max_y = cy0 + radius * end_angle.sin() + 14.0;
        let text_color = vec4(0.35, 0.35, 0.35, 1.0);
        self.plot_view
            .draw_text_centered_px(cx, min_x, min_y, &format!("{:.0}", min_v), text_color, 10.0);
        self.plot_view
            .draw_text_centered_px(cx, max_x, max_y, &format!("{:.0}", max_v), text_color, 10.0);
    }
}

impl Widget for GaugeChart {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.has_value && self.demo_data {
            self.value = 72.0;
            self.min_value = 0.0;
            self.max_value = 100.0;
            self.has_value = true;
        }

        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx); // title only
        self.draw_gauge(cx);
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
            x if x == live_id!(set_value) => {
                if let Some(v) = script_arg_f64(vm, &args, 0) {
                    self.set_value(v);
                }
            }
            x if x == live_id!(set_range) => {
                if let (Some(lo), Some(hi)) =
                    (script_arg_f64(vm, &args, 0), script_arg_f64(vm, &args, 1))
                {
                    self.set_range(lo, hi);
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
// FunnelChart
// =============================================================================

/// A funnel stage entry
#[derive(Clone, Debug)]
pub struct FunnelStage {
    pub label: String,
    pub value: f64,
    pub color: Option<Vec4>,
}

impl FunnelStage {
    pub fn new(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
            color: None,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = Some(color);
        self
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct FunnelChart {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub stages: Vec<FunnelStage>,

    #[live(false)]
    pub show_percentages: bool,
    #[live(true)]
    pub show_values: bool,
    #[live(true)]
    pub demo_data: bool,
}

impl FunnelChart {
    // ---- Rust-side API ----

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn set_data(&mut self, labels: Vec<String>, values: Vec<f64>) {
        self.stages = labels
            .into_iter()
            .zip(values)
            .map(|(l, v)| FunnelStage::new(l, v))
            .collect();
    }

    pub fn set_stages(&mut self, stages: Vec<FunnelStage>) {
        self.stages = stages;
    }

    pub fn add_stage(&mut self, stage: FunnelStage) {
        self.stages.push(stage);
    }

    pub fn set_show_percentages(&mut self, show: bool) {
        self.show_percentages = show;
    }

    pub fn set_show_values(&mut self, show: bool) {
        self.show_values = show;
    }

    pub fn clear(&mut self) {
        self.stages.clear();
    }

    fn make_demo_data(&mut self) {
        self.stages = vec![
            FunnelStage::new("Visits", 1000.0),
            FunnelStage::new("Signups", 640.0),
            FunnelStage::new("Trials", 380.0),
            FunnelStage::new("Purchases", 210.0),
            FunnelStage::new("Renewals", 120.0),
        ];
    }

    fn draw_funnel(&mut self, cx: &mut Cx2d) {
        let max_value = self.stages.iter().map(|s| s.value).fold(0.0f64, f64::max);
        if max_value <= 0.0 || self.stages.is_empty() {
            return;
        }

        let pr = *self.plot_view.plot_rect();
        let num_stages = self.stages.len();
        let gap = 2.0;
        let stage_height =
            ((pr.size.y - gap * (num_stages as f64 - 1.0)) / num_stages as f64).max(2.0);
        let center_x = pr.pos.x + pr.size.x * 0.5;
        let max_width = pr.size.x * 0.9;

        let stages = self.stages.clone();
        for (i, stage) in stages.iter().enumerate() {
            let ratio = (stage.value / max_value).clamp(0.0, 1.0);
            let width = max_width * ratio;
            let y = pr.pos.y + i as f64 * (stage_height + gap);

            let color = stage.color.unwrap_or_else(|| cycle_color(i));

            // Bottom edge width comes from the next stage (taper at the end)
            let next_ratio = if i + 1 < num_stages {
                (stages[i + 1].value / max_value).clamp(0.0, 1.0)
            } else {
                ratio * 0.3
            };
            let next_width = max_width * next_ratio;

            // Trapezoid
            let pts = [
                ((center_x - width * 0.5) as f32, y as f32),
                ((center_x + width * 0.5) as f32, y as f32),
                (
                    (center_x + next_width * 0.5) as f32,
                    (y + stage_height) as f32,
                ),
                (
                    (center_x - next_width * 0.5) as f32,
                    (y + stage_height) as f32,
                ),
            ];
            self.plot_view.fill_polygon_px(&pts, color);
            self.plot_view
                .stroke_polygon_px(&pts, vec4(1.0, 1.0, 1.0, 0.8), 1.0);

            // Stage label centered inside the trapezoid
            let mid_y = y + stage_height * 0.5;
            let mut text = stage.label.clone();
            if self.show_percentages {
                text = format!("{} ({:.1}%)", text, ratio * 100.0);
            } else if self.show_values {
                text = format!("{} ({:.0})", text, stage.value);
            }
            self.plot_view.draw_text_centered_px(
                cx,
                center_x,
                mid_y,
                &text,
                vec4(1.0, 1.0, 1.0, 1.0),
                10.0,
            );
        }
    }
}

impl Widget for FunnelChart {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.stages.is_empty() && self.demo_data {
            self.make_demo_data();
        }

        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx); // title only
        self.draw_funnel(cx);
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
