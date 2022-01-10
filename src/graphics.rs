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
use crate::display::Display;

// static border_stroke: PrimitiveStyle<BinaryColor> = PrimitiveStyleBuilder::new()
//     .stroke_color(BinaryColor::On)
//     .stroke_width(3)
//     .stroke_alignment(StrokeAlignment::Inside)
//     .build();

pub fn draw_page(display: &mut Display, page: Box<dyn Page<Display>>) {
    loop {
        page.draw(display).unwrap();
        display.flush().unwrap();
        // std::thread::sleep(std::time::Duration::from_secs(0xFFFF_FFFF_FFFF_FFFF));
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}

pub trait Page<D> : Send
where
    D: DrawTarget<Color = BinaryColor> + Dimensions,
    D::Color: From<BinaryColor>,
{
    fn draw(&self, display: &mut D) -> Result<(), D::Error>;
}

pub struct FeedPage;
pub struct ExamplePage;
pub struct ConfigPage;

impl<D> Page<D> for FeedPage
where
    D: DrawTarget<Color = BinaryColor> + Dimensions,
    D::Color: From<BinaryColor>,
{
    fn draw(&self, display: &mut D) -> Result<(), D::Error> {
        Ok(())
    }
}

impl<D> Page<D> for ExamplePage
where
    D: DrawTarget<Color = BinaryColor> + Dimensions,
    D::Color: From<BinaryColor>,
{
    fn draw(&self, display: &mut D) -> Result<(), D::Error> {
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

impl<D> Page<D> for ConfigPage
where
    D: DrawTarget<Color = BinaryColor> + Dimensions,
    D::Color: From<BinaryColor>,
{
    fn draw(&self, display: &mut D) -> Result<(), D::Error> {
        let border_stroke = PrimitiveStyleBuilder::new()
            .stroke_color(BinaryColor::On)
            .stroke_width(3)
            .stroke_alignment(StrokeAlignment::Inside)
            .build();

        let character_style_text = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        display
            .bounding_box()
            .into_styled(border_stroke)
            .draw(display)?;

        Text::with_alignment(
            "ESP-Feed\nSetup Mode\nSSID: \"ESP-Feed\"\nPassword: \"38294446\"\nIP: 192.168.71.1",
            display.bounding_box().center() - Point::new(0, 17),
            character_style_text,
            Alignment::Center,
        )
        .draw(display)?;

        Ok(())
    }
}
