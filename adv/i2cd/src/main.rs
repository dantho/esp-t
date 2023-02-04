use anyhow;
use embedded_hal::blocking::delay::DelayMs;
use esp_idf_hal::{
    delay::FreeRtos,
    i2c::{config::MasterConfig, Master, MasterPins, I2C0},
    peripherals::Peripherals,
    prelude::*,
};
// use esp_idf_sys::*;

// uncomment the following line to run the solution, check lib.rs for further instructions
// use i2c_driver_exercise::icm42670p_solution::{DeviceAddr, ICM42670P};

// comment out the following line to run the exercise, check lib.rs for further instructions
use i2c_driver_exercise::icm42670p::{DeviceAddr, ICM42670P};

// Dont change this file. Work in the icm42670p.rs and modify it so main.rs runs.

fn main() -> anyhow::Result<()> {
    let peripherals = Peripherals::take().unwrap();

    let sda = peripherals.pins.gpio10;
    let scl = peripherals.pins.gpio8;

    let i2c = Master::<I2C0, _, _>::new(
        peripherals.i2c0,
        MasterPins { sda, scl },
        <MasterConfig as Default>::default().baudrate(400.kHz().into()),
    )?;

    let mut sensor = ICM42670P::new(i2c, DeviceAddr::AD0)?;
    println!("Sensor init");
    let device_id = sensor.read_device_id_register()?;
    println!("Hello, world, I am sensor {} (0x{:02x})", device_id, device_id);
    assert_eq!(device_id, 103_u8);

    loop {
        FreeRtos.delay_ms(500u32);
    }
}
