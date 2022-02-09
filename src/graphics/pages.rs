use crate::{
    datetime, definitions,
    graphics::{
        style,
        views::{
            feed_group::FeedGroup,
            forecast_row::{ForecastRow, ForecastType},
        },
    },
    state::State,
    storage::{ReadFile, BASE_DIR},
};
use anyhow::Result;
use embedded_graphics::{
    geometry::AnchorPoint,
    geometry::{Point, Size},
    image::Image,
    mono_font::{iso_8859_1::*, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle, Triangle},
    text::{Alignment, Text},
};
use embedded_layout::{
    layout::linear::{spacing::DistributeFill, FixedMargin, LinearLayout},
    prelude::*,
};
use embedded_text::{style::TextBoxStyleBuilder, TextBox};
use enum_dispatch::enum_dispatch;
use std::fs::File;
use tinytga::DynamicTga;

#[derive(Debug)]
pub struct FeedPage;

#[derive(Debug)]
pub struct WeatherPage;

#[derive(Debug)]
pub struct ExamplePage;

#[derive(Debug)]
pub struct ConfigPage;

#[enum_dispatch(Page)]
#[derive(Debug)]
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
        let mut groups = Vec::new();

        for feed in state.feed_controller.feeds() {
            let feed0_group = FeedGroup::new(feed, target.bounding_box());

            let layout = LinearLayout::vertical(
                Chain::new(feed0_group.title)
                    .append(feed0_group.headline0)
                    .append(feed0_group.headline1)
                    .append(feed0_group.headline2)
                    .append(feed0_group.headline3),
            )
            .with_alignment(horizontal::Left)
            .with_spacing(FixedMargin(3))
            .arrange();

            groups.push(layout);
        }

        LinearLayout::vertical(Views::new(groups.as_mut_slice()))
            .with_alignment(horizontal::Left)
            .with_spacing(FixedMargin(3))
            .arrange()
            .align_to(&target.bounding_box(), horizontal::Left, vertical::Top)
            .translate(Point::new(1, 0)) // TODO fix border bug
            .draw(target)?;

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
        if let Some(ref report) = state.weather_controller.current() {
            let mut icon_file =
                File::open(format!("{}/weather/big/{}.tga", BASE_DIR, report.icon)).unwrap();
            let raw_bytes = icon_file.raw_bytes();
            let tga_image = DynamicTga::from_slice(&raw_bytes).unwrap();

            let text_style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);

            let forecast_row = ForecastRow::new(&state.weather_controller, ForecastType::Hourly)
                .align_to(&target.bounding_box(), horizontal::Left, vertical::Bottom)
                .translate(Point::new(1, 0)) // TODO: Remove when border bug is fixed
                .translate(Point::new(-1, 1));

            let forecast_row_top_right = forecast_row.bounding_box().top_left
                + Point::new(forecast_row.bounding_box().size.width as _, 0);
            let current_layout_box = Rectangle::with_corners(Point::zero(), forecast_row_top_right);

            let description = Text::new(report.description, Point::zero(), text_style).align_to(
                &forecast_row.bounding_box(),
                horizontal::Left,
                vertical::BottomToTop,
            );

            let temperature_humidity_text = format!("{:.1}°C / {}%", report.temp, report.humidity);

            let temperature_humidity =
                Text::new(&temperature_humidity_text, Point::zero(), text_style);

            let weather_icon = Image::new(&tga_image, Point::zero()).align_to(
                &description.bounding_box(),
                horizontal::Left,
                vertical::BottomToTop,
            );

            let icon_temp_hum_layout =
                LinearLayout::horizontal(Chain::new(weather_icon).append(temperature_humidity))
                    .with_alignment(vertical::Center)
                    .with_spacing(FixedMargin(8))
                    .arrange();

            let current_layout =
                LinearLayout::vertical(Chain::new(icon_temp_hum_layout).append(description))
                    .with_alignment(horizontal::Left)
                    .with_spacing(FixedMargin(1))
                    .arrange()
                    .align_to(&current_layout_box, horizontal::Left, vertical::Center)
                    .translate(Point::new(1, 1)) // TODO: Remove when border bug is fixed
                    .translate(Point::new(8, 0));

            forecast_row.draw(target)?;
            current_layout.draw(target)?;
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
                time::format_description::parse("[day].[month].[year]\n[hour]:[minute]:[second]")
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
