use embedded_graphics::{
    image::Image,
    prelude::*,
    text::{renderer::TextRenderer, Text},
};

pub trait Centering {
    fn center(&self, at: Point) -> Self;
    fn center_mut(&mut self, at: Point) -> &mut Self;
}

impl<S: Clone + TextRenderer> Centering for Text<'_, S> {
    fn center(&self, at: Point) -> Self {
        self.translate(at - self.bounding_box().center())
    }
    fn center_mut(&mut self, at: Point) -> &mut Self {
        self.translate_mut(at - self.bounding_box().center())
    }
}

impl<T: embedded_graphics::geometry::OriginDimensions> Centering for Image<'_, T> {
    fn center(&self, at: Point) -> Self {
        self.translate(at - self.bounding_box().center())
    }

    fn center_mut(&mut self, at: Point) -> &mut Self {
        self.translate_mut(at - self.bounding_box().center())
    }
}
