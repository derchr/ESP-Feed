use crate::{
    datetime,
    graphics::{pages::WeatherPageType, views::forecast::Forecast},
    weather::{WeatherController, WeatherReport},
};
use embedded_graphics::{
    draw_target::DrawTarget, pixelcolor::BinaryColor, prelude::*, primitives::Rectangle,
};
use embedded_layout::{
    align::vertical::Bottom,
    layout::linear::{FixedMargin, Horizontal, LinearLayout},
    prelude::*,
};

type LayoutOrientation = Horizontal<Bottom, FixedMargin>;
type Layout<'a> = LinearLayout<
    LayoutOrientation,
    Link<
        Forecast<'a>,
        Link<Forecast<'a>, Link<Forecast<'a>, Link<Forecast<'a>, Chain<Forecast<'a>>>>>,
    >,
>;

pub struct ForecastRow<'a> {
    layout: Layout<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum ForecastType {
    Daily,
    Hourly,
}

impl From<WeatherPageType> for ForecastType {
    fn from(page_type: WeatherPageType) -> Self {
        match page_type {
            WeatherPageType::Daily => ForecastType::Daily,
            WeatherPageType::Hourly => ForecastType::Hourly,
        }
    }
}

impl<'a> ForecastRow<'a> {
    pub fn new(controller: &'a WeatherController, forecast_type: ForecastType) -> Self {
        let forecast_widgets: [Forecast; 5] = array_init::array_init(|i| {
            let i = i + 1; // First index is same as current.

            let WeatherReport { dt, icon, temp, .. } = match forecast_type {
                ForecastType::Hourly => controller.hourly(i).unwrap_or_default(),
                ForecastType::Daily => controller.daily(i).unwrap_or_default(),
            };

            let format = match forecast_type {
                ForecastType::Hourly => time::format_description::parse("[hour]:00").unwrap(),
                ForecastType::Daily => time::format_description::parse("[day].[month]").unwrap(),
            };

            let datetime = datetime::get_datetime_from_unix(dt as _).unwrap();
            let time = datetime.format(&format).expect("Could not format time.");

            Forecast::new(icon, time, temp)
        });

        let layout = LinearLayout::horizontal(
            Chain::new(forecast_widgets[0].clone())
                .append(forecast_widgets[1].clone())
                .append(forecast_widgets[2].clone())
                .append(forecast_widgets[3].clone())
                .append(forecast_widgets[4].clone()),
        )
        .with_spacing(FixedMargin(-1))
        .arrange();

        Self { layout }
    }
}

impl<'a> View for ForecastRow<'a> {
    fn translate_impl(&mut self, by: Point) {
        self.layout.translate_mut(by);
    }

    fn bounds(&self) -> Rectangle {
        self.bounding_box()
    }
}

impl<'a> Dimensions for ForecastRow<'a> {
    fn bounding_box(&self) -> Rectangle {
        self.layout.bounds()
    }
}

impl<'a> Drawable for ForecastRow<'a> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D: DrawTarget<Color = BinaryColor>>(&self, target: &mut D) -> Result<(), D::Error> {
        self.layout.draw(target)
    }
}
