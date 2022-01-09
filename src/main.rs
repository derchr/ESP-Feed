#[allow(unused_imports)]
use esp_idf_sys; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use anyhow::Result;
use log::*;

use embedded_svc::storage::Storage;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{netif::*, nvs::*, sysloop::*};
use serde::{Deserialize, Serialize};

use std::{collections::BTreeMap, sync::Arc};

mod datetime;
mod display;
mod feed;
mod graphics;
mod weather;
mod wifi;

use crate::{datetime::*, display::*, feed::*, graphics::*, wifi::*};

#[derive(Serialize, Deserialize, Debug)]
enum Matter {
    Earth(u64),
    Water(i32),
    Fire(bool),
    Air(f64),
}
#[derive(Serialize, Deserialize, Debug)]
struct MyObject {
    data: String,
    len: usize,
    flags: Box<u32>,
    valid: Option<bool>,
    world: Matter,
}

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
    let _wifi = wifi(netif_stack, sys_loop_stack, default_nvs.clone())?;
    use embedded_svc::wifi::Wifi;
    info!("{:?}", _wifi.get_capabilities());
    std::mem::forget(_wifi);

    info!("Open storage!");
    let mut nvs_storage =
        esp_idf_svc::nvs_storage::EspNvsStorage::new_default(default_nvs, "esp_feed", true)
            .expect("Failed to open NVS storage.");

    if let Ok(vec) = nvs_storage.get_raw("my_key") {
        println!("{:?}", vec);
    }

    let my_obj = MyObject {
        data: "Hallo Welt!".into(),
        len: 12,
        flags: Box::new(0x100),
        valid: Some(true),
        world: Matter::Air(5.123213),
    };

    nvs_storage.put("my_key", &my_obj).unwrap_or_else(|e| {
        warn!("Could not store key: {:?}", e);
        false
    });

    if let Ok(vec) = nvs_storage.get_raw("my_key") {
        println!("{:?}", vec);
        println!("{}", std::str::from_utf8(&(vec.unwrap())).unwrap());
    }

    if let Ok(e) = nvs_storage.get("my_key") {
        let s: MyObject = e.unwrap();
        println!("Value: {:?}", s);
    }

    let _sntp = initialize_time()?;
    std::mem::forget(_sntp);
    info!("Current local time: {}", get_datetime()?);

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    let mut display = get_display(pins.gpio27, pins.gpio26, peripherals.i2c0);

    std::thread::Builder::new()
        .stack_size(40960)
        .spawn(move || draw_page(&mut display))
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

    fn httpd() -> Result<esp_idf_svc::httpd::Server> {
        use anyhow::bail;
        use embedded_svc::httpd::registry::*;
        use embedded_svc::httpd::*;

        let form_handler = embedded_svc::httpd::Handler::new(
            "/form",
            embedded_svc::httpd::Method::Get,
            |req| -> Result<embedded_svc::httpd::Response> {
                Ok(embedded_svc::httpd::Response {
                    status: 200,
                    status_message: None,
                    headers: BTreeMap::new(),
                    body: embedded_svc::httpd::Body::Empty,
                    new_session_state: None,
                })
            },
        );

        let server = esp_idf_svc::httpd::ServerRegistry::new()
            .at("/")
            .get(|_| Ok(include_str!("settings.html").into()))?
            .at("/simple")
            .get(|_| Ok(include_str!("simple.html").into()))?
            .at("/foo")
            .get(|_| bail!("Boo, something happened!"))?
            .at("/bar")
            .get(|_| {
                Response::new(403)
                    .status_message("No permissions")
                    .body("You have no permissions to access this page".into())
                    .into()
            })?
            .at("/panic")
            .get(|_| panic!("User requested a panic!"))?
            .handler(form_handler)?
            .at("/favicon.ico")
            .get(|_| {
                Response::new(200)
                    .content_type("image/x-icon")
                    .body(embedded_svc::httpd::Body::Bytes(
                        include_bytes!("favicon.ico").to_vec(),
                    ))
                    .into()
            })?
            .at("/")
            .post(|mut req| {
                let body = req.as_string().unwrap_or("".into());

                #[derive(Deserialize, Debug)]
                struct Form {
                    name: String,
                    ssid: String,
                    auth: String,
                }

                let my_form: Form = serde_json::from_str(&body).unwrap();

                let resp = format!(
                    "Header: {}<br>Body: {}<br>",
                    req.header("User-Agent").unwrap_or_default(),
                    body
                );

                info!("Send resp: {}, Form: {:?}", resp, my_form);

                Response::new(200).body(resp.into()).into()
            })?;

        server.start(&Default::default())
    }

    let _server = httpd().unwrap();
    std::mem::forget(_server);

    Ok(())
}
