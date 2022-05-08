#![warn(clippy::all, clippy::pedantic)]
use image::{RgbImage, DynamicImage, Rgb};
use image::ImageOutputFormat::Png;
use imageproc::drawing::{draw_text_mut, text_size};
use rusttype::{Font, Scale};
use std::io::{Cursor};

const MAX_AREA: u32 = 1_000_000;
const MAX_SIDE: u16 = 2500;

#[allow(clippy::cast_possible_truncation)] // the whole point of this is to truncate 😜
fn convert_f64_to_i32(x: f64) -> i32 { x.round().rem_euclid(2f64.powi(32)) as i32 }

fn check_size(w: u16, h: u16) -> Result<(), ()> {
  if w == 0 || h == 0 {
    return Err(());
  }

  if u32::from(w) * u32::from(h) <= MAX_AREA && w <= MAX_SIDE && h <= MAX_SIDE {
    return Ok(());
  }

  Err(())
}

/// # Errors
///
/// Will return `Err` if desired width & height would result in an image that is too big.
pub fn generate(width: u16, height: u16) -> Result<(Vec<u8>, String), (u16, String)> {
  match (width, height) {
    (w, h) if check_size(w, h).is_ok() => {
      let mut buffer = Cursor::new(vec![]);
      let mut rgb = RgbImage::from_pixel(w.into(), h.into(), Rgb([255, 255, 0]));

      let font = Vec::from(include_bytes!("assets/AzeretMono-Regular.ttf") as &[u8]);
      let font = Font::try_from_vec(font);

      if let Some(font) = font {
        let text = format!("{}×{}", w, h);

        let scale = match f32::from(w) {
          w if w > 999.0 => w * 0.15,
          w if w < 100.0 => w * 0.25,
          w => w * 0.2
        };
        let scale = Scale {
            x: scale,
            y: scale,
        };

        let (text_width, text_height) = text_size(scale, &font, &text);
        let x = i32::from(width / 2) - (text_width / 2);
        let y = i32::from(height / 2) - convert_f64_to_i32(f64::from(text_height) / 1.65);

        draw_text_mut(&mut rgb, Rgb([0, 0, 0]), x, y, scale, &font, &text);
      } else {
        log::error!("Invalid font data.");
      }

      let img = DynamicImage::ImageRgb8(rgb);
      match img.write_to(&mut buffer, Png) {
        Ok(_) => Ok((buffer.into_inner(), String::from("png"))),
        Err(_) => Err((500, String::from("Failed to generate image.")))
      }
    },
    (_, _) => Err((422, format!("Image too big. Total area must be less than or equal to {}px and the maximum length of any side must be less than or equal to {}px.", MAX_AREA, MAX_SIDE))),
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
  use std::time::{SystemTime};
  use quickcheck::quickcheck;

  fn prop(w: u16, h: u16) -> bool {
    let start = SystemTime::now();
    let result = generate(w, h);
    let end = SystemTime::now();
    let elapsed = end.duration_since(start).unwrap().as_millis();

    if elapsed > MAX_TIME {
      return false;
    }

    match result {
      Ok(_) => true,
      Err((status, _)) => {
        // expected error
        if check_size(w, h).is_err() && status == 422 {
          return true;
        }

        false
      }
    }
  }

  quickcheck(prop as fn(u16, u16) -> bool);
}