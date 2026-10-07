// Hierarchy / flow charts: Treemap and SankeyDiagram (Makepad 2.0 / Octoscript port)
//
// Both are NON-cartesian widgets: they draw in pixel space inside
// plot_view.plot_rect() and only use draw_axes for the title.

use crate::plot_view::*;
use crate::script_util::*;
use crate::types::*;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*

    mod.plot.TreemapBase = #(Treemap::register_widget(vm))

    mod.plot.Treemap = set_type_default() do mod.plot.TreemapBase{
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

    mod.plot.SankeyDiagramBase = #(SankeyDiagram::register_widget(vm))

    mod.plot.SankeyDiagram = set_type_default() do mod.plot.SankeyDiagramBase{
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

// ============================================================================
// Treemap
// ============================================================================

/// A labeled value tile in a treemap
#[derive(Clone, Debug)]
pub struct TreemapNode {
    pub label: String,
    pub value: f64,
    pub color: Option<Vec4>,
}

impl TreemapNode {
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
pub struct Treemap {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub nodes: Vec<TreemapNode>,

    #[live(true)]
    pub show_labels: bool,
    #[live(true)]
    pub show_values: bool,
    #[live(true)]
    pub demo_data: bool,

    #[rust]
    fitted: bool,
}

impl Treemap {
    // ---- Rust-side API (mirrors the 1.0 library surface) ----

    pub fn set_data(&mut self, nodes: Vec<TreemapNode>) {
        self.nodes = nodes;
        self.fitted = false;
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn set_show_labels(&mut self, show: bool) {
        self.show_labels = show;
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.fitted = false;
    }

    // ---- Demo data ----

    fn make_demo_data(&mut self) {
        let data = [
            ("Alpha", 38.0),
            ("Beta", 24.0),
            ("Gamma", 15.0),
            ("Delta", 10.0),
            ("Epsilon", 8.0),
            ("Zeta", 5.0),
        ];
        for (label, value) in data {
            self.nodes.push(TreemapNode::new(label, value));
        }
    }

    // ---- Drawing ----

    fn draw_tiles(&mut self, cx: &mut Cx2d) {
        let pr = *self.plot_view.plot_rect();
        let plot_left = pr.pos.x;
        let plot_top = pr.pos.y;
        let plot_width = pr.size.x;
        let plot_height = pr.size.y;
        if plot_width <= 0.0 || plot_height <= 0.0 || self.nodes.is_empty() {
            return;
        }

        let total: f64 = self.nodes.iter().map(|n| n.value.max(0.0)).sum();
        if total <= 0.0 {
            return;
        }

        // Old slice/dice layout: slice along the longer plot axis, each tile
        // taking the full extent of the other axis.
        let area = plot_width * plot_height;
        let mut x = plot_left;
        let mut y = plot_top;
        let mut remaining_width = plot_width;
        let mut remaining_height = plot_height;
        let horizontal = plot_width > plot_height;

        let nodes = self.nodes.clone();
        for (i, node) in nodes.iter().enumerate() {
            let node_area = (node.value.max(0.0) / total) * area;
            let (node_x, node_y, node_w, node_h) = if horizontal {
                let w = if remaining_height > 0.0 {
                    node_area / remaining_height
                } else {
                    0.0
                };
                let w = w.min(remaining_width);
                let result = (x, y, w, remaining_height);
                x += w;
                remaining_width -= w;
                result
            } else {
                let h = if remaining_width > 0.0 {
                    node_area / remaining_width
                } else {
                    0.0
                };
                let h = h.min(remaining_height);
                let result = (x, y, remaining_width, h);
                y += h;
                remaining_height -= h;
                result
            };

            if node_w > 2.0 && node_h > 2.0 {
                let color = node.color.unwrap_or_else(|| cycle_color(i));

                // Filled tile
                self.plot_view.fill_rect_px(
                    node_x as f32,
                    node_y as f32,
                    (node_w - 2.0) as f32,
                    (node_h - 2.0) as f32,
                    color,
                );

                // White border
                self.plot_view.stroke_polygon_px(
                    &[
                        (node_x as f32, node_y as f32),
                        ((node_x + node_w - 2.0) as f32, node_y as f32),
                        (
                            (node_x + node_w - 2.0) as f32,
                            (node_y + node_h - 2.0) as f32,
                        ),
                        (node_x as f32, (node_y + node_h - 2.0) as f32),
                    ],
                    vec4(1.0, 1.0, 1.0, 0.8),
                    1.0,
                );

                // Label + value text with contrast-aware color
                if self.show_labels && node_w > 40.0 && node_h > 25.0 {
                    let center_x = node_x + node_w * 0.5;
                    let center_y = node_y + node_h * 0.5;
                    let brightness = color.x * 0.299 + color.y * 0.587 + color.z * 0.114;
                    let text_color = if brightness > 0.5 {
                        vec4(0.0, 0.0, 0.0, 1.0)
                    } else {
                        vec4(1.0, 1.0, 1.0, 1.0)
                    };
                    let show_value = self.show_values && node_h > 44.0;
                    let label_y = if show_value { center_y - 8.0 } else { center_y };
                    self.plot_view.draw_text_centered_px(
                        cx,
                        center_x,
                        label_y,
                        &node.label,
                        text_color,
                        10.0,
                    );
                    if show_value {
                        let value_text = format_treemap_value(node.value);
                        self.plot_view.draw_text_centered_px(
                            cx,
                            center_x,
                            center_y + 8.0,
                            &value_text,
                            text_color,
                            9.0,
                        );
                    }
                }
            }
        }
    }
}

fn format_treemap_value(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 {
        format!("{:.0}", v)
    } else {
        format!("{:.1}", v)
    }
}

impl Widget for Treemap {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.nodes.is_empty() && self.demo_data {
            self.make_demo_data();
        }
        if !self.fitted {
            self.plot_view.set_viewport(0.0, 1.0, 0.0, 1.0);
            self.fitted = true;
        }

        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx); // title only (grid/ticks/border disabled)
        self.draw_tiles(cx);
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
                let mut nodes = Vec::with_capacity(n);
                for i in 0..n {
                    nodes.push(TreemapNode::new(labels[i].clone(), values[i]));
                }
                self.set_data(nodes);
            }
            x if x == live_id!(set_title) => {
                self.plot_view.title = script_arg_string(vm, &args, 0).unwrap_or_default();
            }
            x if x == live_id!(set_show_labels) => {
                self.show_labels = script_arg_bool(vm, &args, 0).unwrap_or(true);
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

// ============================================================================
// SankeyDiagram
// ============================================================================

/// A node (vertical bar) in a Sankey flow diagram
#[derive(Clone, Debug)]
pub struct SankeyNode {
    pub name: String,
    pub layer: usize,
    pub value: f64,
    pub color: Vec4,
    // Layout computed values (normalized 0..1 within the chart height)
    y: f64,
    height: f64,
}

impl SankeyNode {
    pub fn new(name: impl Into<String>, layer: usize, value: f64, color: Vec4) -> Self {
        Self {
            name: name.into(),
            layer,
            value,
            color,
            y: 0.0,
            height: 0.0,
        }
    }
}

/// A flow between two Sankey nodes
#[derive(Clone, Debug)]
pub struct SankeyLink {
    pub source: usize,
    pub target: usize,
    pub value: f64,
    // Layout computed values (normalized 0..1)
    source_y: f64,
    target_y: f64,
}

impl SankeyLink {
    pub fn new(source: usize, target: usize, value: f64) -> Self {
        Self {
            source,
            target,
            value,
            source_y: 0.0,
            target_y: 0.0,
        }
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct SankeyDiagram {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    pub plot_view: PlotView,

    #[rust]
    pub nodes: Vec<SankeyNode>,
    #[rust]
    pub links: Vec<SankeyLink>,

    #[live(24.0)]
    pub node_width: f64,
    #[live(true)]
    pub show_labels: bool,
    #[live(true)]
    pub demo_data: bool,

    #[rust]
    layout_dirty: bool,
    #[rust]
    fitted: bool,
}

impl SankeyDiagram {
    // ---- Rust-side API ----

    /// Add a node by label; layer and value are auto-derived from links.
    /// Returns the node index.
    pub fn add_node(&mut self, label: impl Into<String>) -> usize {
        let idx = self.nodes.len();
        self.nodes
            .push(SankeyNode::new(label, 0, 0.0, cycle_color(idx)));
        self.layout_dirty = true;
        idx
    }

    /// Add a flow from node `source` to node `target` with the given value.
    pub fn add_link(&mut self, source: usize, target: usize, value: f64) {
        if source >= self.nodes.len() || target >= self.nodes.len() || source == target {
            return;
        }
        self.links.push(SankeyLink::new(source, target, value));
        self.layout_dirty = true;
    }

    /// One-shot setter: replaces all nodes and links.
    pub fn set_data(&mut self, nodes: Vec<SankeyNode>, links: Vec<SankeyLink>) {
        self.nodes = nodes;
        self.links = links;
        self.layout_dirty = true;
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.plot_view.title = title.into();
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.links.clear();
        self.layout_dirty = true;
    }

    // ---- Demo data ----

    fn make_demo_data(&mut self) {
        let coal = self.add_node("Coal");
        let solar = self.add_node("Solar");
        let wind = self.add_node("Wind");
        let elec = self.add_node("Electricity");
        let heat = self.add_node("Heat");
        let industry = self.add_node("Industry");
        let homes = self.add_node("Homes");
        self.add_link(coal, elec, 30.0);
        self.add_link(solar, elec, 20.0);
        self.add_link(wind, elec, 25.0);
        self.add_link(coal, heat, 15.0);
        self.add_link(elec, industry, 40.0);
        self.add_link(elec, homes, 35.0);
        self.add_link(heat, industry, 10.0);
        self.add_link(heat, homes, 5.0);
    }

    // ---- Layout (ported from the 1.0 implementation) ----

    /// Assign each node a column (layer) as the longest link-path distance
    /// from any pure-source node.
    fn compute_layers(&mut self) {
        let n = self.nodes.len();
        let mut layers = vec![0usize; n];
        // Relaxation, bounded by node count to survive accidental cycles
        for _ in 0..n.max(1) {
            let mut changed = false;
            for link in &self.links {
                if layers[link.target] < layers[link.source] + 1 {
                    layers[link.target] = layers[link.source] + 1;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        for (i, node) in self.nodes.iter_mut().enumerate() {
            node.layer = layers[i];
        }
    }

    fn compute_layout(&mut self) {
        self.layout_dirty = false;
        if self.nodes.is_empty() {
            return;
        }

        self.compute_layers();

        // Incoming and outgoing totals
        let mut incoming_totals: Vec<f64> = vec![0.0; self.nodes.len()];
        let mut outgoing_totals: Vec<f64> = vec![0.0; self.nodes.len()];
        for link in &self.links {
            incoming_totals[link.target] += link.value;
            outgoing_totals[link.source] += link.value;
        }

        // Node values: layer-0 nodes flow out, later nodes are sized by inflow
        for i in 0..self.nodes.len() {
            if self.nodes[i].layer == 0 {
                self.nodes[i].value = outgoing_totals[i];
            } else {
                self.nodes[i].value = incoming_totals[i];
            }
        }

        let max_layer = self.nodes.iter().map(|n| n.layer).max().unwrap_or(0);

        // Layout each layer: stack nodes with proportional heights + gaps
        for layer in 0..=max_layer {
            let layer_nodes: Vec<usize> = self
                .nodes
                .iter()
                .enumerate()
                .filter(|(_, n)| n.layer == layer)
                .map(|(i, _)| i)
                .collect();

            let total_value: f64 = layer_nodes.iter().map(|&i| self.nodes[i].value).sum();

            let mut y = 0.0;
            let gap_fraction = 0.08;

            for &idx in &layer_nodes {
                let node_value = self.nodes[idx].value;
                let height = if total_value > 0.0 {
                    node_value / total_value
                        * (1.0 - gap_fraction * (layer_nodes.len() - 1) as f64)
                } else {
                    0.0
                };
                self.nodes[idx].y = y;
                self.nodes[idx].height = height;
                y += height + gap_fraction;
            }
        }

        // Per-node totals used to apportion link thickness
        let source_totals: Vec<f64> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                if node.layer == 0 {
                    node.value
                } else {
                    incoming_totals[i]
                }
            })
            .collect();

        // Compute link attachment offsets along each node
        let mut source_offsets: Vec<f64> = vec![0.0; self.nodes.len()];
        let mut target_offsets: Vec<f64> = vec![0.0; self.nodes.len()];

        for link in &mut self.links {
            let source_idx = link.source;
            let target_idx = link.target;

            link.source_y = self.nodes[source_idx].y + source_offsets[source_idx];
            link.target_y = self.nodes[target_idx].y + target_offsets[target_idx];

            let source_total = source_totals[source_idx];
            let target_total = incoming_totals[target_idx];

            if source_total > 0.0 {
                source_offsets[source_idx] +=
                    link.value / source_total * self.nodes[source_idx].height;
            }
            if target_total > 0.0 {
                target_offsets[target_idx] +=
                    link.value / target_total * self.nodes[target_idx].height;
            }
        }
    }

    // ---- Drawing ----

    fn draw_diagram(&mut self, cx: &mut Cx2d) {
        let pr = *self.plot_view.plot_rect();
        let chart_x = pr.pos.x;
        let chart_y = pr.pos.y;
        let chart_width = pr.size.x;
        let chart_height = pr.size.y;
        if chart_width <= 0.0 || chart_height <= 0.0 || self.nodes.is_empty() {
            return;
        }

        let max_layer = self.nodes.iter().map(|n| n.layer).max().unwrap_or(0);
        let node_width = self.node_width.max(2.0);
        let layer_spacing = if max_layer > 0 {
            (chart_width - node_width) / max_layer as f64
        } else {
            chart_width
        };

        // Precompute totals for link thickness apportioning
        let mut incoming_totals: Vec<f64> = vec![0.0; self.nodes.len()];
        for link in &self.links {
            incoming_totals[link.target] += link.value;
        }
        let source_totals: Vec<f64> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                if node.layer == 0 {
                    node.value
                } else {
                    incoming_totals[i]
                }
            })
            .collect();

        // Curved link ribbons: smoothstep-interpolated quads, per-segment
        // color gradient between the endpoint node colors.
        let links = self.links.clone();
        for link in &links {
            let source = self.nodes[link.source].clone();
            let target = self.nodes[link.target].clone();

            let sx = chart_x + source.layer as f64 * layer_spacing + node_width;
            let sy = chart_y + link.source_y * chart_height;
            let tx = chart_x + target.layer as f64 * layer_spacing;
            let ty = chart_y + link.target_y * chart_height;

            let source_total = source_totals[link.source];
            let target_total = incoming_totals[link.target];

            let link_height_source = if source_total > 0.0 {
                (link.value / source_total) * source.height * chart_height
            } else {
                0.0
            };
            let link_height_target = if target_total > 0.0 {
                (link.value / target_total) * target.height * chart_height
            } else {
                0.0
            };

            let segments = 24;
            for i in 0..segments {
                let t1 = i as f64 / segments as f64;
                let t2 = (i + 1) as f64 / segments as f64;

                let ease1 = t1 * t1 * (3.0 - 2.0 * t1);
                let ease2 = t2 * t2 * (3.0 - 2.0 * t2);

                let x1 = sx + (tx - sx) * t1;
                let x2 = sx + (tx - sx) * t2;
                let y1_top = sy + (ty - sy) * ease1;
                let y2_top = sy + (ty - sy) * ease2;

                let h1 = link_height_source + (link_height_target - link_height_source) * ease1;
                let h2 = link_height_source + (link_height_target - link_height_source) * ease2;

                let t_color = t1 as f32;
                let color = vec4(
                    source.color.x + (target.color.x - source.color.x) * t_color,
                    source.color.y + (target.color.y - source.color.y) * t_color,
                    source.color.z + (target.color.z - source.color.z) * t_color,
                    0.55,
                );

                self.plot_view.fill_polygon_px(
                    &[
                        (x1 as f32, y1_top as f32),
                        (x2 as f32, y2_top as f32),
                        (x2 as f32, (y2_top + h2) as f32),
                        (x1 as f32, (y1_top + h1) as f32),
                    ],
                    color,
                );
            }
        }

        // Node bars
        let nodes = self.nodes.clone();
        for node in &nodes {
            let x = chart_x + node.layer as f64 * layer_spacing;
            let y = chart_y + node.y * chart_height;
            let height = node.height * chart_height;
            if height <= 0.0 {
                continue;
            }
            self.plot_view.fill_rect_px(
                x as f32,
                y as f32,
                node_width as f32,
                height.max(1.0) as f32,
                node.color,
            );
        }

        // Node labels (to the right of the bar; last column to the left)
        if self.show_labels {
            let font_size = 9.0f32;
            let text_color = vec4(0.2, 0.2, 0.2, 1.0);
            for node in &nodes {
                if node.height <= 0.0 || node.name.is_empty() {
                    continue;
                }
                let x = chart_x + node.layer as f64 * layer_spacing;
                let mid_y =
                    chart_y + (node.y + node.height * 0.5) * chart_height - font_size as f64 * 0.6;
                if node.layer == max_layer && max_layer > 0 {
                    let est_w = node.name.len() as f64 * font_size as f64 * 0.5;
                    self.plot_view.draw_text_px(
                        cx,
                        x - est_w - 4.0,
                        mid_y,
                        &node.name,
                        text_color,
                        font_size,
                    );
                } else {
                    self.plot_view.draw_text_px(
                        cx,
                        x + node_width + 4.0,
                        mid_y,
                        &node.name,
                        text_color,
                        font_size,
                    );
                }
            }
        }
    }
}

impl Widget for SankeyDiagram {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.plot_view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.nodes.is_empty() && self.demo_data {
            self.make_demo_data();
        }
        if self.layout_dirty {
            self.compute_layout();
        }
        if !self.fitted {
            self.plot_view.set_viewport(0.0, 1.0, 0.0, 1.0);
            self.fitted = true;
        }

        self.plot_view.begin(cx, walk);
        self.plot_view.draw_axes(cx); // title only (grid/ticks/border disabled)
        self.draw_diagram(cx);
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
            x if x == live_id!(add_node) => {
                let label = script_arg_string(vm, &args, 0).unwrap_or_default();
                self.add_node(label);
            }
            x if x == live_id!(add_link) => {
                let source = script_arg_f64(vm, &args, 0).unwrap_or(-1.0);
                let target = script_arg_f64(vm, &args, 1).unwrap_or(-1.0);
                let value = script_arg_f64(vm, &args, 2).unwrap_or(0.0);
                if source >= 0.0 && target >= 0.0 {
                    self.add_link(source as usize, target as usize, value);
                }
            }
            x if x == live_id!(set_data) => {
                let labels = script_arg_string_array(vm, &args, 0).unwrap_or_default();
                let sources = script_arg_f64_array(vm, &args, 1).unwrap_or_default();
                let targets = script_arg_f64_array(vm, &args, 2).unwrap_or_default();
                let values = script_arg_f64_array(vm, &args, 3).unwrap_or_default();
                self.clear();
                for label in labels {
                    self.add_node(label);
                }
                let n = sources.len().min(targets.len()).min(values.len());
                for i in 0..n {
                    if sources[i] >= 0.0 && targets[i] >= 0.0 {
                        self.add_link(sources[i] as usize, targets[i] as usize, values[i]);
                    }
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
