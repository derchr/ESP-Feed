use anyhow::*;
use embedded_svc::wifi::*;
use esp_idf_svc::{netif::*, nvs::*, sysloop::*, wifi::*};
use log::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const SSID: Option<&str> = option_env!("WIFI_SSID");
const PASS: Option<&str> = option_env!("WIFI_PASS");

#[derive(Serialize, Deserialize, Clone)]
pub struct WifiConfig {
    pub ssid: String,
    pub pass: String,
}

pub fn connect(
    wifi_config: Option<WifiConfig>,
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
) -> Result<EspWifi> {
    let mut wifi = EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?;

    info!("Wifi created, about to scan");
    let WifiConfig { ssid, pass } = wifi_config.unwrap_or(WifiConfig {
        ssid: SSID.unwrap().to_string(),
        pass: PASS.unwrap().to_string(),
    });

    let ap_info_list = wifi.scan()?;
    let ap_info = ap_info_list.into_iter().find(|a| a.ssid == ssid);

    let channel = if let Some(ap_info) = ap_info {
        info!(
            "Found configured access point {} on channel {}",
            ssid, ap_info.channel
        );
        Some(ap_info.channel)
    } else {
        warn!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            ssid
        );
        None
    };

    let configuration = Configuration::Client(ClientConfiguration {
        ssid,
        password: pass,
        channel,
        ..Default::default()
    });

    wifi.set_configuration(&configuration)?;

    info!("Wifi configuration set, about to get status");

    let status = wifi.get_status();

    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
        _,
    ) = &status
    {
        info!("Wifi connected!");
        info!(
            "My IP is {}, Subnet: {}, DNS: {:?}",
            ip_settings.ip,
            ip_settings.subnet.to_string(),
            ip_settings.dns
        );
    }

    Ok(wifi)
}

pub fn create_accesspoint(
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
) -> Result<EspWifi> {
    let mut wifi = EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?;

    let configuration = Configuration::AccessPoint(AccessPointConfiguration {
        ssid: "ESP-Feed".into(),
        channel: 1,
        auth_method: AuthMethod::WPA2Personal,
        password: "38294446".into(),
        ..Default::default()
    });

    wifi.set_configuration(&configuration)?;

    info!("Wifi configuration set, about to get status");

    let status = wifi.get_status();
    if let Status(_, ApStatus::Started(ApIpStatus::Done)) = &status {
        info!("Accesspoint configured!");
    }

    Ok(wifi)
}
