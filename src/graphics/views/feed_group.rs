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
use embedded_text::{
    style::{HeightMode, TextBoxStyleBuilder},
    TextBox,
};

// #[derive(ViewGroup)]
pub struct FeedGroup<'a, C: PixelColor> {
    pub title: TextBox<'a, MonoTextStyle<'static, C>>,
    pub headline0: TextBox<'a, MonoTextStyle<'static, C>>,
    pub headline1: TextBox<'a, MonoTextStyle<'static, C>>,
    pub headline2: TextBox<'a, MonoTextStyle<'static, C>>,
    pub headline3: TextBox<'a, MonoTextStyle<'static, C>>,
}

impl<'a> FeedGroup<'a, BinaryColor> {
    pub fn new(feed: &'a Feed, target_bounds: Rectangle) -> Self {
        let title_style = MonoTextStyle::new(&FONT_6X13_BOLD /*FONT_8X13*/, BinaryColor::On);
        let headline_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

        let textbox_style = TextBoxStyleBuilder::new()
            .height_mode(HeightMode::FitToText)
            .alignment(embedded_text::alignment::HorizontalAlignment::Left)
            .build();

        let bounds = Rectangle::new(Point::zero(), Size::new(target_bounds.size.width, 0));

        Self {
            title: TextBox::with_textbox_style(&feed.title, bounds, title_style, textbox_style),
            headline0: TextBox::with_textbox_style(
                &feed.headlines.get(0).unwrap(),
                bounds,
                headline_style,
                textbox_style,
            ),
            headline1: TextBox::with_textbox_style(
                &feed.headlines.get(1).unwrap(),
                bounds,
                headline_style,
                textbox_style,
            ),
            headline2: TextBox::with_textbox_style(
                &feed.headlines.get(2).unwrap(),
                bounds,
                headline_style,
                textbox_style,
            ),
            headline3: TextBox::with_textbox_style(
                &feed.headlines.get(3).unwrap(),
                bounds,
                headline_style,
                textbox_style,
            ),
        }
    }
}
