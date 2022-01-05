use embedded_graphics::{
    geometry::{Point, Size},
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{
        Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle,
    },
    text::{Alignment, Text},
};

use ssd1306::{mode::BufferedGraphicsMode, prelude::*, Ssd1306};

use crate::datetime::*;

pub trait Page {
    fn draw<D>(&self, display: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor> + Dimensions,
        D::Color: From<BinaryColor>;
}

// pub struct FeedPage {}

// impl Page for FeedPage {
//     fn draw<D>(display: &mut D) -> Result<(), D::Error>
//     where
//         D: DrawTarget<Color = BinaryColor> + Dimensions,
//         D::Color: From<BinaryColor>,
//     {
//         Ok(())
//     }
// }

pub fn draw_page<S>(display: &mut Ssd1306<impl WriteOnlyDataCommand, S, BufferedGraphicsMode<S>>)
where
    S: DisplaySize,
{
    let page = ExamplePage {};

    loop {
        page.draw(display).unwrap();
        display.flush().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(150));
    }
}

pub struct ExamplePage {}

impl Page for ExamplePage {
    fn draw<D>(&self, display: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor> + Dimensions,
        D::Color: From<BinaryColor>,
    {
        let thin_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
        let thick_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 3);
        let border_stroke = PrimitiveStyleBuilder::new()
            .stroke_color(BinaryColor::On)
            .stroke_width(3)
            .stroke_alignment(StrokeAlignment::Inside)
            .build();
        let fill = PrimitiveStyle::with_fill(BinaryColor::On);
        let character_style_text = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let yoffset = 10;

        // Draw a 3px wide outline around the display.
        display
            .bounding_box()
            .into_styled(border_stroke)
            .draw(display)?;

        // Draw a triangle.
        Triangle::new(
            Point::new(16, 16 + yoffset),
            Point::new(16 + 16, 16 + yoffset),
            Point::new(16 + 8, yoffset),
        )
        .into_styled(thin_stroke)
        .draw(display)?;

        // Draw a filled square
        Rectangle::new(Point::new(52, yoffset), Size::new(16, 16))
            .into_styled(fill)
            .draw(display)?;

        // Draw a circle with a 3px wide stroke.
        Circle::new(Point::new(88, yoffset), 17)
            .into_styled(thick_stroke)
            .draw(display)?;

        let text = "embedded-graphics";
        Text::with_alignment(
            text,
            display.bounding_box().center() + Point::new(0, 17),
            character_style_text,
            Alignment::Center,
        )
        .draw(display)?;

        if let Ok(datetime) = get_datetime() {
            let format =
                time::format_description::parse("[day].[month].[year] [hour]:[minute]:[second]")
                    .expect("Invalid format.");
            Text::with_alignment(
                &datetime.format(&format).expect("Could not format time."),
                display.bounding_box().center() + Point::new(0, 27),
                character_style_text,
                Alignment::Center,
            )
            .draw(display)?;
        }

        Ok(())
    }
}
