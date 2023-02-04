use anyhow;
use embedded_hal::blocking::delay::DelayMs;
use esp_idf_hal::{
    delay::FreeRtos,
    i2c::{config::MasterConfig, Master, MasterPins, I2C0},
    peripherals::Peripherals,
    prelude::*,
};
use esp_idf_sys::*;
use shtcx::{self, PowerMode};

// goals of this exercise:
// instantiate i2c peripheral
// implement one sensor, print sensor values
// implement second sensor on same bus to solve an ownership problem

fn main() -> anyhow::Result<()>  {
    let peripherals = Peripherals::take().unwrap();

    // Instanciate the i2c peripheral, correct pins are in the training material.
    let sda = peripherals.pins.gpio10;
    let scl = peripherals.pins.gpio8; 

    let i2c = Master::<I2C0, _, _>::new(
        peripherals.i2c0,
        MasterPins { sda, scl },
        <MasterConfig as Default>::default().baudrate(400.kHz().into()),
    )?;
   
    // Create an instance of the SHTC3 sensor, find help in the documentation.
    let mut sht = shtcx::shtc3(i2c);

    // Read and print the sensor's device ID, find the methods in the documentation.
    let device_id = sht.device_identifier().unwrap;
    println!("SHTC3 T&H Device ID: {}", device_id);

    loop {
        // This loop initiates measurements, reads values and prints humidity in % and Temperature in °C.
        sht.
        FreeRtos.delay_ms(500u32);
    }
}

