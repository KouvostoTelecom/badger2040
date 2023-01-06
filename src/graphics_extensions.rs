use embedded_graphics::{
    prelude::*,
    text::{renderer::TextRenderer, Text},
};

pub trait Centering {
    fn center(&self, at: Point) -> Self;
    fn center_mut(&mut self, at: Point) -> &mut Self;
}

impl<S: Clone + TextRenderer> Centering for Text<'_, S> {
    fn center(&self, at: Point) -> Self {
        Self {
            position: self.position + at - self.bounding_box().center(),
            ..self.clone()
        }
    }
    fn center_mut(&mut self, at: Point) -> &mut Self {
        self.translate_mut(at - self.bounding_box().center())
    }
}
