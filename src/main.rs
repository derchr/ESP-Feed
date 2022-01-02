use anyhow::Result;
use log::*;

#[allow(unused_imports)]
use esp_idf_sys; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_hal::{peripherals::Peripherals, prelude::*};
use esp_idf_svc::{netif::*, nvs::*, sysloop::*};
use smol::{self, prelude::*};

use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use std::{
    net::{TcpListener, TcpStream},
    ops::Sub,
    sync::Arc
};

mod datetime;
mod graphics;
mod wifi;

use crate::{datetime::*, graphics::*, wifi::*};

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

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;
    let i2c0 = peripherals.i2c0;

    let scl = pins.gpio27.into_output().unwrap();
    let sda = pins.gpio26.into_output().unwrap();

    let i2c_master_pins = esp_idf_hal::i2c::MasterPins { sda, scl };

    let config = esp_idf_hal::i2c::config::MasterConfig {
        baudrate: Hertz(1_000_000),
        ..Default::default()
    };

    let master = esp_idf_hal::i2c::Master::new(i2c0, i2c_master_pins, config).unwrap();

    let interface = I2CDisplayInterface::new(master);

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    std::thread::Builder::new()
        .stack_size(40960)
        .spawn(move || draw_display(&mut display))
        .unwrap();

    let _sntp = initialize_time()?;

    info!("Current local time: {}", get_datetime()?);

    // if let Ok(local) = time::OffsetDateTime::now_local() {
    //     info!("Local time: {}", local);
    // }

    // requesot()?;

    rss_feed()?;

    Ok(())
}

fn rss_feed() -> Result<()> {
    let url = std::ffi::CString::new("https://www.tagesschau.de/").expect("Invalid URL CString.");

    // No verification for now. Make sure it's enabled in menuconfig.
    let tls_config = esp_idf_sys::esp_tls_cfg {
        ..Default::default()
    };
    let mut response = Vec::new();

    unsafe {
        info!("Create new tls connection");
        let tls = esp_idf_sys::esp_tls_conn_http_new(url.as_ptr(), &tls_config as *const _);

        if tls == std::ptr::null_mut() {
            info!("connection failed!");
            return Ok(());
        } else {
            info!("connection established!");
        }

        let mut written_bytes = 0;
        let request =
            b"GET /newsticker.rdf HTTP/1.1\r\nHost: www.tagesschau.de\r\nConnection: close\r\n\r\n";

        while written_bytes < request.len() {
            let ret = esp_idf_sys::esp_tls_conn_write(
                tls,
                request.as_ptr().add(written_bytes) as _,
                request.len().sub(written_bytes) as u32,
            );
            written_bytes = written_bytes + ret as usize;
        }

        info!("HTTPS request sent!");

        {
            let mut buf = [0 as u8; 512];

            loop {
                let ret =
                    esp_idf_sys::esp_tls_conn_read(tls, buf.as_mut_ptr() as _, (buf.len()) as _);

                if ret == 0 {
                    info!("Connection closed...");
                    break;
                } else {
                    response.extend_from_slice(&buf[..ret as _]);
                }
            }
        }

        info!("Delete connection!");
        esp_idf_sys::esp_tls_conn_delete(tls);
    }

    let index = response.iter().position(|x| *x == b'<').unwrap();

    info!("Print out all titles!");

    let parser = xml::reader::EventReader::new(&response[index..]);
    let mut title_follows = false;
    for e in parser {
        match e {
            Ok(xml::reader::XmlEvent::StartElement { name, .. }) => {
                if name.local_name == "title" {
                    title_follows = true;
                }
            }
            Ok(xml::reader::XmlEvent::EndElement { name }) => {
                if name.local_name == "title" {
                    title_follows = false;
                }
            }
            Ok(xml::reader::XmlEvent::Characters(content)) => {
                if title_follows {
                    println!("{}", content);
                }
            }
            Err(e) => {
                println!("Parse error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

#[allow(unused)]
fn request() -> Result<()> {
    async fn test_tcp_bind() -> smol::io::Result<()> {
        /// Echoes messages from the client back to it.
        async fn echo(stream: smol::Async<TcpStream>) -> smol::io::Result<()> {
            // smol::io::copy(&stream, &mut &stream).await?;

            loop {
                let mut buf = [0; 512];
                match (&stream).read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => (&stream).write_all(&buf[..n]).await?,
                    Err(_) => (),
                }
            }

            Ok(())
        }

        // Create a listener.
        let listener = smol::Async::<TcpListener>::bind(([0, 0, 0, 0], 8081))?;

        // Accept clients in a loop.
        loop {
            let (stream, peer_addr) = listener.accept().await?;
            info!("Accepted client: {}", peer_addr);

            // Spawn a task that echoes messages from the client back to it.
            smol::spawn(echo(stream)).detach();
        }
    }

    info!("About to bind a simple echo service to port 8081 using async (smol-rs)!");

    esp_idf_sys::esp!(unsafe {
        esp_idf_sys::esp_vfs_eventfd_register(&esp_idf_sys::esp_vfs_eventfd_config_t {
            max_fds: 5,
            ..Default::default()
        })
    })?;

    smol::block_on(test_tcp_bind()).unwrap();

    Ok(())
}
