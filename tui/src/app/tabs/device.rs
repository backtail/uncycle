use ratatui::{
    layout::Rect,
    style::{Color, Stylize},
    text::ToLine,
    Frame,
};
use tui_big_text::*;
use uncycle_core::prelude::*;

use crate::app::{widgets, App};
use crate::device::*;

pub fn render_device_tab(f: &mut Frame, app: &App, area: Rect) {
    let maybe_device;
    {
        maybe_device = app.core.lock().unwrap().device.clone();
    }

    if let Some(device) = maybe_device {
        match device {
            SupportedDevice::TR8(_) => tr8::render(f, app, area),
            // add new devices manually
        }
    } else {
        no_device(f, area);
    }
}

fn no_device(f: &mut Frame, area: Rect) {
    let text = vec![
        "".to_line(),
        "No Device".to_line().dark_gray(),
        "Selected".to_line().dark_gray(),
    ];

    let render = BigText::builder()
        .pixel_size(PixelSize::Full)
        .lines(text)
        .centered()
        .build();

    f.render_widget(widgets::border_rounded("", Color::DarkGray), area);
    f.render_widget(render, area);
}
