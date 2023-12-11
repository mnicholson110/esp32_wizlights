use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use std::net::UdpSocket;
use log::info;

const SSID: &str = "eventually I'm probably going to push my wifi info to a file and gitignore it";
const PASSWORD: &str = "but not today";

fn main() -> anyhow::Result<()> {
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

  let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

  info!("Wifi DHCP info: {:?}", ip_info);

  let socket = UdpSocket::bind("0.0.0.0:0")?;
  let msg_on = r#"{"method":"setPilot","params":{"sceneId":12}}"#.to_string();
  let msg_party = r#"{"method":"setPilot","params":{"sceneId":4}}"#.to_string();
  let light_addr = "192.168.4.145:38899";

  
  loop {
    // info!("Sleep!");

    // set light, wait 5 seconds, set light
    socket.send_to(msg_on.as_bytes(), light_addr)?;
    info!("Sent on!");
    std::thread::sleep(std::time::Duration::from_secs(5));
    socket.send_to(msg_party.as_bytes(), light_addr)?; 
    info!("Sent party!");
    std::thread::sleep(std::time::Duration::from_secs(5));
  }


  #[allow(unreachable_code)]
  Ok(()) 

}

fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<()> {
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
