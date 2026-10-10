//! The floetask logo: ice crystals around a done mark.
//!
//! The SVG in `assets/` is the source; `cargo run -p render-logo` renders
//! the raster icons next to it.

use iced::widget::svg;
use iced::{Element, window};

const LOGO_SVG: &[u8] = include_bytes!("../../../assets/logo.svg");
const ICON_PNG: &[u8] = include_bytes!("../../../assets/icon-256.png");

/// The logo at `size` logical pixels.
pub fn view<'a, Message: 'a>(size: f32) -> Element<'a, Message> {
    svg(svg::Handle::from_memory(LOGO_SVG)).width(size).height(size).into()
}

/// The window and taskbar icon. `None` only if the embedded PNG is broken.
pub fn window_icon() -> Option<window::Icon> {
    let decoder = png::Decoder::new(std::io::Cursor::new(ICON_PNG));
    let mut reader = decoder.read_info().ok()?;
    let mut pixels = vec![0; reader.output_buffer_size()?];
    let frame = reader.next_frame(&mut pixels).ok()?;
    pixels.truncate(frame.buffer_size());
    if frame.color_type != png::ColorType::Rgba {
        return None;
    }
    window::icon::from_rgba(pixels, frame.width, frame.height).ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn embedded_icon_decodes() {
        assert!(super::window_icon().is_some());
    }
}
