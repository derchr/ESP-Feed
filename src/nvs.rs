use crate::wifi::WifiConfig;
use anyhow::{Context, Result};
use embedded_svc::storage::Storage;
use esp_idf_svc::{nvs::EspDefaultNvs, nvs_storage::EspNvsStorage};
use std::sync::Arc;

pub struct NvsController {
    storage: EspNvsStorage,
}

impl NvsController {
    pub fn new(default_nvs: Arc<EspDefaultNvs>) -> Result<Self> {
        Ok(Self {
            storage: EspNvsStorage::new_default(default_nvs, "esp_feed", true)
                .context("Failed to open NVS storage.")?,
        })
    }

    pub fn store_wifi_config(&mut self, wifi_config: &WifiConfig) -> Result<()> {
        self.storage
            .put("wifi_config", wifi_config)
            .context("Could not store wifi config into NVS")?;

        Ok(())
    }

    pub fn get_wifi_config(&self) -> Result<WifiConfig> {
        let config = self
            .storage
            .get("wifi_config")?
            .context("Could not read wifi config from NVS")?;

        Ok(config)
    }

    pub fn get_string(&self, key: &str) -> Result<String> {
        let value = self
            .storage
            .get(key)?
            .with_context(|| format!("Could not get key \"{}\" from NVS", key))?;

        Ok(value)
    }

    pub fn store_string(&mut self, key: &str, value: &String) -> Result<()> {
        self.storage.put(key, value).with_context(|| {
            format!(
                "Could not store key \"{}\" with value \"{}\" into NVS",
                key, value
            )
        })?;

        Ok(())
    }
}
