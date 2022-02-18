//! Command type that the main task listenes for.

use crate::server::{PersonalData, WifiData};

pub enum Command {
    SavePersonalConfig(PersonalData),
    SaveWifiConfig(WifiData),
    SwitchPage,
}
