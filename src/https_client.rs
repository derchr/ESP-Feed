use anyhow::{bail, Result};
use esp_idf_sys::{esp_tls, esp_tls_cfg};
use log::*;
use std::ops::Sub;
use url::Url;

pub fn https_request(url: &Url) -> Result<Vec<u8>> {
    let url_string = std::ffi::CString::new(url.as_str()).expect("Invalid CString.");

    // No verification for now. Make sure it's enabled in menuconfig.
    let tls_config = esp_tls_cfg {
        ..Default::default()
    };

    info!("Create new TLS connection.");
    let tls =
        unsafe { esp_idf_sys::esp_tls_conn_http_new(url_string.as_ptr(), &tls_config as *const _) };

    if tls == std::ptr::null_mut() {
        warn!("Connection failed!");
        bail!("Connection failed!");
    } else {
        info!("Connection established!");
    }

    send_request(url, tls)?;
    let response = read_response(tls)?;

    info!("Delete connection!");
    unsafe { esp_idf_sys::esp_tls_conn_delete(tls) };

    Ok(response)
}

fn send_request(url: &Url, tls: *mut esp_tls) -> Result<()> {
    let mut written_bytes = 0;

    let request = String::new()
        + "GET "
        + url.path()
        + " HTTP/1.1\r\nHost: "
        + url.domain().expect("No domain.")
        + "\r\nConnection: close\r\n\r\n";

    while written_bytes < request.len() {
        let ret = unsafe {
            esp_idf_sys::esp_tls_conn_write(
                tls,
                request.as_ptr().add(written_bytes) as _,
                request.len().sub(written_bytes) as u32,
            )
        };
        written_bytes = written_bytes + ret as usize;
    }

    info!("HTTPS request sent!");
    Ok(())
}

fn read_response(tls: *mut esp_tls) -> Result<Vec<u8>> {
    let mut response = Vec::new();
    let mut buf = [0 as u8; 512];

    loop {
        let ret =
            unsafe { esp_idf_sys::esp_tls_conn_read(tls, buf.as_mut_ptr() as _, (buf.len()) as _) };

        if ret == 0 {
            info!("Connection closed...");
            break;
        } else {
            response.extend_from_slice(&buf[..ret as _]);
        }
    }

    Ok(response)
}
