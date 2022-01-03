use esp_idf_hal::{gpio, i2c, prelude::*};
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

pub fn get_display(
    scl: gpio::Gpio27<gpio::Output>,
    sda: gpio::Gpio26<gpio::Output>,
    i2c: i2c::I2C0,
) -> Ssd1306<impl WriteOnlyDataCommand, impl DisplaySize, BufferedGraphicsMode<impl DisplaySize>> {
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