// Neopixel and Rgb code from github.com/esp-rs/esp-idf-hal/blob/master/examples/rmt_neopixel.rs

use anyhow::Result;
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use std::net::UdpSocket;
use core::time::Duration;
use log::info;
#[path = "utils/neopixel.rs"] mod neopixel;

const SSID: &str = "";
const PASSWORD: &str = "";

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

  let chan = peripherals.rmt.channel0;
  let gpio = peripherals.pins.gpio5;

  let mut np = neopixel::Neopixel::new(chan, gpio);

  np.set_color_hsv(0, 0, 100)?;
  std::thread::sleep(Duration::from_millis(500));
  np.set_color_hsv(0, 0, 0)?;
  std::thread::sleep(Duration::from_millis(500));
  np.set_color_hsv(0, 0, 100)?;
  std::thread::sleep(Duration::from_millis(500));
  let mut i: u8 = 0;

  loop {
    std::thread::sleep(Duration::from_millis(100));
    // Using lc in this while loop is a hack to debounce the button.
    // Seems bad, but I don't know enough about embedded programming to
    // know the "right" way to do it.
    let mut lc: u8 = 0;
    while button.is_low() {
      if lc == 0 {
        np.set_color_hsv(i as u32 *100, 100, 20)?;
        i = set_light(i)?;
      }
      lc = 1;
      std::thread::sleep(Duration::from_millis(10));
    } 
    if button.is_high() {
      np.set_color_hsv(0, 0, 0)?;
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
