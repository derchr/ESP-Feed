use crate::{
    feed::FeedController,
    graphics::pages::{ConfigPage, ExamplePage, FeedPage, Page, PageType, WeatherPage},
    weather::WeatherController,
    wifi::WifiConfig,
};

pub struct State {
    pub feed_controller: FeedController,
    pub weather_controller: WeatherController,
    pub setup_mode: bool,
    pub page: PageType,
    pub wifi: Option<WifiConfig>,
    pub location: String,
}

impl State {
    pub fn new(setup_mode: bool, wifi_config: Option<WifiConfig>) -> Self {
        let page = if setup_mode {
            ConfigPage.into()
        } else {
            // FeedPage.into()
            // ExamplePage.into()
            WeatherPage.into()
        };

        Self {
            feed_controller: FeedController::new(),
            weather_controller: WeatherController::new(),
            setup_mode,
            page,
            wifi: wifi_config,
            location: String::new(),
        }
    }

    pub fn next_page(&mut self) {
        match self.page.next_page() {
            PageType::ConfigPage(_) => self.page = ConfigPage.into(),
            PageType::ExamplePage(_) => self.page = ExamplePage.into(),
            PageType::FeedPage(_) => self.page = FeedPage.into(),
            PageType::WeatherPage(_) => self.page = WeatherPage.into(),
        }

        log::info!("Switched page to {:?}", self.page);
    }
}
