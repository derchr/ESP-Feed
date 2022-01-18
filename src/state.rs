use crate::{
    display::Display,
    feed::FeedController,
    graphics::{ConfigPage, ExamplePage, FeedPage, Page, PageType},
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Clone)]
pub struct WifiConfig {
    pub ssid: String,
    pub pass: String,
}

pub struct State {
    pub feed_controller: FeedController,
    pub setup_mode: bool,
    pub page: Box<dyn Page<Display>>,
    pub wifi: Option<WifiConfig>,
    pub location: String,
}

impl State {
    pub fn new(setup_mode: bool, wifi_config: Option<WifiConfig>) -> Self {
        let page: Box<dyn Page<Display>> = if setup_mode {
            Box::new(ConfigPage)
        } else {
            Box::new(ExamplePage)
        };

        Self {
            feed_controller: FeedController::new(),
            setup_mode,
            page: page,
            wifi: wifi_config,
            location: String::new(),
        }
    }

    pub fn page(&self) -> &dyn Page<Display> {
        &*self.page
    }

    pub fn next_page(&mut self) {
        match self.page.next_page() {
            PageType::ConfigPage => self.page = Box::new(ConfigPage),
            PageType::ExamplePage => self.page = Box::new(ExamplePage),
            PageType::FeedPage => self.page = Box::new(FeedPage),
        }
    }
}
