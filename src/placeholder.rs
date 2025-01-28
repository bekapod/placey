#![warn(clippy::all, clippy::pedantic)]
use crate::colour::Rgb;
use image::ImageOutputFormat::Png;
use image::{DynamicImage, RgbImage};
use imageproc::drawing::{draw_text_mut, text_size};
use rusttype::{Font, Scale};
use std::io::Cursor;

const MAX_AREA: u32 = 1_000_000;
const MAX_SIDE: u16 = 2500;

#[allow(clippy::cast_possible_truncation)] // the whole point of this is to truncate 😜
fn convert_f64_to_i32(x: f64) -> i32 {
    x.round().rem_euclid(2f64.powi(32)) as i32
}

fn check_size(w: u16, h: u16) -> Result<(), ()> {
    if w == 0 || h == 0 {
        return Err(());
    }

    if u32::from(w) * u32::from(h) <= MAX_AREA && w <= MAX_SIDE && h <= MAX_SIDE {
        return Ok(());
    }

    Err(())
}

#[derive(Debug, Default)]
pub struct GenerateOptions {
    pub width: u16,
    pub height: Option<u16>,
    pub background_colour: Option<String>,
    pub foreground_colour: Option<String>,
}

/// # Errors
///
/// Will return `Err` if desired width & height would result in an image that is too big.
#[tracing::instrument]
pub fn generate(opts: GenerateOptions) -> Result<(Vec<u8>, String), (u16, String)> {
    match (
        opts.width,
        opts.height,
        opts.background_colour,
        opts.foreground_colour,
    ) {
        (w, Some(h), Some(bg), Some(fg)) if check_size(w, h).is_ok() => {
            create_image(w, h, &bg, &fg)
        }
        (w, Some(h), Some(bg), None) if check_size(w, h).is_ok() => create_image(w, h, &bg, ""),
        (w, Some(h), None, Some(fg)) if check_size(w, h).is_ok() => create_image(w, h, "", &fg),
        (w, Some(h), None, None) if check_size(w, h).is_ok() => create_image(w, h, "", ""),
        (w, None, Some(bg), Some(fg)) if check_size(w, w).is_ok() => create_image(w, w, &bg, &fg),
        (w, None, Some(bg), None) if check_size(w, w).is_ok() => create_image(w, w, &bg, ""),
        (w, None, None, Some(fg)) if check_size(w, w).is_ok() => create_image(w, w, "", &fg),
        (w, None, None, None) if check_size(w, w).is_ok() => create_image(w, w, "", ""),
        (_, _, _, _) => Err((422, format!("Image too big. Total area must be less than or equal to {MAX_AREA}px and the maximum length of any side must be less than or equal to {MAX_SIDE}px."))),
    }
}

#[tracing::instrument]
fn create_image(
    width: u16,
    height: u16,
    background_colour: &str,
    foreground_colour: &str,
) -> Result<(Vec<u8>, String), (u16, String)> {
    let mut buffer = Cursor::new(vec![]);
    let background_colour = Rgb::from_hex(background_colour).unwrap_or(Rgb {
        red: 255,
        green: 255,
        blue: 0,
    });
    let foreground_colour = Rgb::from_hex(foreground_colour)
        .unwrap_or_else(|_| background_colour.get_contrasting_colour());

    let mut img = RgbImage::from_pixel(
        width.into(),
        height.into(),
        image::Rgb([
            background_colour.red,
            background_colour.green,
            background_colour.blue,
        ]),
    );

    let font = Vec::from(include_bytes!("assets/AzeretMono-Regular.ttf") as &[u8]);
    let font = Font::try_from_vec(font);

    if let Some(font) = font {
        let text = format!("{width}×{height}");

        let scale = match f32::from(width) {
            w if w > 999.0 => w * 0.15,
            w if w < 100.0 => w * 0.25,
            w => w * 0.2,
        };
        let scale = Scale { x: scale, y: scale };

        let (text_width, text_height) = text_size(scale, &font, &text);
        let x = i32::from(width / 2) - (text_width / 2);
        let y = i32::from(height / 2) - convert_f64_to_i32(f64::from(text_height) / 1.65);

        draw_text_mut(
            &mut img,
            image::Rgb([
                foreground_colour.red,
                foreground_colour.green,
                foreground_colour.blue,
            ]),
            x,
            y,
            scale,
            &font,
            &text,
        );
    } else {
        log::error!("Invalid font data.");
    }

    match DynamicImage::ImageRgb8(img).write_to(&mut buffer, Png) {
        Ok(()) => Ok((buffer.into_inner(), String::from("png"))),
        Err(_) => Err((500, String::from("Failed to generate image."))),
    }
}

#[cfg(test)]
const MAX_TIME: u128 = 100;

#[test]
fn check_image_with_no_size() {
    assert!(check_size(0, 0).is_err());
}

#[test]
fn check_image_with_no_height() {
    assert!(check_size(500, 0).is_err());
}

#[test]
fn check_image_with_no_width() {
    assert!(check_size(500, 0).is_err());
}

#[test]
fn check_image_with_large_area() {
    assert!(check_size(1001, 1000).is_err());
    assert!(check_size(9999, 9999).is_err());
}

#[test]
fn check_image_with_large_height() {
    assert!(check_size(1, 9999).is_err());
    assert!(check_size(1, 2501).is_err());
}

#[test]
fn check_image_with_large_width() {
    assert!(check_size(9999, 1).is_err());
    assert!(check_size(2501, 1).is_err());
}

#[test]
fn check_image_with_acceptable_size() {
    assert!(check_size(1, 1).is_ok());
    assert!(check_size(1000, 1000).is_ok());
    assert!(check_size(100, 2500).is_ok());
    assert!(check_size(2500, 250).is_ok());
}

#[test]
fn generate_fuzz() {
    use quickcheck::quickcheck;
    use std::time::SystemTime;

    fn prop(width: u16, height: u16) -> bool {
        let start = SystemTime::now();
        let result = generate(GenerateOptions {
            width,
            height: Some(height),
            ..GenerateOptions::default()
        });
        let end = SystemTime::now();
        let elapsed = end.duration_since(start).unwrap().as_millis();

        if elapsed > MAX_TIME {
            return false;
        }

        match result {
            Ok(_) => true,
            Err((status, _)) => {
                // expected error
                if check_size(width, height).is_err() && status == 422 {
                    return true;
                }

                false
            }
        }
    }

    quickcheck(prop as fn(u16, u16) -> bool);
}
