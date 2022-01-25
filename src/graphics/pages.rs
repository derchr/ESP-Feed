use super::style;
use crate::{datetime, definitions, state::State};

use anyhow::Result;
use embedded_graphics::{
    geometry::AnchorPoint,
    geometry::{Point, Size},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle, Triangle},
    text::{Alignment, Text},
};
use embedded_layout::{
    layout::linear::{spacing::DistributeFill, LinearLayout},
    prelude::*,
};
use embedded_text::{style::TextBoxStyleBuilder, TextBox};
use enum_dispatch::enum_dispatch;

pub struct FeedPage;
pub struct WeatherPage;
pub struct ExamplePage;
pub struct ConfigPage;

#[enum_dispatch(Page)]
pub enum PageType {
    FeedPage,
    WeatherPage,
    ExamplePage,
    ConfigPage,
}

#[enum_dispatch]
pub trait Page: Send + Sync {
    fn draw<D>(&self, target: &mut D, state: &State) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor> + Dimensions,
        D::Color: From<BinaryColor>;

    fn next_page(&self) -> PageType;
}

impl Page for FeedPage {
    fn draw<D>(&self, target: &mut D, state: &State) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor> + Dimensions,
        D::Color: From<BinaryColor>,
    {
        let textbox_style = TextBoxStyleBuilder::new()
            .alignment(embedded_text::alignment::HorizontalAlignment::Center)
            .vertical_alignment(embedded_text::alignment::VerticalAlignment::Middle)
            .build();

        if let Some(feed) = state.feed_controller.feeds().get(0) {
            let text = feed.headlines.get(0).unwrap();
            TextBox::with_textbox_style(
                text,
                target.bounding_box(),
                style::normal_text(),
                textbox_style,
            )
            .draw(target)?;
        }

        Ok(())
    }

    fn next_page(&self) -> PageType {
        WeatherPage.into()
    }
}

impl Page for WeatherPage {
    fn draw<D>(&self, target: &mut D, state: &State) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor> + Dimensions,
        D::Color: From<BinaryColor>,
    {
        let textbox_style = TextBoxStyleBuilder::new()
            .alignment(embedded_text::alignment::HorizontalAlignment::Center)
            .vertical_alignment(embedded_text::alignment::VerticalAlignment::Middle)
            .build();

        if let Some(report) = state.weather_controller.current() {
            let text = &format!(
                "{}\n{}\n\nTemperatur: {}°C",
                report.name, report.description, report.temp
            );
            TextBox::with_textbox_style(
                text,
                target.bounding_box(),
                style::normal_text(),
                textbox_style,
            )
            .draw(target)?;
        }

        Ok(())
    }

    fn next_page(&self) -> PageType {
        ExamplePage.into()
    }
}

impl Page for ExamplePage {
    fn draw<D>(&self, target: &mut D, _: &State) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor> + Dimensions,
        D::Color: From<BinaryColor>,
    {
        let fill = PrimitiveStyle::with_fill(BinaryColor::On);

        let y_offset = 10;

        let bounding_box = target.bounding_box();

        // Draw a 3px wide outline around the display.
        bounding_box
            .into_styled(style::border_stroke())
            .draw(target)?;

        // Draw a triangle.
        let triangle = Triangle::new(Point::new(0, 16), Point::new(16, 16), Point::new(8, 0))
            .into_styled(style::thin_stroke());

        // Draw a filled square
        let rectangle = Rectangle::new(Point::zero(), Size::new(16, 16)).into_styled(fill);

        // Draw a circle with a 3px wide stroke.
        let circle = Circle::new(Point::zero(), 17).into_styled(style::thick_stroke());

        LinearLayout::horizontal(Chain::new(triangle).append(rectangle).append(circle))
            .with_spacing(DistributeFill(bounding_box.size.width - 32))
            .with_alignment(vertical::Center)
            .arrange()
            .align_to(&bounding_box, horizontal::Center, vertical::Top)
            .translate(Point { x: 0, y: y_offset })
            .draw(target)?;

        let textbox_style = TextBoxStyleBuilder::new()
            .alignment(embedded_text::alignment::HorizontalAlignment::Center)
            .vertical_alignment(embedded_text::alignment::VerticalAlignment::Middle)
            .build();

        let text = "embedded-graphics";
        Text::with_alignment(
            text,
            target.bounding_box().center() + Point::new(0, 17),
            style::normal_text(),
            Alignment::Center,
        )
        .draw(target)?;

        if let Ok(datetime) = datetime::get_datetime() {
            let format =
                time::format_description::parse("[day].[month].[year] [hour]:[minute]:[second]")
                    .expect("Invalid format.");

            let time = datetime.format(&format).expect("Could not format time.");

            TextBox::with_textbox_style(
                &time,
                target.bounding_box().resized(
                    Size {
                        width: target.bounding_box().size.width,
                        height: target.bounding_box().size.height / 2,
                    },
                    AnchorPoint::BottomCenter,
                ),
                style::normal_text(),
                textbox_style,
            )
            .draw(target)?;
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
        FeedPage.into()
    }
}

impl Page for ConfigPage {
    fn draw<D>(&self, target: &mut D, _: &State) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor> + Dimensions,
        D::Color: From<BinaryColor>,
    {
        target
            .bounding_box()
            .into_styled(style::border_stroke())
            .draw(target)?;

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
            target.bounding_box(),
            style::normal_text(),
            textbox_style,
        )
        .draw(target)?;

        Ok(())
    }

    fn next_page(&self) -> PageType {
        ConfigPage.into()
    }
}
