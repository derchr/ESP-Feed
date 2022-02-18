use anyhow::Result;
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::BinaryColor, prelude::*};
use epd_waveshare::{
    epd2in13_v2::{Display2in13, Epd2in13},
    prelude::*,
};
use esp_idf_hal::{
    delay, gpio,
    spi::{config::Config, Master, Pins, SPI3},
};

type SpiMaster = Master<
    SPI3,
    gpio::Gpio18<gpio::Unknown>,
    gpio::Gpio23<gpio::Unknown>,
    gpio::Gpio1<gpio::Input>,
    gpio::Gpio5<gpio::Output>,
>;

type Edp2in13Display = Epd2in13<
    SpiMaster,
    gpio::Gpio5<gpio::Output>,
    gpio::Gpio4<gpio::Input>,
    gpio::Gpio17<gpio::Output>,
    gpio::Gpio16<gpio::Output>,
    delay::Ets,
>;

pub struct EpdDisplay {
    pub epd2in13: Edp2in13Display,
    pub master: SpiMaster,
    pub display: Box<RotatedDisplay2in13>,
    pub delay: delay::Ets,
}

pub struct RotatedDisplay2in13(Display2in13);

impl RotatedDisplay2in13 {
    pub fn new() -> Self {
        let mut display = Display2in13::default();
        display.set_rotation(DisplayRotation::Rotate90);
        Self(display)
    }
}

impl Default for RotatedDisplay2in13 {
    fn default() -> Self {
        Self::new()
    }
}

impl DrawTarget for RotatedDisplay2in13 {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        self.0.draw_iter(pixels)
    }
}

impl OriginDimensions for RotatedDisplay2in13 {
    fn size(&self) -> embedded_graphics::prelude::Size {
        Size::new(self.0.size().height, self.0.size().width)
    }
}

impl Display for RotatedDisplay2in13 {
    fn buffer(&self) -> &[u8] {
        self.0.buffer()
    }

    fn get_mut_buffer(&mut self) -> &mut [u8] {
        self.0.get_mut_buffer()
    }

    fn set_rotation(&mut self, rotation: DisplayRotation) {
        self.0.set_rotation(rotation)
    }

    fn rotation(&self) -> DisplayRotation {
        self.0.rotation()
    }
}

pub fn get_epd_display(
    busy: gpio::Gpio4<gpio::Input>,
    rst: gpio::Gpio16<gpio::Output>,
    dc: gpio::Gpio17<gpio::Output>,
    cs: gpio::Gpio5<gpio::Output>,
    sclk: gpio::Gpio18<gpio::Unknown>,
    mosi: gpio::Gpio23<gpio::Unknown>,
    spi3: SPI3,
) -> Result<EpdDisplay> {
    let spi_pins = Pins {
        sclk,
        sdo: mosi,
        sdi: Option::<gpio::Gpio1<gpio::Input>>::None,
        cs: Option::<gpio::Gpio5<gpio::Output>>::None,
    };

    let spi_config = Config {
        // baudrate: esp_idf_hal::units::MegaHertz(12).into(),
        baudrate: esp_idf_hal::units::MegaHertz(2).into(),
        ..Default::default()
    };

    let mut master: SpiMaster =
        Master::<SPI3, _, _, _, _>::new(spi3, spi_pins, spi_config).unwrap();

    let mut delay = delay::Ets;

    let epd2in13: Edp2in13Display =
        Epd2in13::new(&mut master, cs, busy, dc, rst, &mut delay).unwrap();

    let display: Box<RotatedDisplay2in13> = Default::default();
    //     epd2in13.clear_frame(&mut master, &mut delay)?;

    //     epd2in13
    //         .set_refresh(&mut master, &mut delay, RefreshLut::Quick)
    //         .unwrap();
    //     epd2in13.clear_frame(&mut master, &mut delay).unwrap();
    // use embedded_hal::blocking::delay::DelayMs;
    //     // a moving `Hello World!`
    //     let limit = 10;
    //     for i in 0..limit {
    //         draw_text(&mut display, "  Hello World! ", 5 + i * 12, 50);

    //         epd2in13
    //             .update_and_display_frame(&mut master, display.buffer(), &mut delay)
    //             .expect("display frame new graphics");

    //         delay.delay_ms(5u8);
    //     }

    Ok(EpdDisplay {
        epd2in13,
        master,
        display,
        delay,
    })
}

#[allow(unused)]
fn draw_text(display: &mut RotatedDisplay2in13, text: &str, x: i32, y: i32) {
    use embedded_graphics::mono_font::MonoTextStyleBuilder;
    use embedded_graphics::prelude::*;
    use embedded_graphics::text::Baseline;
    use embedded_graphics::text::Text;
    use embedded_graphics::text::TextStyleBuilder;
    use epd_waveshare::{
        color::*,
        epd2in13_v2::{Display2in13, Epd2in13},
        graphics::DisplayRotation,
        prelude::*,
    };
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
        .text_color(White)
        .background_color(Black)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style(text, Point::new(x, y), style, text_style).draw(display);
}
