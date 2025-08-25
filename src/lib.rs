//! Font rendering (ttf and otf) with embedded-graphics.
//!
//! Embedded graphics provides static mono font rendering directly from the code.
//! But it can render any font if the proper trait is implemented.
//!
//! This is an implementation that uses the [rusttype](https://gitlab.redox-os.org/redox-os/rusttype)
//! crate to parse ttf and otf fonts before rendering them on a `DrawTarget`
//!
//! # Usage
//!
//! Use [`FontTextStyleBuilder`] to easily create a [`FontTextStyle`] object.
//!
//! This style can then be directly used with embedded graphics' [`Text`] struct.
//!
//! ```
//! let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(350, 200));
//!
//! let style = FontTextStyleBuilder::new(
//!     Font::try_from_bytes(include_bytes!("../assets/Roboto-Regular.ttf")).unwrap())
//!     .font_size(16)
//!     .text_color(Rgb565::WHITE)
//!     .build();
//!
//! Text::new("Hello World!", Point::new(15, 30), style).draw(&mut display)?;
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use num_traits::float::FloatCore;

#[cfg(feature = "std")]
use std as stdlib;

#[cfg(not(feature = "std"))]
mod stdlib {
    pub use ::alloc::vec;
    pub use core::*;
}

use stdlib::{f32, vec::Vec};

use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb888,
    prelude::*,
    primitives::Rectangle,
    text::{
        renderer::{CharacterStyle, TextMetrics, TextRenderer},
        Baseline, DecorationColor,
    },
};

use rusttype::Font;

/// Antialiasing can be difficult with embedded graphics since the background pixel is not known
/// during the drawing process.
#[derive(Debug, Clone)]
pub enum AntiAliasing<C> {
    /// Use the font background color (default), choose this if you defined a background color.
    /// This is equivalent to SolidColor if the background color is defined,
    /// This is equivalent to None if the background color is not defined
    BackgroundColor,
    /// Use given color as a "known" background color.
    SolidColor(C),
    /// Replace the alpha channel with a simple transparency (cutoff at 50%), choose this if you don't know the background at all.
    None,
}

/// Style properties for text using a ttf and otf font.
///
/// A `FontTextStyle` can be applied to a [`Text`] object to define how the text is drawn.
///
#[derive(Debug, Clone)]
pub struct FontTextStyle<C> {
    /// Text color.
    pub text_color: Option<C>,

    /// Background color.
    pub background_color: Option<C>,

    /// How to apply antialiasing.
    pub anti_aliasing: AntiAliasing<C>,

    /// Underline color.
    pub underline_color: DecorationColor<C>,

    /// Strikethrough color.
    pub strikethrough_color: DecorationColor<C>,

    /// Font size.
    pub font_size: u32,

    /// Font from rusttype.
    font: Font<'static>,
}

impl<C: PixelColor> FontTextStyle<C> {
    /// Creates a text style with a transparent background.
    pub fn new(font: Font<'static>, text_color: C, font_size: u32) -> Self {
        FontTextStyleBuilder::new(font)
            .text_color(text_color)
            .font_size(font_size)
            .build()
    }

    /// Resolves a decoration color.
    fn resolve_decoration_color(&self, color: DecorationColor<C>) -> Option<C> {
        match color {
            DecorationColor::None => None,
            DecorationColor::TextColor => self.text_color,
            DecorationColor::Custom(c) => Some(c),
        }
    }

    fn draw_background<D>(
        &self,
        width: u32,
        position: Point,
        target: &mut D,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        if width == 0 {
            return Ok(());
        }

        if let Some(background_color) = self.background_color {
            target.fill_solid(
                &Rectangle::new(position, Size::new(width, self.font_size)),
                background_color,
            )?;
        }

        Ok(())
    }

    fn draw_strikethrough<D>(
        &self,
        width: u32,
        position: Point,
        target: &mut D,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        if let Some(strikethrough_color) = self.resolve_decoration_color(self.strikethrough_color) {
            let top_left = position + Point::new(0, self.font_size as i32 / 2);
            // small strikethrough width
            let size = Size::new(width, self.font_size / 30 + 1);

            target.fill_solid(&Rectangle::new(top_left, size), strikethrough_color)?;
        }

        Ok(())
    }

    fn draw_underline<D>(&self, width: u32, position: Point, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        if let Some(underline_color) = self.resolve_decoration_color(self.underline_color) {
            let top_left = position + Point::new(0, self.font_size as i32);
            // small underline width
            let size = Size::new(width, self.font_size / 30 + 1);

            target.fill_solid(&Rectangle::new(top_left, size), underline_color)?;
        }

        Ok(())
    }
}

impl<C: PixelColor> CharacterStyle for FontTextStyle<C> {
    type Color = C;

    fn set_text_color(&mut self, text_color: Option<Self::Color>) {
        self.text_color = text_color;
    }

    fn set_background_color(&mut self, background_color: Option<Self::Color>) {
        self.background_color = background_color;
        if background_color.is_some() {
            // best antialiasing in this case
            self.anti_aliasing = AntiAliasing::BackgroundColor;
        }
    }

    fn set_underline_color(&mut self, underline_color: DecorationColor<Self::Color>) {
        self.underline_color = underline_color;
    }

    fn set_strikethrough_color(&mut self, strikethrough_color: DecorationColor<Self::Color>) {
        self.strikethrough_color = strikethrough_color;
    }
}

impl<C> TextRenderer for FontTextStyle<C>
where
    C: PixelColor + Into<Rgb888> + From<Rgb888> + stdlib::fmt::Debug,
{
    type Color = C;

    fn draw_string<D>(
        &self,
        text: &str,
        position: Point,
        _baseline: Baseline,
        target: &mut D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let scale = rusttype::Scale::uniform(self.font_size as f32);

        let v_metrics = self.font.v_metrics(scale);
        let offset = rusttype::point(0.0, v_metrics.ascent);

        let glyphs: Vec<rusttype::PositionedGlyph> =
            self.font.layout(text, scale, offset).collect();

        let width = glyphs
            .iter()
            .rev()
            .filter_map(|g| {
                g.pixel_bounding_box()
                    .map(|b| b.min.x as f32 + g.unpositioned().h_metrics().advance_width)
            })
            .next()
            .unwrap_or(0.0)
            .ceil() as i32;

        let height = self.font_size as i32;

        let mut pixels = Vec::new();

        if let Some(text_color) = self.text_color {
            for g in glyphs.iter() {
                if let Some(bb) = g.pixel_bounding_box() {
                    g.draw(|off_x, off_y, v| {
                        let off_x = off_x as i32 + bb.min.x;
                        let off_y = off_y as i32 + bb.min.y;
                        // There's still a possibility that the glyph clips the boundaries of the bitmap
                        if off_x >= 0 && off_x < width as i32 && off_y >= 0 && off_y < height as i32
                        {
                            let c = (v * 255.0) as u32;

                            let (text_r, text_g, text_b, text_a) =
                                u32_to_rgba(c << 24 | (pixel_color_to_u32(text_color) & 0xFFFFFF));

                            let bg_color = match self.anti_aliasing {
                                AntiAliasing::BackgroundColor => self.background_color,
                                AntiAliasing::SolidColor(c) => Some(c),
                                AntiAliasing::None => None,
                            };
                            match bg_color {
                                None => if text_a > 127 {
                                    pixels.push(Pixel(
                                        Point::new(position.x + off_x, position.y + off_y),
                                        Rgb888::new(text_r, text_g, text_b).into(),
                                    ));
                                }
                                Some(color) => {
                                    let (new_r, new_g, new_b) = rgba_blend(
                                        text_r,
                                        text_g,
                                        text_b,
                                        text_a,
                                        color,
                                    );
                                    pixels.push(Pixel(
                                        Point::new(position.x + off_x, position.y + off_y),
                                        Rgb888::new(new_r, new_g, new_b).into(),
                                    ));
                                }
                            }
                        }
                    });
                }
            }
        }

        self.draw_background(width as u32, position, target)?;
        target.draw_iter(pixels)?;
        self.draw_strikethrough(width as u32, position, target)?;
        self.draw_underline(width as u32, position, target)?;

        Ok(position + Point::new(width, 0))
    }

    fn draw_whitespace<D>(
        &self,
        width: u32,
        position: Point,
        _baseline: Baseline,
        target: &mut D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.draw_background(width, position, target)?;
        self.draw_strikethrough(width, position, target)?;
        self.draw_underline(width, position, target)?;

        Ok(position + Size::new(width, 0))
    }

    fn measure_string(&self, text: &str, position: Point, _baseline: Baseline) -> TextMetrics {
        let scale = rusttype::Scale::uniform(self.font_size as f32);
        let v_metrics = self.font.v_metrics(scale);
        let offset = rusttype::point(0.0, v_metrics.ascent);

        let glyphs: Vec<rusttype::PositionedGlyph> =
            self.font.layout(text, scale, offset).collect();

        let width = glyphs
            .iter()
            .rev()
            .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
            .next()
            .unwrap_or(0.0)
            .ceil() as f64;

        let size = Size::new(width as u32, self.font_size);

        TextMetrics {
            bounding_box: Rectangle::new(position, size),
            next_position: position + size.x_axis(),
        }
    }

    fn line_height(&self) -> u32 {
        self.font_size
    }
}

/// Text style builder for ttf and otf fonts.
///
/// Use this builder to create [`FontTextStyle`]s for [`Text`].
pub struct FontTextStyleBuilder<C: PixelColor> {
    style: FontTextStyle<C>,
}

impl<C: PixelColor> FontTextStyleBuilder<C> {
    /// Create a new text style builder.
    pub fn new(font: Font<'static>) -> Self {
        Self {
            style: FontTextStyle {
                font,
                background_color: None,
                anti_aliasing: AntiAliasing::None,
                font_size: 12,
                text_color: None,
                underline_color: DecorationColor::None,
                strikethrough_color: DecorationColor::None,
            },
        }
    }

    /// Set the font size of the style in pixels.
    pub fn font_size(mut self, font_size: u32) -> Self {
        self.style.font_size = font_size;
        self
    }

    /// Enable underline using the text color.
    pub fn underline(mut self) -> Self {
        self.style.underline_color = DecorationColor::TextColor;
        self
    }

    /// Enable strikethrough using the text color.
    pub fn strikethrough(mut self) -> Self {
        self.style.strikethrough_color = DecorationColor::TextColor;
        self
    }

    /// Set the text color.
    pub fn text_color(mut self, text_color: C) -> Self {
        self.style.text_color = Some(text_color);
        self.style.anti_aliasing = AntiAliasing::BackgroundColor;
        self
    }

    /// Set the background color.
    pub fn background_color(mut self, background_color: C) -> Self {
        self.style.background_color = Some(background_color);
        self
    }

    /// Apply antialiasing over a known color.
    pub fn anti_aliasing_color(mut self, background_color: C) -> Self {
        self.style.anti_aliasing = AntiAliasing::SolidColor(background_color);
        self
    }

    /// Enable underline with a custom color.
    pub fn underline_with_color(mut self, underline_color: C) -> Self {
        self.style.underline_color = DecorationColor::Custom(underline_color);
        self
    }

    /// Enable strikethrough with a custom color.
    pub fn strikethrough_with_color(mut self, strikethrough_color: C) -> Self {
        self.style.strikethrough_color = DecorationColor::Custom(strikethrough_color);

        self
    }

    /// Build the text style.
    pub fn build(self) -> FontTextStyle<C> {
        self.style
    }
}

fn pixel_color_to_u32<C: Into<Rgb888>>(color: C) -> u32 {
    let color = color.into();

    0xFF000000 | ((color.r() as u32) << 16) | ((color.g() as u32) << 8) | (color.b() as u32)
}

fn u32_to_rgba(color: u32) -> (u8, u8, u8, u8) {
    (
        ((color & 0x00FF0000) >> 16) as u8,
        ((color & 0x0000FF00) >> 8) as u8,
        (color & 0x000000FF) as u8,
        ((color & 0xFF000000) >> 24) as u8,
    )
}

fn rgba_to_rgb(r: u8, g: u8, b: u8, a: u8) -> (u8, u8, u8) {
    let alpha = a as f32 / 255.;

    (
        (r as f32 * alpha).ceil() as u8,
        (g as f32 * alpha).ceil() as u8,
        (b as f32 * alpha).ceil() as u8,
    )
}

fn rgba_blend<C: Into<Rgb888>>(
    r: u8,
    g: u8,
    b: u8,
    a: u8,
    background_color: C,
) -> (u8, u8, u8) {
    let background_color_data = pixel_color_to_u32(background_color);
    let (br, bg, bb, ba) = u32_to_rgba(background_color_data);
    let (br, bg, bb) = rgba_to_rgb(br, bg, bb, ba);

    let alpha = a as f32 / 255.;
    let b_alpha = 1. - alpha;

    // blend with background color
    (
        ((r as f32 * alpha) + br as f32 * b_alpha).ceil() as u8,
        ((g as f32 * alpha) + bg as f32 * b_alpha).ceil() as u8,
        ((b as f32 * alpha) + bb as f32 * b_alpha).ceil() as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::pixelcolor::Rgb888;

    #[test]
    fn test_pixel_color_to_u32() {
        assert_eq!(4294967295, pixel_color_to_u32(Rgb888::WHITE));
        assert_eq!(4278190080, pixel_color_to_u32(Rgb888::BLACK));
    }

    #[test]
    fn test_u32_to_rgba() {
        assert_eq!((255, 255, 255, 255), u32_to_rgba(4294967295));
        assert_eq!((0, 0, 0, 255), u32_to_rgba(4278190080));
    }

    #[test]
    fn test_rgba_to_rgb() {
        assert_eq!((255, 255, 255), rgba_to_rgb(255, 255, 255, 255));
        assert_eq!((100, 100, 100), rgba_to_rgb(255, 255, 255, 100));
    }

    #[test]
    fn test_rgba_background_to_rgb() {
        assert_eq!(
            (255, 255, 255),
            rgba_background_to_rgb::<Rgb888>(255, 255, 255, 255, None)
        );
        assert_eq!(
            (100, 100, 100),
            rgba_background_to_rgb(255, 255, 255, 100, Some(Rgb888::BLACK))
        );
    }
}
