use anyhow::*;
use embedded_svc::wifi::*;
use esp_idf_svc::{netif::*, nvs::*, sysloop::*, wifi::*};
use log::*;
use std::sync::Arc;

const SSID: &str = env!("WIFI_SSID");
const PASS: &str = env!("WIFI_PASS");

pub fn wifi(
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
    ap_mode: bool,
) -> Result<EspWifi> {
    let mut wifi = EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?;

    info!("Wifi created, about to scan");

    let ap_info_list = wifi.scan()?;
    let ap_info = ap_info_list.into_iter().find(|a| a.ssid == SSID);

    let channel = if let Some(ap_info) = ap_info {
        info!(
            "Found configured access point {} on channel {}",
            SSID, ap_info.channel
        );
        Some(ap_info.channel)
    } else {
        warn!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            SSID
        );
        None
    };

    let conf = if ap_mode {
        Configuration::AccessPoint(AccessPointConfiguration {
            ssid: "ESP-Feed".into(),
            channel: channel.unwrap_or(1),
            auth_method: AuthMethod::WPA2Personal,
            password: "38294446".into(),
            ..Default::default()
        })
    } else {
        Configuration::Client(ClientConfiguration {
            ssid: SSID.into(),
            password: PASS.into(),
            channel,
            ..Default::default()
        })
    };

    wifi.set_configuration(&conf)?;

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

    if let Status(_, ApStatus::Started(ApIpStatus::Done)) = &status {
        info!("Accesspoint configured!");
    }

    Ok(wifi)
}
