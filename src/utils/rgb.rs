// Neopixel and Rgb code from github.com/esp-rs/esp-idf-hal/blob/master/examples/rmt_neopixel.rs
use anyhow::Result;

pub struct Rgb {
  pub r: u8,
  pub g: u8,
  pub b: u8,
}

impl Rgb {
  pub fn from_hsv(h: u32, s: u32, v: u32) -> Result<Self> {
    if h > 360 || s > 100 || v > 100 {
      return Err(anyhow::anyhow!("Invalid HSV values"));
    }
    let s = s as f64 / 100.0;
    let v = v as f64 / 100.0;
    let c = v * s;  
    let x = c * (1.0 - (((h as f64 / 60.0) % 2.0) - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h {
      0..=59 => (c, x, 0.0),
      60..=119 => (x, c, 0.0),
      120..=179 => (0.0, c, x),
      180..=239 => (0.0, x, c),
      240..=299 => (x, 0.0, c),
      _ => (c, 0.0, x),
    };
    Ok(Self {
      r: ((r + m) * 255.0) as u8,
      g: ((g + m) * 255.0) as u8,
      b: ((b + m) * 255.0) as u8,
    })
  }
}
