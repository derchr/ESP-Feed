use crate::{
    datetime,
    graphics::views::forecast::Forecast,
    weather::WeatherReport, // TODO Weathercontroller forecast ?
};
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*, primitives::Rectangle};
use embedded_layout::{
    layout::linear::{FixedMargin, LinearLayout},
    prelude::*,
};
// use embedded_layout_macros::ViewGroup;

// #[derive(ViewGroup)]
pub struct ForecastRow<'a> {
    forecast0: Forecast<'a>,
    forecast1: Forecast<'a>,
    forecast2: Forecast<'a>,
    forecast3: Forecast<'a>,
    forecast4: Forecast<'a>,
}

impl<'a> ForecastRow<'a> {
    pub fn new(forecast_report: &'a WeatherReport) -> Self {
        let datetime = datetime::get_datetime().unwrap();
        let format = time::format_description::parse("[hour]:00").expect("Invalid format.");
        let time = Box::new(datetime.format(&format).expect("Could not format time."));
        let time = Box::leak(time).as_str(); // wtf

        let forecast0 = Forecast::new("", &time, forecast_report.temp);
        let forecast1 = Forecast::new("", &time, forecast_report.temp);
        let forecast2 = Forecast::new("", &time, forecast_report.temp);
        let forecast3 = Forecast::new("", &time, forecast_report.temp);
        let forecast4 = Forecast::new("", &time, forecast_report.temp);

        Self {
            forecast0,
            forecast1,
            forecast2,
            forecast3,
            forecast4,
        }
    }
}

impl<'a> View for ForecastRow<'a> {
    fn translate_impl(&mut self, by: Point) {
        self.forecast0.translate_mut(by);
        self.forecast1.translate_mut(by);
        self.forecast2.translate_mut(by);
        self.forecast3.translate_mut(by);
        self.forecast4.translate_mut(by);
    }

    fn bounds(&self) -> Rectangle {
        self.bounding_box()
    }
}

impl<'a> Dimensions for ForecastRow<'a> {
    fn bounding_box(&self) -> Rectangle {
        self.forecast0
            .bounding_box()
            .enveloping(&self.forecast4.bounding_box())
    }
}

impl<'a> Drawable for ForecastRow<'a> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D: DrawTarget<Color = BinaryColor>>(&self, target: &mut D) -> Result<(), D::Error> {
        self.forecast0.draw(target);
        self.forecast1.draw(target);
        self.forecast2.draw(target);
        self.forecast3.draw(target);
        self.forecast4.draw(target);
        Ok(())
    }
}
