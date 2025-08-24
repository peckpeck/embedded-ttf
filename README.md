[![crates.io](https://img.shields.io/crates/v/embedded-ttf)](https://crates.io/crates/embedded-ttf)
[![docs.rs](https://img.shields.io/docsrs/embedded-ttf)](https://docs.rs/embedded-ttf)
[![Crates.io License](https://img.shields.io/crates/l/embedded-ttf/0.2.0)](https://github.com/peckpeck/embedded-ttf/blob/main/LICENSE)

# Embedded TTF rendering

Font rendering (ttf and otf) for [embedded-graphics](https://docs.rs/embedded-graphics/latest/embedded_graphics/) (no_std).

## Notable dependencies

* [embedded-graphics-core](https://github.com/embedded-graphics/embedded-graphics) used for cross platform 2D rendering and provides base text styling
* [RustType](https://gitlab.redox-os.org/redox-os/rusttype) used for rendering ttf and otf fonts

## Ho to use
```rust
let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(350, 200));

let style = FontTextStyleBuilder::new(
    Font::try_from_bytes(include_bytes!("../assets/Roboto-Regular.ttf")).unwrap())
    .font_size(16)
    .text_color(Rgb565::WHITE)
    .build();

Text::new("Hello World!", Point::new(15, 30), style).draw(&mut display)?;
```
