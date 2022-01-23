use esp_feed::{
    datetime, feed, graphics, graphics::display, nvs::NvsController, server, state,
    storage::StorageHandle, weather, wifi,
};

use anyhow::{Context, Result};
// use embedded_hal::digital::blocking::InputPin;
use embedded_hal::digital::v2::InputPin;
use esp_idf_hal::prelude::*;
use esp_idf_svc::{netif::*, nvs::*, sysloop::*};
use esp_idf_sys; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

fn setup_logging() {
    esp_idf_svc::log::EspLogger::initialize_default();
    esp_idf_svc::log::EspLogger.set_target_level("*", log::LevelFilter::Debug);

    // std::env::set_var("RUST_BACKTRACE", "1");
}

fn main() -> Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    setup_logging();

    let _storage_handle = StorageHandle::new();
    std::mem::forget(_storage_handle);

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    let interrupt_pin = pins.gpio13.into_input().unwrap();
    let setup_button = pins.gpio35.into_input().unwrap();
    let setup_mode = setup_button.is_high().unwrap();

    let netif_stack = Arc::new(EspNetifStack::new()?);
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    let default_nvs = Arc::new(EspDefaultNvs::new()?);

    let btn1_state = Arc::new(esp_idf_hal::interrupt::Mutex::new(false));
    let btn1_state_cloned = Arc::clone(&btn1_state);

    let button_handle = {
        let mut last_tick = 0;
        let mut last_pressed = false;

        move || {
            let mut pressed = btn1_state_cloned.lock();

            // Assume tick rate is 100 Hz. Wait for at least 5 ticks (50ms).
            // TODO: Get the real tick rate from the config variable.
            let current_tick = unsafe { esp_idf_sys::xTaskGetTickCountFromISR() };
            if current_tick - last_tick < 5 {
                return;
            } else {
                last_tick = current_tick;
            }

            if last_pressed {
                last_pressed = false;
                return;
            } else {
                last_pressed = true;
            }

            *pressed = true;
        }
    };

    unsafe {
        esp_idf_sys::gpio_set_intr_type(13, esp_idf_sys::GPIO_INT_TYPE_GPIO_PIN_INTR_ANYEDGE);
        esp_idf_sys::gpio_install_isr_service(0);

        fn add_isr_handler<F>(f: F)
        where
            F: FnMut() -> (),
            F: 'static,
        {
            extern "C" fn isr_handler(arg: *mut std::ffi::c_void) {
                let closure: &mut Box<dyn FnMut() -> ()> = unsafe { std::mem::transmute(arg) };
                closure();
            }

            let cb: Box<Box<dyn FnMut() -> ()>> = Box::new(Box::new(f));
            unsafe {
                // Note: This leaks the closure, but it's fine as it
                // has live until the end of the program anyways.
                esp_idf_sys::gpio_isr_handler_add(
                    13,
                    Some(isr_handler),
                    Box::into_raw(cb) as *mut _,
                );
            }
        }

        add_isr_handler(button_handle);
    }

    let mut nvs_controller = NvsController::new(Arc::clone(&default_nvs))?;
    nvs_controller.store_wifi_config(&state::WifiConfig {
        ssid: wifi::SSID.into(),
        pass: wifi::PASS.into(),
    })?;
    let wifi_config = nvs_controller.get_wifi_config().ok();

    let mut display = display::get_display(pins.gpio27, pins.gpio26, peripherals.i2c0);

    let state = Arc::new(Mutex::new(state::State::new(
        setup_mode,
        wifi_config.clone(),
    )));

    {
        let state = Arc::clone(&state);

        std::thread::Builder::new()
            .stack_size(10240)
            .spawn(move || graphics::draw_pages(&mut display, state))
            .map_err(|e| anyhow::Error::from(e))
            .context("Could not create display thread.")?;
    }

    let _wifi = if setup_mode {
        wifi::create_accesspoint(netif_stack, sys_loop_stack, default_nvs.clone())?
    } else {
        match wifi::connect(
            wifi_config.clone(),
            Arc::clone(&netif_stack),
            Arc::clone(&sys_loop_stack),
            Arc::clone(&default_nvs),
        ) {
            Ok(wifi) => wifi,
            Err(_) => wifi::create_accesspoint(netif_stack, sys_loop_stack, default_nvs.clone())?,
        }
    };
    std::mem::forget(_wifi);

    let _server = server::httpd()?;
    std::mem::forget(_server);

    if setup_mode {
        return Ok(());
    }

    let _sntp = datetime::initialize_time()?;
    std::mem::forget(_sntp);

    {
        let mut controller = &mut state.lock().unwrap().feed_controller;
        let urls = [
            url::Url::parse("https://www.tagesschau.de/newsticker.rdf").expect("Invalid Url"),
            url::Url::parse("https://www.uni-kl.de/pr-marketing/studium/rss.xml")
                .expect("Invalid Url"),
        ];
        controller.urls().extend_from_slice(&urls);
        controller.refresh().context("Could not retrieve feeds.")?;
    }

    loop {
        {
            let val = {
                let mut val = btn1_state.lock();
                let old_val = *val;
                if old_val {
                    *val = false;
                }
                old_val
            };

            if val {
                println!("Button pressed!");
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    Ok(())
}
