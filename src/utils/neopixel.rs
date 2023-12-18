// Neopixel and Rgb code from github.com/esp-rs/esp-idf-hal/blob/master/examples/rmt_neopixel.rs
use anyhow::Result;
use esp_idf_svc::hal::rmt::*;
use core::time::Duration;
#[path = "rgb.rs"] mod rgb;

pub struct Neopixel<'a> {
    tx: TxRmtDriver<'a>,
}

impl Neopixel<'_> {

    pub fn new(tx: TxRmtDriver<'_>) -> Neopixel<'_> {
        Neopixel {
            tx
        }
    }

    pub fn set_color(&mut self, rgb: rgb::Rgb) -> Result<()> {
        let color: u32 = ((rgb.r as u32) << 16) | ((rgb.g as u32) << 8) | (rgb.b as u32); 
        let ticks_hz = self.tx.counter_clock()?;
        let (t0h, t0l, t1h, t1l) = (
            Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(350))?,
            Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(800))?,
            Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(700))?,
            Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(600))?,
        );
        let mut signal = FixedLengthSignal::<24>::new();
        for i in (0..24).rev() {
            let p = 2_u32.pow(i);
            let bit: bool = p & color != 0;
            let (high_pulse, low_pulse) = if bit { (t1h, t1l) } else { (t0h, t0l) };
            signal.set(23 - i as usize, &(high_pulse, low_pulse))?;
        }
        self.tx.start_blocking(&signal)?;
        Ok(())
    }

    pub fn set_color_hsv(&mut self, h: u32, s: u32, v: u32) -> Result<()> {
        self.set_color(rgb::Rgb::from_hsv(h, s, v)?)?;
        Ok(())
    }
}

