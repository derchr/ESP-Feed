use crate::{datetime, feed::Feed, graphics::views::forecast::Forecast};
use embedded_graphics::{
    mono_font::{iso_8859_1::*, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, Rectangle},
    text::Text,
};
use embedded_layout::{
    layout::linear::{FixedMargin, LinearLayout},
    prelude::*,
};
use embedded_layout_macros::ViewGroup;

#[derive(ViewGroup)]
pub struct StatusBar {
    line: Line,
}
