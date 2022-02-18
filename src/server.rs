//! Webserver that provides a web interface to configure the application.

use crate::{command::Command, storage::BASE_DIR, wifi::WifiConfig};
use anyhow::{Context, Result};
use embedded_svc::httpd::{registry::Registry, Handler, Method, Response};
use esp_idf_svc::httpd::Server;
use serde::{Deserialize, Serialize};
use std::{fs::File, sync::mpsc::Sender};

pub trait ConfigData<'de>: Deserialize<'de> + Serialize + std::fmt::Debug + Into<Command> {
    fn key() -> &'static str;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PersonalData {
    pub name: String,
    pub location: String,
}

impl<'de> ConfigData<'de> for PersonalData {
    fn key() -> &'static str {
        "personal"
    }
}

impl From<PersonalData> for Command {
    fn from(config: PersonalData) -> Self {
        Command::SavePersonalConfig(config)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WifiData {
    pub ssid: String,
    pub pass: String,
}

impl<'de> ConfigData<'de> for WifiData {
    fn key() -> &'static str {
        "wifi"
    }
}

impl From<WifiData> for WifiConfig {
    fn from(data: WifiData) -> Self {
        Self {
            ssid: data.ssid,
            pass: data.pass,
        }
    }
}

impl From<WifiData> for Command {
    fn from(config: WifiData) -> Self {
        Command::SaveWifiConfig(config)
    }
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

fn settings_handler<T>(uri: &str, command_tx: Sender<Command>) -> Handler
where
    for<'de> T: ConfigData<'de>,
{
    Handler::new(uri, Method::Post, move |mut req| {
        let body = req.as_bytes()?;

        let form: T = serde_json::from_reader(body.as_slice()).unwrap();
        command_tx.send(form.into())?;

        let resp = "Gespeichert!";
        Ok(Response::new(200).body(resp.into()))
    })
}

pub fn httpd(command_tx: Sender<Command>) -> Result<Server> {
    let server = esp_idf_svc::httpd::ServerRegistry::new()
        .handler(favicon_handler())?
        .at("/simple")
        .get(|_| {
            let settings_path = &format!("{}/simple.htm", BASE_DIR);
            let settings = File::open(settings_path)
                .with_context(|| format!("Could not find html: {}", settings_path))?;

            Ok(settings.into())
        })?
        .at("/")
        .get(|_| {
            let settings_path = &format!("{}/settings/overview.htm", BASE_DIR);
            let settings = File::open(settings_path)
                .with_context(|| format!("Could not find html: {}", settings_path))?;

            Ok(settings.into())
        })?
        .at("/personal")
        .get(|_| {
            let settings_path = &format!("{}/settings/personal.htm", BASE_DIR);
            let settings = File::open(settings_path)
                .with_context(|| format!("Could not find html: {}", settings_path))?;

            Ok(settings.into())
        })?
        .at("/wifi")
        .get(|_| {
            let settings_path = &format!("{}/settings/wifi.htm", BASE_DIR);
            let settings = File::open(settings_path)
                .with_context(|| format!("Could not find html: {}", settings_path))?;

            Ok(settings.into())
        })?
        .handler(settings_handler::<PersonalData>(
            "/personal",
            command_tx.clone(),
        ))?
        .handler(settings_handler::<WifiData>("/wifi", command_tx))?;

    server.start(&Default::default())
}
