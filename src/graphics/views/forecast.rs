use crate::{
    graphics::style,
    storage::{ReadFile, BASE_DIR},
};
use embedded_graphics::{
    draw_target::DrawTarget,
    image::Image,
    pixelcolor::BinaryColor,
    prelude::{Dimensions, Point, Primitive, Size},
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
    Drawable,
};
use embedded_layout::{
    align::horizontal,
    layout::linear::{spacing::DistributeFill, LinearLayout},
    prelude::*,
};
use std::fs::File;
use tinytga::DynamicTga;

#[derive(Clone)]
pub struct Forecast<'a> {
    icon_code: &'a str,
    datetime: String,
    temperature: f32,
    bounds: Rectangle,
}

impl<'a> Forecast<'a> {
    pub fn new(icon: &'a str, datetime: String, temperature: f32) -> Self {
        Self {
            icon_code: icon,
            datetime,
            temperature,
            bounds: Rectangle::new(Point::zero(), Size::new(42, 52)),
        }
    }
}

impl<'a> View for Forecast<'a> {
    fn translate_impl(&mut self, by: Point) {
        self.bounds.translate_mut(by);
    }

    fn bounds(&self) -> Rectangle {
        self.bounds
    }
}

impl<'a> Dimensions for Forecast<'a> {
    fn bounding_box(&self) -> Rectangle {
        self.bounds
    }
}

impl<'a> Drawable for Forecast<'a> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D: DrawTarget<Color = BinaryColor>>(&self, target: &mut D) -> Result<(), D::Error> {
        // Create styles
        let border_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

        // Create a 1px border
        let border = self.bounding_box().into_styled(border_style);

        let mut icon_file =
            File::open(format!("{}/weather/small/{}.tga", BASE_DIR, self.icon_code)).unwrap();
        let raw_bytes = icon_file.raw_bytes();
        let tga_image = DynamicTga::from_slice(&raw_bytes).unwrap();
        let image = Image::new(&tga_image, Point::zero());

        let datetime_text = Text::new(&self.datetime, Point::zero(), style::normal_text());

        let temp = &format!("{:.1}°C", self.temperature);
        let temperature_text = Text::new(temp, Point::zero(), style::normal_text());

        let layout = LinearLayout::vertical(
            Chain::new(datetime_text)
                .append(image)
                .append(temperature_text),
        )
        .with_spacing(DistributeFill(self.bounding_box().size.height - 4))
        .with_alignment(horizontal::Center)
        .arrange()
        .align_to(&border, horizontal::Center, vertical::Center);

        // Draw views
        border.draw(target)?;
        layout.draw(target)?;

        Ok(())
    }
}
