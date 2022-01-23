use anyhow::Result;
use epd_waveshare::{
    epd2in13_v2::{Display2in13, Epd2in13},
    prelude::WaveshareDisplay,
};
use esp_idf_hal::{
    gpio, i2c,
    prelude::*,
    spi::{config::Config, Master, Pins, SPI3}, delay::Ets,
};
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

pub type Display = Ssd1306<
    I2CInterface<i2c::Master<i2c::I2C0, gpio::Gpio26<gpio::Unknown>, gpio::Gpio27<gpio::Unknown>>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

pub fn get_display(
    scl: gpio::Gpio27<gpio::Unknown>,
    sda: gpio::Gpio26<gpio::Unknown>,
    i2c: i2c::I2C0,
) -> Display {
    let i2c_master_pins = esp_idf_hal::i2c::MasterPins { sda, scl };

    let config = esp_idf_hal::i2c::config::MasterConfig {
        baudrate: Hertz(1_000_000),
        ..Default::default()
    };

    let master = i2c::Master::new(i2c, i2c_master_pins, config).unwrap();
    let interface = I2CDisplayInterface::new(master);

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    display
}

type SpiMaster = Master<
    SPI3,
    gpio::Gpio18<gpio::Output>,
    gpio::Gpio23<gpio::Output>,
    gpio::Gpio1<gpio::Input>,
    gpio::Gpio2<gpio::Output>,
>;

type Edp2in13Display = Epd2in13<
    SpiMaster,
    gpio::Gpio5<gpio::Output>,
    gpio::Gpio4<gpio::Input>,
    gpio::Gpio17<gpio::Output>,
    gpio::Gpio16<gpio::Output>,
    Ets,
>;

pub struct EpdDisplay {
    pub epd2in13: Edp2in13Display,
    pub master: SpiMaster,
    pub display: Display2in13,
}

pub fn get_epd_display(
    busy: gpio::Gpio4<gpio::Input>,
    rst: gpio::Gpio16<gpio::Output>,
    dc: gpio::Gpio17<gpio::Output>,
    cs: gpio::Gpio5<gpio::Output>,
    sclk: gpio::Gpio18<gpio::Output>,
    mosi: gpio::Gpio23<gpio::Output>,
    spi3: SPI3,
) -> Result<EpdDisplay> {
    let spi_pins = Pins {
        sclk,
        sdo: mosi,
        sdi: Option::<gpio::Gpio1<gpio::Input>>::None,
        cs: Option::<gpio::Gpio2<gpio::Output>>::None,
    };

    let spi_config = Config {
        baudrate: esp_idf_hal::units::MegaHertz(4).into(),
        ..Default::default()
    };

    let mut master: SpiMaster =
        Master::<SPI3, _, _, _, _>::new(spi3, spi_pins, spi_config).unwrap();

    let mut delay = Ets;
    let epd2in13: Edp2in13Display =
        Epd2in13::new(&mut master, cs, busy, dc, rst, &mut delay).unwrap();
    let display = Display2in13::default();

    Ok(EpdDisplay {
        epd2in13,
        master,
        display,
    })
}
