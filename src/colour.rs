#![warn(clippy::all, clippy::pedantic)]

#[derive(Debug, PartialEq)]
pub enum ErrorCode {
  InvalidHexCharacter(String),
  InvalidHexLength(usize),
}


#[derive(Debug, PartialEq)]
pub struct Rgb {
  pub red: u8,
  pub green: u8,
  pub blue: u8,
}

impl Rgb {
  pub fn from_tuple(red: u8, green: u8, blue: u8) -> Rgb {
    Rgb { red, green, blue }
  }

  pub fn from_hex(hex: &str) -> Result<Rgb, ErrorCode> {
    if !is_valid_hex(hex) {
      return match hex.len() {
        3 | 6 => Err(ErrorCode::InvalidHexCharacter(hex.to_string())),
        len => Err(ErrorCode::InvalidHexLength(len)),
      };
    }

    match hex.len() {
      3 => {
        let chars: Vec<char> = hex.chars().collect();
        Rgb::from_hex(&format!(
          "{r}{r}{g}{g}{b}{b}",
          r = chars[0],
          g = chars[1],
          b = chars[2]
        ))
      }
      6 => {
        let chars: Vec<char> = hex.chars().collect();
        let parsed = (
          hex_pair_to_int(chars[0], chars[1]),
          hex_pair_to_int(chars[2], chars[3]),
          hex_pair_to_int(chars[4], chars[5]),
        );
        match parsed {
          (Ok(red), Ok(green), Ok(blue)) => Ok(Rgb { red, green, blue }),
          _ => Err(ErrorCode::InvalidHexCharacter(hex.to_string())),
        }
      }
      length => Err(ErrorCode::InvalidHexLength(length)),
    }
  }

  pub fn get_contrasting_colour(&self) -> Rgb {
    let yiq: u32 =
      (u32::from(self.red) * 299 + u32::from(self.green) * 587 + u32::from(self.blue) * 114) / 1000;
    match yiq {
      yiq if yiq >= 128 => Rgb {
        red: 0,
        green: 0,
        blue: 0,
      },
      _ => Rgb {
        red: 255,
        green: 255,
        blue: 255,
      },
    }
  }

  pub fn to_hex(&self) -> String {
    format!("{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
  }
}

impl std::convert::From<&str> for Rgb {
  fn from(value: &str) -> Rgb {
    let default = Rgb {
      red: 0,
      green: 0,
      blue: 0,
    };
    if is_valid_hex(value) {
      Rgb::from_hex(value).unwrap_or(default)
    } else {
      default
    }
  }
}

impl std::convert::From<Rgb> for String {
  fn from(colour: Rgb) -> Self {
    colour.to_hex()
  }
}

fn hex_pair_to_int(a: char, b: char) -> Result<u8, std::num::ParseIntError> {
  u8::from_str_radix(&format!("{}{}", a, b), 16)
}

fn is_valid_hex(hex: &str) -> bool {
  if hex.is_empty() {
    return false;
  }

  let mut unprefixed = hex;

  if let Some(x) = hex.strip_prefix('#') {
    unprefixed = x;
  }

  match unprefixed.len() {
    3 | 6 => u32::from_str_radix(unprefixed, 16).is_ok(),
    _ => false,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod is_valid_hex {
    use super::*;

    #[test]
    fn six_char() {
      assert!(is_valid_hex("d3d09f"));
    }

    #[test]
    fn three_char() {
      assert!(is_valid_hex("fea"));
    }

    #[test]
    fn six_char_with_prefix() {
      assert!(is_valid_hex("#f34a23"));
    }

    #[test]
    fn three_char_with_prefix() {
      assert!(is_valid_hex("#a4d"));
    }

    #[test]
    fn three_char_invalid() {
      assert!(!is_valid_hex("dkl"));
    }

    #[test]
    fn three_char_with_prefix_invalid() {
      assert!(!is_valid_hex("#9x2"));
    }

    #[test]
    fn six_char_invalid() {
      assert!(!is_valid_hex("dkldsa"));
    }

    #[test]
    fn six_char_with_prefix_invalid() {
      assert!(!is_valid_hex("#9x2444"));
    }

    #[test]
    fn invalid() {
      assert!(!is_valid_hex("fdsfdsfrtre"));
    }
  }

  mod rgb_from_hex {
    use super::*;

    #[test]
    fn six_char() {
      assert_eq!(
        Rgb::from_hex("F43C8E"),
        Ok(Rgb {
          red: 244,
          green: 60,
          blue: 142
        })
      );
    }

    #[test]
    fn three_char() {
      assert_eq!(
        Rgb::from_hex("d15"),
        Ok(Rgb {
          red: 221,
          green: 17,
          blue: 85
        })
      );
    }

    #[test]
    fn invalid_char() {
      assert_eq!(
        Rgb::from_hex("F43C8X"),
        Err(ErrorCode::InvalidHexCharacter("F43C8X".to_string()))
      );
    }

    #[test]
    fn invalid_length() {
      assert_eq!(
        Rgb::from_hex("F43C"),
        Err(ErrorCode::InvalidHexLength(4))
      );
    }
  }

  mod rgb_from_string {
    use super::*;

    #[test]
    fn six_char() {
      assert_eq!(
        Rgb::from("F43C8E"),
        Rgb {
          red: 244,
          green: 60,
          blue: 142
        }
      );
    }

    #[test]
    fn three_char() {
      assert_eq!(
        Rgb::from("d15"),
        Rgb {
          red: 221,
          green: 17,
          blue: 85
        }
      );
    }

    #[test]
    fn invalid_char() {
      assert_eq!(Rgb::from("F43C8X"), Rgb { red: 0, green: 0, blue: 0 });
    }

    #[test]
    fn invalid_length() {
      assert_eq!(Rgb::from("F43C"), Rgb { red: 0, green: 0, blue: 0 });
    }
  }

  mod rgb_contrasting_colour {
    use super::*;

    #[test]
    fn dark_colour() {
      assert_eq!(
        Rgb::from("054").get_contrasting_colour(),
        Rgb {
          red: 255,
          green: 255,
          blue: 255
        }
      );
    }

    #[test]
    fn light_colour() {
      assert_eq!(
        Rgb::from("f54").get_contrasting_colour(),
        Rgb {
          red: 0,
          green: 0,
          blue: 0
        }
      );
    }
  }
}