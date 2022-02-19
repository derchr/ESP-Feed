//! Command type that the main task listenes for.

use crate::server::{PersonalData, RssData, StockData, WifiData};

pub enum Command {
    SavePersonalConfig(PersonalData),
    SaveWifiConfig(WifiData),
    SaveRssConfig(RssData),
    SaveStockConfig(StockData),
    SwitchPage,
}
