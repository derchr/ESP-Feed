use anyhow::{Context, Result};
use embedded_hal::{adc::OneShot, digital::v2::InputPin};
// use embedded_hal_alpha::adc::nb::OneShot;
use esp_feed::{
    command::Command,
    datetime, graphics,
    graphics::{
        display,
        pages::{ConfigPage, PageType},
    },
    interrupt,
    nvs::NvsController,
    server::{self, PersonalData, RssData, StockData, WifiData},
    state,
    storage::StorageHandle,
    wifi,
};
use esp_idf_hal::{adc::PoweredAdc, gpio::Pin, prelude::*};
use esp_idf_svc::{
    log::EspLogger, netif::EspNetifStack, nvs::EspDefaultNvs, sysloop::EspSysLoopStack,
};
use esp_idf_sys as _; // Always keep it imported
use log::*;
use std::sync::{
    mpsc::{self, RecvTimeoutError},
    Arc, Mutex,
};

// #[allow(dead_code)]
fn setup_logging() {
    EspLogger::initialize_default();

    // In release build the CONFIG_LOG_MAXIMUM_LEVEL should be set to Warn.
    // TODO: Still doesn't work.
    // EspLogger.set_target_level("esp_feed", LevelFilter::Info);
    EspLogger.set_target_level("*", LevelFilter::Error);

    // No longer working with ESP-IDF 4.3.1+
    // std::env::set_var("RUST_BACKTRACE", "1");
}

fn main() -> Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    setup_logging();

    // Refer to "ECO_and_Workarounds_for_Bugs_in_ESP32" section 3.11.
    // A proposed workaround is to call this function once.
    unsafe { esp_idf_sys::adc_power_acquire() };

    let _storage_handle = StorageHandle::new();

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    let vbat_pin = pins.gpio35.into_analog_atten_11db().unwrap();
    let vbat_adc = PoweredAdc::new(peripherals.adc1, Default::default()).unwrap();
    let mut vbat = (vbat_pin, vbat_adc);

    let button_pin = pins.gpio39.into_input().unwrap();
    let setup_mode = button_pin.is_low().unwrap();

    let netif_stack = Arc::new(EspNetifStack::new()?);
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    let default_nvs = Arc::new(EspDefaultNvs::new()?);

    let button1_state = interrupt::register_button_interrupt(button_pin.pin());

    let mut nvs_controller = NvsController::new(Arc::clone(&default_nvs))?;
    let wifi_config = nvs_controller.get_config::<WifiData>().ok().map(Into::into);
    let personal_config = nvs_controller.get_config::<PersonalData>().ok();
    let rss_config = nvs_controller.get_config::<RssData>().ok();
    let stock_config = nvs_controller
        .get_config::<StockData>()
        .unwrap_or(StockData {
            symbol: "IBM".into(),
        });

    let location = personal_config
        .map(|data| data.location)
        .unwrap_or_default();

    let start_page = {
        let page = nvs_controller.get("last_page").unwrap_or_default();

        if page == PageType::ConfigPage(ConfigPage) {
            Default::default()
        } else {
            page
        }
    };

    let (command_tx, command_rx) = mpsc::channel();
    let (update_page_tx, update_page_rx) = mpsc::channel();

    let state = Arc::new(Mutex::new(state::State::new(
        setup_mode,
        wifi_config.clone(),
        location,
        start_page,
        &stock_config.symbol,
    )));

    let spi3 = peripherals.spi3;
    let busy = pins.gpio4.into_input()?;
    let rst = pins.gpio16.into_output()?;
    let dc = pins.gpio17.into_output()?;
    let cs = pins.gpio5.into_output()?;
    let sclk = pins.gpio18;
    let mosi = pins.gpio23;

    let mut display = display::get_epd_display(busy, rst, dc, cs, sclk, mosi, spi3)?;
    {
        let state = Arc::clone(&state);

        std::thread::Builder::new()
            .name("Display".into())
            .stack_size(10240)
            .spawn(move || graphics::draw_pages(&mut display, state, update_page_rx))
            .context("Could not create display thread.")?;
    }

    if setup_mode {
        let wifi = wifi::create_accesspoint(
            Arc::clone(&netif_stack),
            Arc::clone(&sys_loop_stack),
            Arc::clone(&default_nvs),
        )?;

        // Dont ever disconnect from the wifi.
        std::mem::forget(wifi);
    };

    if !setup_mode {
        let (wifi_tx, wifi_rx) = mpsc::channel();

        std::thread::Builder::new()
            .name("Deep Sleep".into())
            .stack_size(10240)
            .spawn(move || -> Result<()> {
                loop {
                    let wifi = wifi::connect(
                        wifi_config.as_ref(),
                        Arc::clone(&netif_stack),
                        Arc::clone(&sys_loop_stack),
                        Arc::clone(&default_nvs),
                    )?;

                    // Notify main thread.
                    wifi_tx.send(()).unwrap();

                    std::thread::sleep(std::time::Duration::from_secs(90));

                    // Gracefully shut down wifi.
                    info!("Shutdown wifi.");
                    drop(wifi);

                    info!("Enter deep sleep now.");
                    unsafe {
                        esp_idf_sys::esp_deep_sleep(1800 * 1_000_000u64);
                    }

                    // NOTE: Currently the ESP crashes after deep sleep wakeup.
                    // BUT this is not a huge problem and can be exploited to
                    // restart the ESP which it should do anyways.

                    info!("Wakeup from deep sleep.");
                }
            })
            .context("Could not create deep sleep thread.")?;

        // Wait until wifi connection is established.
        wifi_rx.recv().unwrap();
    }

    {
        let command_tx = command_tx.clone();

        std::thread::Builder::new()
            .name("Server".into())
            .spawn(move || -> Result<()> {
                let _server = server::httpd(command_tx.clone())?;

                // Let the server run for 10 minutes, then shut it down.
                // Note: In normal mode this actually does nothing because
                // the deep sleep will shut it down anyways.
                std::thread::sleep(std::time::Duration::from_secs(600));

                Ok(())
            })?;
    }

    let _sntp = datetime::initialize_time()?;

    {
        let controller = &mut state.lock().unwrap().feed_controller;
        let rss_data = rss_config.unwrap_or(RssData {
            url: "https://www.tagesschau.de/newsticker.rdf".into(),
        });

        if let Ok(url) = url::Url::parse(&rss_data.url) {
            let urls = [url];
            controller.urls_mut().extend_from_slice(&urls);
        }
    }

    let fetching_thread = {
        let state = Arc::clone(&state);
        let update_page_tx = update_page_tx.clone();

        move || {
            fn fetch_data(state: &mut state::State) -> Result<()> {
                let feed_controller = &mut state.feed_controller;
                info!("Fetching feeds: {:?}", feed_controller.urls_mut());
                feed_controller
                    .refresh()
                    .context("Could not retrieve feeds.")?;

                let weather_controller = &mut state.weather_controller;
                info!("Fetching weather.");
                weather_controller
                    .refresh(&state.location)
                    .context("Could not retrieve weather data.")?;

                let stock_controller = &mut state.stock_controller;
                info!("Fetching stock info.");
                stock_controller
                    .refresh()
                    .context("Could not retrieve stock info.")?;
                Ok(())
            }

            loop {
                {
                    let state = &mut state.lock().unwrap();

                    if let Err(e) = fetch_data(state) {
                        log::warn!("{:?}", e.context("Could not retrieve new data."));
                    }

                    // Get battery voltage
                    if let Ok(val) = vbat.1.read(&mut vbat.0) {
                        state.battery = val * 2; // 1/2 Voltage divider
                    }
                }

                // Update page to show new data.
                update_page_tx.send(()).ok();

                std::thread::sleep(std::time::Duration::from_secs(1200));
            }
        }
    };

    std::thread::Builder::new()
        .name("Feed".into())
        .stack_size(30720)
        .spawn(fetching_thread)
        .context("Could not create feed fetching thread.")?;

    loop {
        match command_rx.recv_timeout(std::time::Duration::from_millis(750)) {
            Ok(Command::SwitchPage) => {
                let mut state = state.lock().unwrap();
                state.next_page();

                nvs_controller.store("last_page", &state.page)?;

                update_page_tx.send(())?;
            }
            Ok(Command::SavePersonalConfig(ref config)) => {
                info!("Save this personal config: {:?}", config);

                nvs_controller.store_config(config)?;

                let state = &mut state.lock().unwrap();
                state.location = config.location.clone();
            }
            Ok(Command::SaveWifiConfig(ref config)) => {
                info!("Save this wifi config: {:?}", config);

                nvs_controller.store_config(config)?;

                // let state = &mut state.lock().unwrap(); // TODO
            }
            Ok(Command::SaveRssConfig(ref config)) => {
                info!("Save this rss config: {:?}", config);

                nvs_controller.store_config(config)?;
            }
            Ok(Command::SaveStockConfig(ref config)) => {
                info!("Save this stock config: {:?}", config);

                nvs_controller.store_config(config)?;
            }
            Err(RecvTimeoutError::Timeout) => {
                // Check if a button was pressed in the meanwhile.
                let btn1_pressed = {
                    let mut pressed = button1_state.lock();
                    let old_pressed = *pressed;
                    if old_pressed > 0 {
                        *pressed = 0;
                    }
                    old_pressed
                };

                for _ in 0..btn1_pressed {
                    command_tx.send(Command::SwitchPage)?;
                }
            }
            _ => {}
        }
    }
}
