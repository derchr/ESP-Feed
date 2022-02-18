//! Module that handles the drawing to the display.

pub mod display;
pub mod pages;
mod style;
mod views;

use crate::{datetime, state::State};
// use display::OledDisplay;
use anyhow::Result;
use display::EpdDisplay;
use embedded_graphics::{
    mono_font::{iso_8859_1::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
};
use embedded_text::{
    alignment::{HorizontalAlignment, VerticalAlignment},
    style::TextBoxStyleBuilder,
    TextBox,
};
use epd_waveshare::{color::Color, prelude::*};
use pages::Page;
use std::{
    sync::{mpsc::Receiver, Arc, Mutex},
    time::Duration,
};

pub fn draw_pages(
    display: &mut EpdDisplay,
    state: Arc<Mutex<State>>,
    update_page_rx: Receiver<()>,
) -> Result<()> {
    loop {
        let _ = update_page_rx.recv_timeout(Duration::from_secs(60));

        display
            .epd2in13
            .wake_up(&mut display.master, &mut display.delay)?;
        display.display.clear_buffer(Color::White);
        // display.epd2in13.clear_frame(&mut display.master, &mut display.delay)?;
        {
            let state = state.lock().unwrap();
            let page = &state.page;
            // let target = display.display.as_mut();

            // let style = PrimitiveStyleBuilder::new()
            //     .stroke_color(BinaryColor::On)
            //     .stroke_width(1)
            //     .fill_color(BinaryColor::Off)
            //     .build();

            let height = 12;
            // Line::new(
            //     Point::new(0, height),
            //     Point::new(display.display.bounding_box().size.width as _, height),
            // )
            // .into_styled(style)
            // .draw(display.display.as_mut())?;

            let status_bar_area = Rectangle::new(
                Point::zero(),
                Size::new(display.display.bounding_box().size.width, height as _),
            );

            let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
            let text_box_left_style = TextBoxStyleBuilder::new()
                .alignment(HorizontalAlignment::Left)
                .vertical_alignment(VerticalAlignment::Middle)
                .build();
            let text_box_center_style = TextBoxStyleBuilder::new()
                .alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Middle)
                .build();
            let text_box_right_style = TextBoxStyleBuilder::new()
                .alignment(HorizontalAlignment::Right)
                .vertical_alignment(VerticalAlignment::Middle)
                .build();

            if let Ok(datetime) = datetime::get_datetime() {
                let date_format = time::format_description::parse("[day].[month].[year]")
                    .expect("Invalid format.");

                let time_format =
                    time::format_description::parse("[hour]:[minute]").expect("Invalid format.");

                let date = datetime
                    .format(&date_format)
                    .expect("Could not format time.");
                let time = datetime
                    .format(&time_format)
                    .expect("Could not format time.");

                TextBox::with_textbox_style(
                    &date,
                    status_bar_area,
                    text_style,
                    text_box_left_style,
                )
                .translate(Point::new(1, 0)) // TODO remove border bug
                .draw(display.display.as_mut())?;
                TextBox::with_textbox_style(
                    &time,
                    status_bar_area,
                    text_style,
                    text_box_center_style,
                )
                .draw(display.display.as_mut())?;
            }

            TextBox::with_textbox_style(
                &state.location,
                status_bar_area,
                text_style,
                text_box_right_style,
            )
            .draw(display.display.as_mut())?;

            // TODO: Percentage

            let page_area = Rectangle::new(
                Point::new(0, height),
                Size::new(
                    display.display.bounding_box().size.width,
                    display.display.bounding_box().size.height - height as u32,
                ),
            );
            let mut page_draw_target = display.display.cropped(&page_area);
            page.draw(&mut page_draw_target, &state).unwrap();
        }
        display.epd2in13.update_frame(
            &mut display.master,
            display.display.buffer(),
            &mut display.delay,
        )?;
        display
            .epd2in13
            .display_frame(&mut display.master, &mut display.delay)?;
        display
            .epd2in13
            .sleep(&mut display.master, &mut display.delay)?;

        // std::thread::sleep(std::time::Duration::from_secs(0xFFFF_FFFF_FFFF_FFFF));
        // std::thread::sleep(Duration::from_millis(15000));
    }
}
