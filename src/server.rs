use anyhow::{Context, Result};
use embedded_svc::httpd::{registry::Registry, Handler, Method, Response};
use esp_idf_svc::httpd::Server;
use log::*;
use serde::Deserialize;
use std::fs::File;

fn favicon_handler() -> Handler {
    Handler::new("/favicon.ico", Method::Get, |_| {
        let favicon_path = "/mnt/favicon.ico";
        let favicon = File::open(favicon_path)
            .with_context(|| format!("Could not find favicon: {}", favicon_path))?;

        Ok(Response::new(200)
            .content_type("image/x-icon")
            .body(favicon.into()))
    })
}

fn weather() -> Handler {
    Handler::new("/weather", Method::Get, |_| {
        let path = "/mnt/01d.png";
        let icon = File::open(path)
            .with_context(|| format!("Could not find weather icon: {}", path))?;

        Ok(Response::new(200)
            .content_type("image/png")
            .body(icon.into()))
    })
}

pub fn httpd() -> Result<Server> {
    let server = esp_idf_svc::httpd::ServerRegistry::new()
        .at("/")
        .get(|_| {
            Ok(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/resources/settings.html"
            ))
            .into())
        })?
        .at("/simple")
        .get(|_| {
            Ok(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/resources/simple.html"
            ))
            .into())
        })?
        .handler(favicon_handler())?
        .handler(weather())?
        .at("/")
        .post(move |mut req| {
            let body = req.as_string().unwrap_or("".into());

            #[derive(Deserialize, Debug)]
            struct Form {
                name: String,
                ssid: String,
                pass: String,
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
