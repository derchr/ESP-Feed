use anyhow::{Context, Result};
use embedded_hal::digital::blocking::InputPin;
use esp_feed::{datetime, display, feed, graphics, nvs::NvsController, server, state, wifi};
use esp_idf_hal::prelude::*;
use esp_idf_svc::{netif::*, nvs::*, sysloop::*};
use esp_idf_sys; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use std::sync::{Arc, Mutex};

fn setup_logging() {
    esp_idf_svc::log::EspLogger::initialize_default();
    esp_idf_svc::log::EspLogger.set_target_level("*", log::LevelFilter::Debug);

    std::env::set_var("RUST_BACKTRACE", "1");
}

fn main() -> Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    setup_logging();

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    let setup_button = pins.gpio35.into_input().unwrap();
    let setup_mode = setup_button.is_high().unwrap();

    let netif_stack = Arc::new(EspNetifStack::new()?);
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    let default_nvs = Arc::new(EspDefaultNvs::new()?);
    
    let mut nvs_controller = NvsController::new(Arc::clone(&default_nvs))?;

    nvs_controller.store_wifi_config(&state::WifiConfig {
        ssid: wifi::SSID.into(),
        auth: wifi::PASS.into(),
    })?;
    let wifi_config = nvs_controller.get_wifi_config().ok();
    
    let _wifi = if setup_mode {
        wifi::create_accesspoint(netif_stack, sys_loop_stack, default_nvs.clone())?
    } else {
        match wifi::connect(
            Arc::clone(&netif_stack),
            Arc::clone(&sys_loop_stack),
            Arc::clone(&default_nvs),
        ) {
            Ok(wifi) => wifi,
            Err(_) => wifi::create_accesspoint(netif_stack, sys_loop_stack, default_nvs.clone())?,
        }
    };
    std::mem::forget(_wifi);

    let mut display = display::get_display(pins.gpio27, pins.gpio26, peripherals.i2c0);

    let state = Arc::new(Mutex::new(state::State::new(setup_mode, wifi_config)));

    {
        let state = Arc::clone(&state);

        std::thread::Builder::new()
            .stack_size(10240)
            .spawn(move || graphics::draw_pages(&mut display, state))
            .map_err(|e| anyhow::Error::from(e))
            .context("Could not create display thread.")?;
    }

    let _server = server::httpd()?;
    std::mem::forget(_server);

    if setup_mode {
        return Ok(());
    }

    let _sntp = datetime::initialize_time()?;
    std::mem::forget(_sntp);

    let mut controller = feed::FeedController::new();
    let urls = [
        url::Url::parse("https://www.tagesschau.de/newsticker.rdf").expect("Invalid Url"),
        url::Url::parse("https://www.uni-kl.de/pr-marketing/studium/rss.xml").expect("Invalid Url"),
    ];
    controller.urls().extend_from_slice(&urls);
    controller.refresh().context("Could not retrieve feeds.")?;

    Ok(())
}
