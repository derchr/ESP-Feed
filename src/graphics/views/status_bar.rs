use embedded_graphics::{prelude::*, primitives::Line};
use embedded_layout::prelude::*;
use embedded_layout_macros::ViewGroup;

#[derive(ViewGroup)]
pub struct StatusBar {
    line: Line,
}
