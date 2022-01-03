#[allow(unused_imports)]
use esp_idf_sys; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use anyhow::Result;
use log::*;

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{netif::*, nvs::*, sysloop::*};

use std::sync::Arc;

mod datetime;
mod display;
mod feed;
mod graphics;
mod https_client;
mod wifi;

use crate::{datetime::*, display::*, feed::*, graphics::*, wifi::*};

fn main() -> Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();
    esp_idf_svc::log::EspLogger
        .set_target_level("*" /* nur wifi stuff ? */, log::LevelFilter::Trace);

    std::env::set_var("RUST_BACKTRACE", "1");

    let netif_stack = Arc::new(EspNetifStack::new()?);
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    let default_nvs = Arc::new(EspDefaultNvs::new()?);
    let _wifi = wifi(netif_stack, sys_loop_stack, default_nvs)?; // Do not drop until enf of program.

    let _sntp = initialize_time()?;
    info!("Current local time: {}", get_datetime()?);

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;
    let i2c0 = peripherals.i2c0;

    let scl = pins.gpio27.into_output().unwrap();
    let sda = pins.gpio26.into_output().unwrap();

    let mut display = get_display(scl, sda, i2c0);

    std::thread::Builder::new()
        .stack_size(40960)
        .spawn(move || draw_display(&mut display))
        .expect("Could not create display thread.");

    let url = url::Url::parse("https://www.tagesschau.de/newsticker.rdf").expect("Invalid Url");
    if let Ok(feed) = rss_feed(&url) {
        info!("New feed: {}", feed.title);
        for line in &feed.headlines {
            info!("{}", line);
        }
    }

    let url =
        url::Url::parse("https://www.uni-kl.de/pr-marketing/studium/rss.xml").expect("Invalid Url");
    if let Ok(feed) = rss_feed(&url) {
        info!("New feed: {}", feed.title);
        for line in &feed.headlines {
            info!("{}", line);
        }
    }

    Ok(())
}
