use bsc::{
    led::{RGB8, WS2812RMT},
    temp_sensor::BoardTempSensor,
    wifi::wifi,
};
use embedded_svc::mqtt::client::{
    Client,
    Details::{Complete, InitialChunk, SubsequentChunk},
    Event::{self, Received, Published},
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
use log::{error, info, warn};

// imported message topics
use mqtt_messages::{
    hello_topic,
    color_topic,
    temperature_data_topic,
    Command,
    RawCommandData,
    cmd_topic_fragment,
    ColorData,
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

    dbg!(&broker_url);
    dbg!(&broker_url);
    dbg!(&broker_url);
    dbg!(&broker_url);
    
    let mqtt_config = MqttClientConfiguration::default();

    // Your Code:
    // 1. Create a client with default configuration and empty handler
    // Part2: Modify to handle color topic subscription
    let mut client = EspMqttClient::new(
        broker_url, 
        &mqtt_config,
        move |message_event| match message_event {
            Ok(Received(msg)) => process_message(msg, &mut led),
            Ok(Published(msg_id)) => (),
            _ => warn!("Received from MQTT: {:?}", message_event),
        }
    )?;

    info!(">>> EspMqttClient instantiated <<<");

    // 2. publish an empty hello message
    let publish_topic = hello_topic(UUID);
    let empty_payload: &[u8] = &[];
    client.publish(
        publish_topic,
        QoS::AtLeastOnce,
        false,
        empty_payload,
    )?;

    info!(">>> Published hello topic <<<");

    let empty_cmd = Command::BoardLed(RGB8::new(0,0,0));
    client.subscribe(empty_cmd.topic(UUID), QoS::AtLeastOnce);

    info!(">>> Subscribed to all commands <<<");

    loop {
        sleep(Duration::from_secs(1));
        // temperature
        let temp = temp_sensor.read_owning_peripherals();
        let temperature_data = &temp.to_be_bytes() as &[u8];
        // 3. publish CPU temperature
        client.publish(
            temperature_data_topic(UUID),
            QoS::AtLeastOnce,
            false,
            temperature_data,
        )?;
    }
}

fn process_message(message: &EspMqttMessage, led: &mut WS2812RMT) {
    match message.details() {
        // all messages in this exercise will be of type `Complete`
        // the other variants of the `Details` enum are for larger message payloads
        Complete => {
            // Cow<&[u8]> can be coerced into a slice &[u8] or a Vec<u8>
            // You can coerce it into a slice to be sent to try_from()
            // RGB LED command
            let is_command_topic = message.topic().unwrap().split("/").nth(1) == Some("command");
            if is_command_topic {
                let raw = RawCommandData {
                    path: "",
                    data: message.data(),
                };
                if let Ok(Command::BoardLed(color)) = Command::try_from(raw) {
                    // set the LED to the newly received color
                    led.set_pixel(color);
                    info!("Setting LED to {:?}", color);
                }
            } else {
                let message_data: &[u8] = &message.data();
                if let Ok(ColorData::BoardLed(color)) = ColorData::try_from(message_data) {
                    // set the LED to the newly received color
                    led.set_pixel(color);
                    info!("Setting LED to {:?}", color);
                }
            }
        },
        bad => warn!("Unexpected message type from MQTT: {:?}", bad),
    }    
}

