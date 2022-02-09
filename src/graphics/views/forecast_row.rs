use crate::{
    datetime,
    graphics::views::forecast::Forecast,
    weather::WeatherController,
    weather::WeatherReport, // TODO Weathercontroller forecast ?
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

pub enum ForecastType {
    Daily,
    Hourly,
}

impl<'a> ForecastRow<'a> {
    pub fn new(controller: &'a WeatherController, forecast_type: ForecastType) -> Self {
        let format = time::format_description::parse("[hour]:00").expect("Invalid format.");

        let WeatherReport { dt, icon, temp, .. } = controller.hourly(1).unwrap_or_default();
        let datetime = datetime::get_datetime_from_unix(dt as _).unwrap();
        let time = Box::new(datetime.format(&format).expect("Could not format time."));
        let time = Box::leak(time).as_str(); // wtf TODO
        let widget0 = Forecast::new(icon, time, temp);

        let WeatherReport { dt, icon, temp, .. } = controller.hourly(2).unwrap_or_default();
        let datetime = datetime::get_datetime_from_unix(dt as _).unwrap();
        let time = Box::new(datetime.format(&format).expect("Could not format time."));
        let time = Box::leak(time).as_str(); // wtf TODO
        let widget1 = Forecast::new(icon, time, temp);

        let WeatherReport { dt, icon, temp, .. } = controller.hourly(3).unwrap_or_default();
        let datetime = datetime::get_datetime_from_unix(dt as _).unwrap();
        let time = Box::new(datetime.format(&format).expect("Could not format time."));
        let time = Box::leak(time).as_str(); // wtf TODO
        let widget2 = Forecast::new(icon, time, temp);

        let WeatherReport { dt, icon, temp, .. } = controller.hourly(4).unwrap_or_default();
        let datetime = datetime::get_datetime_from_unix(dt as _).unwrap();
        let time = Box::new(datetime.format(&format).expect("Could not format time."));
        let time = Box::leak(time).as_str(); // wtf TODO
        let widget3 = Forecast::new(icon, time, temp);

        let WeatherReport { dt, icon, temp, .. } = controller.hourly(5).unwrap_or_default();
        let datetime = datetime::get_datetime_from_unix(dt as _).unwrap();
        let time = Box::new(datetime.format(&format).expect("Could not format time."));
        let time = Box::leak(time).as_str(); // wtf TODO
        let widget4 = Forecast::new(icon, time, temp);

        let layout = LinearLayout::horizontal(
            Chain::new(widget0)
                .append(widget1)
                .append(widget2)
                .append(widget3)
                .append(widget4),
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
