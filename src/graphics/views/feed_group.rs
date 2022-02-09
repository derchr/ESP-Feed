use crate::{datetime, feed::Feed, graphics::views::forecast::Forecast};
use embedded_graphics::{
    mono_font::{iso_8859_1::*, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
    text::Text,
};
use embedded_layout::{
    layout::linear::{FixedMargin, LinearLayout},
    prelude::*,
};
use embedded_layout_macros::ViewGroup;

#[derive(ViewGroup)]
pub struct FeedGroup<'a, C: PixelColor> {
    title: Text<'a, MonoTextStyle<'static, C>>,
    headline0: Text<'a, MonoTextStyle<'static, C>>,
    headline1: Text<'a, MonoTextStyle<'static, C>>,
    headline2: Text<'a, MonoTextStyle<'static, C>>,
    // headline3: Text<'a, MonoTextStyle<'static, C>>,
    // headline4: Text<'a, MonoTextStyle<'static, C>>,
}

impl<'a> FeedGroup<'a, BinaryColor> {
    pub fn new(feed: &'a Feed) -> Self {
        let title_style = MonoTextStyle::new(&FONT_6X13_BOLD /*FONT_8X13*/, BinaryColor::On);
        let headline_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

        Self {
            title: Text::new(&feed.title, Point::zero(), title_style),
            headline0: Text::new(
                &feed.headlines.get(0).unwrap(),
                Point::zero(),
                headline_style,
            ),
            headline1: Text::new(
                &feed.headlines.get(1).unwrap(),
                Point::zero(),
                headline_style,
            ),
            headline2: Text::new(
                &feed.headlines.get(2).unwrap(),
                Point::zero(),
                headline_style,
            ),
            // headline3: Text::new(&feed.headlines.get(3).unwrap(), Point::zero(), headline_style),
            // headline4: Text::new(&feed.headlines.get(4).unwrap(), Point::zero(), headline_style),
        }
    }
}
