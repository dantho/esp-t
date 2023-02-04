use anyhow;
use embedded_hal::blocking::delay::DelayMs;
#[allow(unused_imports)]
use esp_idf_hal::{
    delay::FreeRtos,
    i2c::{config::MasterConfig, Master, MasterPins, I2C0},
    peripherals::Peripherals,
    prelude::*,
    mutex::Mutex,
};
#[allow(unused_imports)]
use esp_idf_sys::*;
#[allow(unused_imports)]
use shtcx::{self, PowerMode}; // driver for the Sensirion SHTCx T & H sensor series
// use imc42670p; 
#[allow(unused_imports)]
use icm42670::{prelude::*, Address, Icm42670}; // driver for the ICM-42670 6-axis IMU from InvenSense

#[allow(unused_imports)]
use core::fmt::Write;

// // From icm42670 crate's example code, not training:
// use esp32c3_hal::{
//     clock::ClockControl,
//     gpio::IO,
//     i2c::I2C,
//     pac::Peripherals,
//     prelude::*,
//     timer::TimerGroup,
//     Delay,
//     Rtc,
//     UsbSerialJtag,
// };

// goals of part 1:
// instantiate i2c peripheral
// implement one T&H sensor, print sensor values
// goals of part 2:
// implement IMU sensor on same bus to solve an ownership problem

fn main() -> anyhow::Result<()>  {
    let peripherals = Peripherals::take().unwrap();

    // Instanciate the i2c peripheral, correct pins are in the training material.
    let sda = peripherals.pins.gpio10;
    let scl = peripherals.pins.gpio8; 

    let i2c_singleton = Master::<I2C0, _, _>::new(
        peripherals.i2c0,
        MasterPins { sda, scl },
        <MasterConfig as Default>::default().baudrate(400.kHz().into()),
    )?;

    // Instantiate the bus manager, pass the i2c bus. 
    let i2c_bus = shared_bus::BusManagerSimple::new(i2c_singleton);

    // Create two proxies. Now, each sensor can have their own instance of a proxy i2c, which resolves the ownership problem. 
    let i2c_bus_seat1 =i2c_bus.acquire_i2c(); // a proxy for the i2c bus
    let i2c_bus_seat2 =i2c_bus.acquire_i2c(); // a 2nd proxy for the same i2c bus

    // Create an instance of the SHTC3 sensor, find help in the documentation.
    let mut sht = shtcx::shtc3(i2c_bus_seat1);
    let device_id = sht.device_identifier().unwrap();
    println!("SHTC3 T&H Device ID: {}", device_id);

    // Create an instance of the IMU, find help via 'cargo doc --open'
    let mut imu = Icm42670::new(i2c_bus_seat2, Address::Primary).unwrap();
    let device_id = imu.device_id().unwrap();
    println!("ICM42670 IMU Device ID: {}", device_id);

    loop {
        // This loop initiates measurements, reads values and prints humidity in % and Temperature in °C.
        sht.start_measurement(PowerMode::NormalMode).unwrap();
        FreeRtos.delay_ms(100u32);
        let th_meas = sht.get_measurement_result().unwrap(); 

        let acc_meas = imu.accel_norm().unwrap();
        let gyro_meas = imu.gyro_norm().unwrap();

        println!("TEMP: {:.2} °C", th_meas.temperature.as_degrees_celsius());
        println!("HUM: {:.2} %", th_meas.humidity.as_percent());
        println!("ACC:  X: {:.3}, Y: {:.3}, Z: {:.3} g's", acc_meas.x, acc_meas.y, acc_meas.z);
        println!("GYRO: X: {:.2}, Y: {:.2}, Z: {:.2} °/s ?", gyro_meas.x, gyro_meas.y, gyro_meas.z);
        println!(" ");

        FreeRtos.delay_ms(500u32);
    }
}

