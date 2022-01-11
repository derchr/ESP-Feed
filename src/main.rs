#[allow(unused_imports)]
use esp_idf_sys; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use anyhow::{Context, Result};
use log::*;

use embedded_hal::digital::blocking::InputPin;
use esp_idf_hal::prelude::*;
use esp_idf_svc::{netif::*, nvs::*, sysloop::*};

use std::sync::mpsc::channel;
use std::sync::Arc;

use esp_feed::{datetime::*, display::*, feed::*, graphics::*, server::*, wifi::*};

fn main() -> Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();
    esp_idf_svc::log::EspLogger
        .set_target_level("*" /* nur wifi stuff ? */, log::LevelFilter::Trace);

    std::env::set_var("RUST_BACKTRACE", "1");

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    let ap_pin = pins.gpio35.into_input().unwrap();
    let ap_mode = ap_pin.is_high().unwrap();

    let netif_stack = Arc::new(EspNetifStack::new()?);
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    let default_nvs = Arc::new(EspDefaultNvs::new()?);
    let _wifi = wifi(netif_stack, sys_loop_stack, default_nvs.clone(), ap_mode)?;
    std::mem::forget(_wifi);

    let mut display = get_display(pins.gpio27, pins.gpio26, peripherals.i2c0);

    let page: Box<dyn Page<Display>> = if ap_mode {
        Box::new(ConfigPage)
    } else {
        Box::new(ExamplePage)
    };

    std::thread::Builder::new()
        .stack_size(40960)
        .spawn(move || draw_page(&mut display, page))
        .map_err(|e| anyhow::Error::from(e))
        .context("Could not create display thread.")?;

    let _server = httpd().unwrap();
    std::mem::forget(_server);

    if ap_mode {
        return Ok(());
    }

    let _sntp = initialize_time()?;
    std::mem::forget(_sntp);

    let mut controller = FeedController::new();
    let urls = [
        url::Url::parse("https://www.tagesschau.de/newsticker.rdf").expect("Invalid Url"),
        url::Url::parse("https://www.uni-kl.de/pr-marketing/studium/rss.xml").expect("Invalid Url"),
        url::Url::parse("https://blog.rust-embedded.org/rss.xml").expect("Invalid Url"),
    ];
    controller.set_urls(&urls);
    controller
        .retrieve_feeds()
        .context("Could not retrieve feeds.")?;

    Ok(())
}
