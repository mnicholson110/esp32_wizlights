// Neopixel and Rgb code from github.com/esp-rs/esp-idf-hal/examples/rmt_neopixel.rs

use anyhow::Result;
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::rmt::config::TransmitConfig;
use esp_idf_svc::hal::rmt::*;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use std::net::UdpSocket;
use core::time::Duration;
use log::info;

const SSID: &str = "";
const PASSWORD: &str = "";

struct Rgb {
  r: u8,
  g: u8,
  b: u8,
}

impl Rgb {
  fn from_hsv(h: u32, s: u32, v: u32) -> Result<Self> {
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

fn neopixel(rgb: Rgb, tx: &mut TxRmtDriver) -> Result<()> {
  let color: u32 = ((rgb.r as u32) << 16) | ((rgb.g as u32) << 8) | (rgb.b as u32); 
  let ticks_hz = tx.counter_clock()?;
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
  tx.start_blocking(&signal)?;
  Ok(())
}

fn main() -> Result<()> {
  esp_idf_svc::sys::link_patches();

  // Bind the log crate to the ESP Logging facilities
  EspLogger::initialize_default();
  let peripherals = Peripherals::take()?;
  let sys_loop = EspSystemEventLoop::take()?;
  let nvs = EspDefaultNvsPartition::take()?;

  let mut wifi = BlockingWifi::wrap(
    EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
    sys_loop,
  )?;

  connect_wifi(&mut wifi)?;

  let mut button = PinDriver::input(peripherals.pins.gpio4)?;
  button.set_pull(Pull::Up)?;

  let mut tx = TxRmtDriver::new(peripherals.rmt.channel0, peripherals.pins.gpio5, &TransmitConfig::new().clock_divider(1))?;
  let mut i: u8 = 0;

  loop {
    std::thread::sleep(Duration::from_millis(100));
    // Using lc in this while loop is a hack to debounce the button.
    // Seems bad, but I don't know enough about embedded programming to
    // know the "right" way to do it.
    let mut lc: u8 = 0;
    while button.is_low() {
      if lc == 0 {
        neopixel(Rgb::from_hsv(i as u32 *100, 100, 20)?, &mut tx)?;
        i = set_light(i)?;
      }
      lc = 1;
    } 
    if button.is_high() {
      neopixel(Rgb::from_hsv(0, 0, 0)?, &mut tx)?;
    }
  }

  #[allow(unreachable_code)]
  Ok(()) 
}

fn connect_wifi(wifi: &mut BlockingWifi<EspWifi>) -> Result<()> {
  let wifi_config: Configuration = Configuration::Client(ClientConfiguration {
    ssid: SSID.into(),
    bssid: None,
    auth_method: AuthMethod::WPA2Personal,
    password: PASSWORD.into(),
    channel: None,
  });

  wifi.set_configuration(&wifi_config)?;
  wifi.start()?;
  info!("Wifi started!");
  wifi.connect()?;
  info!("Wifi connected!");
  wifi.wait_netif_up()?;
  info!("Wifi netif up!");

  Ok(())
}

fn set_light(scene: u8) -> Result<u8> {
  let socket = UdpSocket::bind("0.0.0.0:0")?;
  let msg =  match scene {
    0 => format!(r#"{{"method":"setPilot","params":{{"sceneId":{},"dimming":100}}}}"#, 12),
    _ => format!(r#"{{"method":"setPilot","params":{{"sceneId":{},"dimming":10}}}}"#, 6),
  };
  let light_addr = "192.168.4.145:38899";
  //let light_addr = "192.168.4.112:5514";
  socket.send_to(msg.as_bytes(), light_addr)?;
  info!("Light set to scene {}", scene);
  Ok((scene + 1) % 2)
}
