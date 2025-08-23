use embedded_graphics::{pixelcolor::Rgb565, prelude::*, text::Text, Drawable};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};

use embedded_font::*;
use rusttype::Font;

fn main() -> Result<(), core::convert::Infallible> {
    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(350, 200));

    let style = FontTextStyleBuilder::new(
        Font::try_from_bytes(include_bytes!("../assets/Roboto-Regular.ttf")).unwrap(),
    )
    .font_size(16)
    .text_color(Rgb565::WHITE)
    .build();

    Text::new("Hello World!", Point::new(15, 30), style).draw(&mut display)?;

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    Window::new("Fonts", &output_settings).show_static(&display);

    Ok(())
}
