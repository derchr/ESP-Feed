//! Handles the non-volatile storage of the ESP32.

use crate::server::ConfigData;
use anyhow::{Context, Result};
use embedded_svc::storage::Storage;
use esp_idf_svc::{nvs::EspDefaultNvs, nvs_storage::EspNvsStorage};
use serde::{de::DeserializeOwned, Serialize};
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

    pub fn store_config<'de, T>(&mut self, config: &T) -> Result<()>
    where
        T: ConfigData<'de>,
    {
        self.storage
            .put(T::key(), config)
            .context("Could not store config into NVS")?;

        Ok(())
    }

    pub fn get_config<T>(&self) -> Result<T>
    where
        for<'de> T: ConfigData<'de>,
    {
        let config = self
            .storage
            .get(T::key())?
            .context("Could not read config from NVS")?;

        Ok(config)
    }

    pub fn get<T>(&self, key: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let value = self
            .storage
            .get(key)?
            .with_context(|| format!("Could not get key \"{}\" from NVS", key))?;

        Ok(value)
    }

    pub fn store<T>(&mut self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.storage
            .put(key, value)
            .with_context(|| format!("Could not store key \"{}\" into NVS", key))?;

        Ok(())
    }
}
