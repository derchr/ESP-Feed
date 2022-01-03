use anyhow::{bail, Result};
use esp_idf_sys::{esp_tls, esp_tls_cfg};
use log::*;
use std::{io::Read, ops::Sub};
use url::Url;

pub struct HttpsClient {
    tls: *mut esp_tls,
}

impl HttpsClient {
    pub fn new(url: &Url) -> Result<HttpsClient> {
        let url_string = std::ffi::CString::new(url.as_str()).expect("Invalid CString.");

        // No verification for now. Make sure it's enabled in menuconfig.
        let tls_config = esp_tls_cfg {
            ..Default::default()
        };

        info!("Create new TLS connection.");
        let tls = unsafe {
            esp_idf_sys::esp_tls_conn_http_new(url_string.as_ptr(), &tls_config as *const _)
        };

        if tls == std::ptr::null_mut() {
            warn!("Connection failed!");
            bail!("Connection failed!");
        } else {
            info!("Connection established!");
        }

        send_request(tls, url)?;

        Ok(HttpsClient { tls })
    }
}

impl Read for HttpsClient {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let ret = unsafe {
            esp_idf_sys::esp_tls_conn_read(self.tls, buf.as_mut_ptr() as _, (buf.len()) as _)
        };

        Ok(ret as _)
    }
}

impl Drop for HttpsClient {
    fn drop(&mut self) {
        info!("Delete connection!");
        unsafe { esp_idf_sys::esp_tls_conn_delete(self.tls) };
    }
}

fn send_request(tls: *mut esp_tls, url: &Url) -> Result<()> {
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
