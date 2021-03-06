//! State struct that holds the complete state of the application.

use crate::{
    feed::FeedController,
    graphics::pages::{
        ConfigPage, ExamplePage, FeedPage, Page, PageType, StockPage, WeatherPage, WeatherPageType,
    },
    stock::StockController,
    weather::WeatherController,
    wifi::WifiConfig,
};

pub struct State {
    pub feed_controller: FeedController,
    pub weather_controller: WeatherController,
    pub stock_controller: StockController,
    pub setup_mode: bool,
    pub page: PageType,
    pub wifi: Option<WifiConfig>,
    pub location: String,
    pub battery: u16,
}

impl State {
    pub fn new(
        setup_mode: bool,
        wifi_config: Option<WifiConfig>,
        location: String,
        start_page: PageType,
        stock_symbol: &str
    ) -> Self {
        let page = if setup_mode {
            ConfigPage.into()
        } else {
            start_page
        };

        Self {
            feed_controller: FeedController::new(),
            weather_controller: WeatherController::new(),
            stock_controller: StockController::new(stock_symbol),
            setup_mode,
            page,
            wifi: wifi_config,
            location,
            battery: 0
        }
    }

    pub fn next_page(&mut self) {
        match self.page.next_page() {
            PageType::ConfigPage(_) => self.page = ConfigPage.into(),
            PageType::ExamplePage(_) => self.page = ExamplePage.into(),
            PageType::FeedPage(_) => self.page = FeedPage.into(),
            PageType::WeatherPage(WeatherPage(WeatherPageType::Daily)) => {
                self.page = WeatherPage(WeatherPageType::Daily).into()
            }
            PageType::WeatherPage(WeatherPage(WeatherPageType::Hourly)) => {
                self.page = WeatherPage(WeatherPageType::Hourly).into()
            }
            PageType::StockPage(_) => self.page = StockPage.into(),
        }

        log::info!("Switched page to {:?}", self.page);
    }
}
