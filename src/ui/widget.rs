use std::sync::Arc;

use super::UiContext;

pub trait Widget {
    fn bounds(&self) -> epaint::Rect;
    fn shapes(&self) -> Vec<epaint::ClippedShape>;
}

pub struct Label {
    ui_context: UiContext,
    position: epaint::Pos2,
    color: epaint::Color32,
    font: epaint::FontId,
    galley: Arc<epaint::Galley>,
}

impl Label {
    pub fn new(
        ui_context: UiContext,
        position: epaint::Pos2,
        text: impl Into<String>,
        font: epaint::FontId,
        color: epaint::Color32,
    ) -> Self {
        let galley = ui_context.layout_no_wrap(text.into(), font.clone(), color);
        Self {
            ui_context,
            position,
            color,
            font,
            galley,
        }
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.galley = self
            .ui_context
            .layout_no_wrap(text, self.font.clone(), self.color)
    }
}

impl Widget for Label {
    fn bounds(&self) -> epaint::Rect {
        self.galley.rect
    }

    fn shapes(&self) -> Vec<epaint::ClippedShape> {
        let text_shape = epaint::ClippedShape {
            clip_rect: epaint::Rect::EVERYTHING,
            shape: epaint::Shape::Text(epaint::TextShape::new(
                self.position,
                Arc::clone(&self.galley),
                epaint::Color32::default(),
            )),
        };
        vec![text_shape]
    }
}
