use bsc::{
    led::{RGB8, WS2812RMT},
    temp_sensor::BoardTempSensor,
    wifi::wifi,
};
use embedded_svc::mqtt::client::{
    Client,
    Details::{Complete, InitialChunk, SubsequentChunk},
    Event::{self, Received},
    Message, Publish, QoS,
};
use esp32_c3_dkc02_bsc as bsc;
use esp_idf_svc::{
    log::EspLogger,
    mqtt::client::{EspMqttClient, EspMqttMessage, MqttClientConfiguration},
};
use std::{borrow::Cow, convert::TryFrom, thread::sleep, time::Duration};
// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_sys as _;
use log::{error, info};

// imported message topics
use mqtt_messages::{
    hello_topic,
    temperature_data_topic,
    Command,
    RawCommandData,
    cmd_topic_fragment,
};

const UUID: &'static str = get_uuid::uuid();

#[toml_cfg::toml_config]
pub struct Config {
    #[default("localhost")]
    mqtt_host: &'static str,
    #[default("")]
    mqtt_user: &'static str,
    #[default("")]
    mqtt_pass: &'static str,
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

fn main() -> anyhow::Result<()> {

    // Setup 
    esp_idf_sys::link_patches();

    EspLogger::initialize_default();

    let app_config = CONFIG;

    info!("our UUID is:");
    info!("{}", UUID);

    let mut temp_sensor = BoardTempSensor::new_taking_peripherals();

    let mut led = WS2812RMT::new()?;
    led.set_pixel(RGB8::new(1, 1, 0))?;

    let _wifi = wifi(app_config.wifi_ssid, app_config.wifi_psk)?;

    // Client configuration:
    let broker_url = if app_config.mqtt_user != "" {
        format!(
            "mqtt://{}:{}@{}",
            app_config.mqtt_user, app_config.mqtt_pass, app_config.mqtt_host
        )
    } else {
        format!("mqtt://{}", app_config.mqtt_host)
    };

    let mqtt_config = MqttClientConfiguration::default();

    // Your Code:
    // 1. Create a client with default configuration and empty handler
    let mut client = EspMqttClient::new(
        broker_url, 
        &mqtt_config,
        |_|{}
    )?;

    // 2. publish an empty hello message
    client.publish(
        hello_topic(UUID),
        QoS::AtLeastOnce,
        false,
        Vec::new(),
    )?;

    loop {
        sleep(Duration::from_secs(1));
        let temp = temp_sensor.read_owning_peripherals();
        let temp_msg = temperature(temp);
        // 3. publish CPU temperature
        client.publish(
            temperature_data_topic(UUID),
            QoS::AtLeastOnce,
            false,
            temp_msg.chars().map(|c|c as u8).collect::<Vec<u8>>(),
        )?;
    }
}

fn templated(content: impl AsRef<str>) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8">
        <title>esp-rs web server</title>
    </head>
    <body>
        {}
    </body>
</html>
"#,
        content.as_ref()
    )
}

fn index_html() -> String {
    templated("Hello from mcu!")
}

fn temperature(val: f32) -> String {
    templated(format!("chip temperature: {:.2}°C", val))
}
