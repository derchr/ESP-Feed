pub mod display;
pub mod pages;
mod style;
mod views;

use crate::state::State;
use display::Display;
use pages::Page;

use anyhow::Result;
use embedded_graphics::{prelude::*, primitives::Rectangle};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

pub fn draw_pages(display: &mut Display, state: Arc<Mutex<State>>) -> Result<()> {
    loop {
        display.clear();
        {
            let state = state.lock().unwrap();
            let page = &state.page;

            page.draw(display, &state).unwrap();
        }
        display.flush().unwrap();

        // std::thread::sleep(std::time::Duration::from_secs(0xFFFF_FFFF_FFFF_FFFF));
        std::thread::sleep(Duration::from_millis(100));
    }
}
