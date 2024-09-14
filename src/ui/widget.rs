#![allow(dead_code)]

use std::sync::Arc;

pub trait Widget {
    fn min_size(&mut self) -> epaint::Vec2 {
        epaint::Vec2::new(0.0, 0.0)
    }

    fn layout(&mut self, bounds: epaint::Rect);

    fn bounds(&self) -> epaint::Rect;

    fn shapes(&self) -> Vec<epaint::ClippedShape>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct NodeId(usize);

struct Node {
    parent_id: Option<NodeId>,
    children: Vec<NodeId>,
    widget: Box<dyn Widget>,
}

impl Node {
    pub fn layout(&mut self, bounds: epaint::Rect) {
        self.widget.layout(bounds);
    }
}

#[derive(Default)]
struct NodeTree {
    nodes: std::collections::HashMap<NodeId, Node>,
    next_node_id: NodeId,
}

impl NodeTree {
    pub fn layout(&mut self, bounds: epaint::Rect) {
        self.nodes
            .iter_mut()
            .filter(|(_, node)| node.parent_id.is_none())
            .for_each(|(_, node)| node.layout(bounds));
    }
}

pub struct Label {
    bounds: epaint::Rect,
    color: epaint::Color32,
    font: epaint::FontId,
    galley: Arc<epaint::Galley>,
}

impl Label {
    pub fn new(
        fonts: &epaint::Fonts,
        text: impl Into<String>,
        font: epaint::FontId,
        color: epaint::Color32,
    ) -> Self {
        let galley = fonts.layout_no_wrap(text.into(), font.clone(), color);
        Self {
            bounds: galley.rect,
            color,
            font,
            galley,
        }
    }
}

impl Label {
    pub fn set_text(&mut self, fonts: &epaint::Fonts, text: impl Into<String>) {
        let galley = fonts.layout_no_wrap(text.into(), self.font.clone(), self.color);
        self.galley = galley;
    }
}

impl Widget for Label {
    fn min_size(&mut self) -> epaint::Vec2 {
        self.galley.size()
    }

    fn layout(&mut self, bounds: epaint::Rect) {
        self.bounds = bounds
    }

    fn bounds(&self) -> epaint::Rect {
        self.galley.rect
    }

    fn shapes(&self) -> Vec<epaint::ClippedShape> {
        let text_shape = epaint::ClippedShape {
            clip_rect: epaint::Rect::EVERYTHING,
            shape: epaint::Shape::Text(epaint::TextShape::new(
                self.bounds.min,
                Arc::clone(&self.galley),
                self.color,
            )),
        };
        vec![text_shape]
    }
}

slotmap::new_key_type! {
    pub struct SliderId;
}

pub struct Slider {
    fonts: Arc<epaint::Fonts>,
    pub bounds: epaint::Rect,
    label_galley: Arc<epaint::Galley>,
    color: epaint::Color32,
    value: f32,
    min: f32,
    max: f32,

    // If the user clicks in this area, we will start to drag the knob.
    slide_bounds: epaint::Rect,
}

impl Slider {
    pub fn new(fonts: Arc<epaint::Fonts>, label: impl Into<String>) -> Self {
        let label_galley = fonts.layout_no_wrap(
            label.into(),
            epaint::FontId::monospace(16.0),
            epaint::Color32::WHITE,
        );
        Self {
            fonts,
            bounds: epaint::Rect::ZERO,
            label_galley,
            color: epaint::Color32::LIGHT_GREEN,
            value: 0.5,
            min: 0.0,
            max: 1.0,
            slide_bounds: epaint::Rect::ZERO,
        }
    }

    pub fn with_min_max(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self.value = self.value.clamp(min, max);
        self
    }

    pub fn with_value(mut self, value: f32) -> Self {
        self.set_value(value);
        self
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(self.min, self.max);
    }

    pub fn shapes(&mut self) -> Vec<epaint::ClippedShape> {
        const PADDING: f32 = 10.0;
        // const SLIDE_THICKNESS: f32 = 20.0;

        // let background_shape = epaint::ClippedShape {
        //     clip_rect: epaint::Rect::EVERYTHING,
        //     shape: epaint::Shape::rect_filled(
        //         self.bounds,
        //         epaint::Rounding::ZERO,
        //         epaint::Color32::DARK_GREEN,
        //     ),
        // };

        let v_center = self.bounds.height() / 2.0;

        let label_position = self.bounds.min
            + epaint::vec2(PADDING, v_center - self.label_galley.rect.height() / 2.0);
        let label_shape = epaint::ClippedShape {
            clip_rect: epaint::Rect::EVERYTHING,
            shape: epaint::Shape::galley(
                label_position,
                Arc::clone(&self.label_galley),
                self.color,
            ),
        };

        self.slide_bounds = epaint::Rect {
            min: epaint::pos2(
                self.bounds.min.x + PADDING * 2.0 + self.label_galley.rect.width(),
                self.bounds.min.y + v_center - self.label_galley.rect.height() / 2.0,
            ),
            max: epaint::pos2(
                self.bounds.max.x - PADDING,
                self.bounds.min.y + v_center + self.label_galley.rect.height() / 2.0,
            ),
        };

        let slide = epaint::ClippedShape {
            clip_rect: epaint::Rect::EVERYTHING,
            shape: epaint::Shape::rect_filled(
                self.slide_bounds,
                epaint::Rounding::ZERO,
                epaint::Color32::DARK_GRAY,
            ),
        };

        let t = (self.value - self.min) / (self.max - self.min);
        let progress_bounds = epaint::Rect {
            min: self.slide_bounds.min,
            max: epaint::pos2(
                self.slide_bounds.min.x + (self.slide_bounds.width() * t),
                self.slide_bounds.max.y,
            ),
        };

        let progress = epaint::ClippedShape {
            clip_rect: epaint::Rect::EVERYTHING,
            shape: epaint::Shape::rect_filled(
                progress_bounds,
                epaint::Rounding::ZERO,
                epaint::Color32::LIGHT_GREEN,
            ),
        };

        let progress_label = self.fonts.layout_no_wrap(
            format!("{}", self.value),
            epaint::FontId::monospace(14.0),
            epaint::Color32::BLACK,
        );
        let progress_label = epaint::ClippedShape {
            clip_rect: epaint::Rect::EVERYTHING,
            shape: epaint::Shape::galley(
                self.slide_bounds.min + self.slide_bounds.size() / 2.0
                    - progress_label.rect.size() / 2.0,
                progress_label,
                epaint::Color32::default(),
            ),
        };

        vec![
            //background_shape,
            label_shape,
            slide,
            progress,
            progress_label,
        ]
    }

    pub fn on_mouse_down(&mut self, x: f32, y: f32) -> bool {
        if self.slide_bounds.contains(epaint::pos2(x, y)) {
            self.update_value(x);
            return true;
        }

        false
    }

    pub fn on_mouse_up(&mut self) {}

    pub fn on_mouse_dragged(&mut self, x: f32, _y: f32) {
        if x < self.slide_bounds.min.x {
            self.value = self.min;
            return;
        }

        if x > self.slide_bounds.max.x {
            self.value = self.max;
            return;
        }

        self.update_value(x);
    }

    fn update_value(&mut self, x: f32) {
        let t = (x - self.slide_bounds.min.x) / self.slide_bounds.width();
        self.value = self.min + (self.max - self.min) * t;
    }

    pub fn on_mouse_moved(&mut self, _x: f32, _y: f32) {}
}
