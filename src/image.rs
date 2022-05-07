#![warn(clippy::all, clippy::pedantic)]
use image::{DynamicImage, RgbImage, Rgb};
use image::ImageOutputFormat::PNG;

pub fn generate(width: u16, height: u16) -> Result<(Vec<u8>, String), (u16, String)> {
  match (width, height) {
    (0, _) | (_, 0) => Err((422, String::from("Cannot generate an image with no size."))),
    (w, h) if u64::from(w) * u64::from(h) <= 1_000_000 => {
      let mut buffer: Vec<u8> = Vec::new();
      let rgb = RgbImage::from_pixel(w.into(), h.into(), Rgb([255, 255, 0]));
      let img = DynamicImage::ImageRgb8(rgb);

      match img.write_to(&mut buffer, PNG) {
        Ok(_) => Ok((buffer, String::from("png"))),
        Err(_) => Err((500, String::from("Failed to generate image.")))
      }
    },
    (_, _) => Err((422, format!("Image too big. Total area must be less than {}px.", 1000 * 1000))),
  }
}

#[test]
fn generate_fuzz() {
  use quickcheck::quickcheck;

  fn prop(w: u16, h: u16) -> bool {
    match generate(w, h) {
      Ok(_) => true,
      Err((status, _)) => {
        // expected error
        if (w == 0 || h == 0 || u64::from(w) * u64::from(h) > 1_000_000) && status == 422 {
          return true;
        }

        false
      }
    }
  }

  quickcheck(prop as fn(u16, u16) -> bool);
}