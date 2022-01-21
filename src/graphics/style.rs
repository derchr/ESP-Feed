use embedded_graphics::{
    mono_font::{ascii::*, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, StrokeAlignment},
};

pub fn thin_stroke() -> PrimitiveStyle<BinaryColor> {
    PrimitiveStyle::with_stroke(BinaryColor::On, 1)
}

pub fn thick_stroke() -> PrimitiveStyle<BinaryColor> {
    PrimitiveStyle::with_stroke(BinaryColor::On, 3)
}

pub fn border_stroke() -> PrimitiveStyle<BinaryColor> {
    PrimitiveStyleBuilder::new()
        .stroke_color(BinaryColor::On)
        .stroke_width(3)
        .stroke_alignment(StrokeAlignment::Inside)
        .build()
}

pub fn normal_text() -> MonoTextStyle<'static, BinaryColor> {
    MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .background_color(BinaryColor::Off)
        .build()
}
