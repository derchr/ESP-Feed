use crate::{
    graphics::display::Display,
    wifi::WifiConfig,
    feed::FeedController,
    graphics::pages::{ConfigPage, ExamplePage, FeedPage, Page, PageType},
};
use std::sync::{Arc, Mutex};

pub struct State {
    pub feed_controller: FeedController,
    pub setup_mode: bool,
    pub page: PageType,
    pub wifi: Option<WifiConfig>,
    pub location: String,
    pub width: u32,
}

impl State {
    pub fn new(setup_mode: bool, wifi_config: Option<WifiConfig>) -> Self {
        let page = if setup_mode {
            ConfigPage.into()
        } else {
            // ExamplePage.into()
            FeedPage.into()
        };

        Self {
            feed_controller: FeedController::new(),
            setup_mode,
            page: page,
            wifi: wifi_config,
            location: String::new(),
            width: 128,
        }
    }

    pub fn next_page(&mut self) {
        match self.page.next_page() {
            PageType::ConfigPage(_) => self.page = ConfigPage.into(),
            PageType::ExamplePage(_) => self.page = ExamplePage.into(),
            PageType::FeedPage(_) => self.page = FeedPage.into(),
        }
    }
}
