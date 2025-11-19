use ratatui::{layout::Rect, Frame};

use crate::app::App;
use crate::device::tr8;

pub fn render_device_tab(f: &mut Frame, app: &App, area: Rect) {
    // todo: build logic once multiple devices are supported
    tr8::render(f, app, area);
}
