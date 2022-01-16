use crate::{datetime, definitions, display::Display, feed::Feed, state::State};
use anyhow::Result;
use embedded_graphics::{
    geometry::{Point, Size},
    image::{Image, ImageRawBE},
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{
        Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle,
    },
    text::{Alignment, Text},
};
use embedded_text::{style::TextBoxStyleBuilder, TextBox};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

pub struct FeedPage;
pub struct ExamplePage;
pub struct ConfigPage;

pub enum PageType {
    FeedPage,
    ExamplePage,
    ConfigPage,
}

pub fn draw_pages(display: &mut Display, state: Arc<Mutex<State>>) -> Result<()> {
    loop {
        {
            let state = state.lock().unwrap();
            let page = state.page();

            display.clear();
            page.draw(display, &state).unwrap();
            display.flush().unwrap();
        }

        // std::thread::sleep(std::time::Duration::from_secs(0xFFFF_FFFF_FFFF_FFFF));
        std::thread::sleep(Duration::from_secs(1));
    }
}

pub trait Page<D>: Send + Sync
where
    D: DrawTarget<Color = BinaryColor> + Dimensions,
    D::Color: From<BinaryColor>,
{
    fn draw(&self, display: &mut D, state: &State) -> Result<(), D::Error>;
    fn next_page(&self) -> PageType;
}

impl<D> Page<D> for FeedPage
where
    D: DrawTarget<Color = BinaryColor> + Dimensions,
    D::Color: From<BinaryColor>,
{
    fn draw(&self, display: &mut D, _: &State) -> Result<(), D::Error> {
        Ok(())
    }

    fn next_page(&self) -> PageType {
        PageType::ExamplePage
    }
}

impl<D> Page<D> for ExamplePage
where
    D: DrawTarget<Color = BinaryColor> + Dimensions,
    D::Color: From<BinaryColor>,
{
    fn draw(&self, display: &mut D, _: &State) -> Result<(), D::Error> {
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

        if let Ok(datetime) = datetime::get_datetime() {
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

        // Picture PoC
        // use std::io::Read;
        // let mut icon = std::fs::File::open("/mnt/bw.tga").unwrap();
        // let mut buf = Vec::new();
        // icon.read_to_end(&mut buf).unwrap();
        // let tga = tinytga::DynamicTga::from_slice(&buf).unwrap();
        // let image = Image::new(&tga, Point::zero());
        // image.draw(display)?;

        Ok(())
    }

    fn next_page(&self) -> PageType {
        PageType::FeedPage
    }
}

impl<D> Page<D> for ConfigPage
where
    D: DrawTarget<Color = BinaryColor> + Dimensions,
    D::Color: From<BinaryColor>,
{
    fn draw(&self, display: &mut D, state: &State) -> Result<(), D::Error> {
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

        let textbox_style = TextBoxStyleBuilder::new()
            .alignment(embedded_text::alignment::HorizontalAlignment::Center)
            .vertical_alignment(embedded_text::alignment::VerticalAlignment::Middle)
            .build();

        TextBox::with_textbox_style(
            &format!(
                "Setup Mode\n\nSSID: {}\nPassword: {}\nIP: 192.168.71.1",
                definitions::AP_SSID,
                definitions::AP_PASSWORD
            ),
            display.bounding_box(),
            character_style_text,
            textbox_style,
        )
        .draw(display)?;

        Ok(())
    }

    fn next_page(&self) -> PageType {
        PageType::ConfigPage
    }
}
