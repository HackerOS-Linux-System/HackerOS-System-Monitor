use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use gtk::cairo::Context;
use gtk::prelude::*;
use gtk::DrawingArea;

#[derive(Clone, Copy)]
pub struct Rgb(pub f64, pub f64, pub f64);

pub const ACCENT_BLUE: Rgb = Rgb(0.30, 0.62, 0.98);
pub const ACCENT_GREEN: Rgb = Rgb(0.30, 0.82, 0.55);
pub const ACCENT_ORANGE: Rgb = Rgb(0.98, 0.62, 0.25);
pub const ACCENT_PURPLE: Rgb = Rgb(0.68, 0.48, 0.98);

/// A single filled line graph with a fixed y-range of `0.0..=max_value`
/// (or auto-scaling when `max_value` is `None`).
#[derive(Clone)]
pub struct Graph {
    pub widget: DrawingArea,
    data: Rc<RefCell<VecDeque<f64>>>,
}

impl Graph {
    pub fn new(color: Rgb, max_value: Option<f64>, capacity: usize) -> Self {
        let mut initial = VecDeque::with_capacity(capacity);
        initial.resize(capacity, 0.0);
        let data = Rc::new(RefCell::new(initial));

        let widget = DrawingArea::new();
        widget.set_content_height(64);
        widget.set_hexpand(true);
        widget.set_vexpand(true);
        widget.add_css_class("monitor-graph");

        let draw_data = data.clone();
        widget.set_draw_func(move |_area, cr, width, height| {
            draw_graph(cr, width, height, &draw_data.borrow(), color, max_value);
        });

        Self { widget, data }
    }

    /// Replace the buffer wholesale (used by views that keep their own
    /// [`crate::stats::History`] and copy into the graph each refresh).
    pub fn set_data(&self, samples: &VecDeque<f64>) {
        {
            let mut d = self.data.borrow_mut();
            d.clear();
            d.extend(samples.iter().copied());
        }
        self.widget.queue_draw();
    }
}

fn draw_graph(
    cr: &Context,
    width: i32,
    height: i32,
    samples: &VecDeque<f64>,
    color: Rgb,
    max_value: Option<f64>,
) {
    let w = width as f64;
    let h = height as f64;

    // Background.
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
    let _ = cr.paint();

    // Horizontal gridlines.
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.07);
    cr.set_line_width(1.0);
    for i in 1..4 {
        let y = h * (i as f64) / 4.0;
        cr.move_to(0.0, y);
        cr.line_to(w, y);
    }
    let _ = cr.stroke();

    if samples.is_empty() {
        return;
    }

    let peak = max_value.unwrap_or_else(|| {
        samples
            .iter()
            .copied()
            .fold(1.0_f64, |a, b| if b > a { b } else { a })
    });
    let peak = if peak <= 0.0 { 1.0 } else { peak };

    let n = samples.len();
    let step = if n > 1 { w / (n as f64 - 1.0) } else { w };

    let point_y = |v: f64| -> f64 { h - (v.clamp(0.0, peak) / peak) * h };

    // Filled area under the line.
    cr.move_to(0.0, h);
    for (i, v) in samples.iter().enumerate() {
        cr.line_to(i as f64 * step, point_y(*v));
    }
    cr.line_to((n - 1) as f64 * step, h);
    cr.close_path();
    let Rgb(r, g, b) = color;
    cr.set_source_rgba(r, g, b, 0.22);
    let _ = cr.fill_preserve();

    // Line stroke on top.
    cr.new_path();
    for (i, v) in samples.iter().enumerate() {
        let x = i as f64 * step;
        let y = point_y(*v);
        if i == 0 {
            cr.move_to(x, y);
        } else {
            cr.line_to(x, y);
        }
    }
    cr.set_source_rgba(r, g, b, 0.95);
    cr.set_line_width(1.6);
    cr.set_line_join(gtk::cairo::LineJoin::Round);
    let _ = cr.stroke();
}
