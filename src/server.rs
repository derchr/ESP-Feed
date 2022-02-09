use crate::{command::Command, storage::BASE_DIR};
use anyhow::{Context, Result};
use embedded_svc::httpd::{registry::Registry, Handler, Method, Response};
use esp_idf_svc::httpd::Server;
use serde::Deserialize;
use std::{fs::File, sync::mpsc::Sender};

#[derive(Deserialize, Debug, Clone)]
pub struct FormData {
    pub name: String,
    pub ssid: String,
    pub pass: String,
}

fn favicon_handler() -> Handler {
    Handler::new("/favicon.ico", Method::Get, |_| {
        let favicon_path = &format!("{}/favicon.ico", BASE_DIR);
        let favicon = File::open(favicon_path)
            .with_context(|| format!("Could not find favicon: {}", favicon_path))?;

        Ok(Response::new(200)
            .content_type("image/x-icon")
            .body(favicon.into()))
    })
}

fn weather() -> Handler {
    Handler::new("/weather", Method::Get, |_| {
        let path = &format!("{}/01d.png", BASE_DIR);
        let icon =
            File::open(path).with_context(|| format!("Could not find weather icon: {}", path))?;

        Ok(Response::new(200)
            .content_type("image/png")
            .body(icon.into()))
    })
}

pub fn httpd(command_tx: Sender<Command>) -> Result<Server> {
    let server = esp_idf_svc::httpd::ServerRegistry::new()
        .at("/")
        .get(|_| {
            let settings_path = &format!("{}/settings.htm", BASE_DIR);
            let settings = File::open(settings_path)
                .with_context(|| format!("Could not find html: {}", settings_path))?;

            Ok(settings.into())
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
            let body = req.as_string()?;

            let form: FormData = serde_json::from_str(&body).unwrap();
            command_tx.send(Command::SaveConfig(form))?;

            let resp = "Gespeichert!";
            Response::new(200).body(resp.into()).into()
        })?;

    server.start(&Default::default())
}
