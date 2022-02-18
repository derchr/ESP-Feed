use crate::{
    datetime,
    graphics::{pages::WeatherPageType, views::forecast::Forecast},
    weather::WeatherController,
    weather::WeatherReport, // TODO Weathercontroller forecast ?
};
use embedded_graphics::{
    draw_target::DrawTarget, pixelcolor::BinaryColor, prelude::*, primitives::Rectangle,
};
use embedded_layout::prelude::*;

pub struct ForecastRow<'a> {
    forecast_widgets: [Forecast<'a>; 5],
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

        Self { forecast_widgets }
    }
}

impl<'a> View for ForecastRow<'a> {
    fn translate_impl(&mut self, by: Point) {
        self.forecast_widgets.iter_mut().for_each(|w| {
            w.translate_mut(by);
        });
    }

    fn bounds(&self) -> Rectangle {
        self.bounding_box()
    }
}

impl<'a> Dimensions for ForecastRow<'a> {
    fn bounding_box(&self) -> Rectangle {
        let size = self.forecast_widgets.len();
        self.forecast_widgets[0]
            .bounds()
            .enveloping(&self.forecast_widgets[size].bounds())
    }
}

impl<'a> Drawable for ForecastRow<'a> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D: DrawTarget<Color = BinaryColor>>(&self, target: &mut D) -> Result<(), D::Error> {
        for w in self.forecast_widgets.iter() {
            w.draw(target)?;
        }

        Ok(())
    }
}
