use anyhow;
#[allow(unused_imports)]
use core::fmt::Write;
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
// use imc42670p; 
#[allow(unused_imports)]
use icm42670::{prelude::*, Address, Icm42670}; // driver for the ICM-42670 6-axis IMU from InvenSense
use icm42670::accelerometer::vector::VectorExt;
#[allow(unused_imports)]
use shtcx; // driver for the Sensirion SHTCx T & H sensor series

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

struct ImuConfig {
    device_id: u8,
    power_mode: icm42670::PowerMode,
    gyro_range: icm42670::GyroRange,
    accel_range: icm42670::AccelRange,
}

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
    let sht_device_id = sht.device_identifier().unwrap();
    println!("SHTC3 T&H Device ID: {}", sht_device_id);

    // Create an instance of the IMU, find help via 'cargo doc --open'
    let mut imu = Icm42670::new(i2c_bus_seat2, Address::Primary).unwrap();
    imu.device_id().unwrap(); // Test bus config (ignore return value)

    // Set IMU Config
    imu.set_power_mode(icm42670::PowerMode::SixAxisLowNoise).unwrap();
    imu.set_accel_range(icm42670::AccelRange::G16).unwrap();
    imu.set_gyro_range(icm42670::GyroRange::Deg2000).unwrap();

    // Read IMU Config
    let imu_cfg = ImuConfig {
        device_id: imu.device_id().unwrap(),
        power_mode: imu.power_mode().unwrap(),
        gyro_range: imu.gyro_range().unwrap(),
        accel_range: imu.accel_range().unwrap(),    
    };

    // Print IMU Config
    println!("ICM42670 IMU:");
    println!("  {} (Device ID)", imu_cfg.device_id);
    println!("  {:?} (PowerMode)", imu_cfg.power_mode);
    println!("  {:?} (GyroRange)", imu_cfg.gyro_range);
    println!("  {:?} (AccelRange)", imu_cfg.accel_range);

    loop {
        // This loop initiates measurements, reads values and prints humidity in % and Temperature in °C.
        sht.start_measurement(shtcx::PowerMode::NormalMode).unwrap();
        FreeRtos.delay_ms(100u32);
        let th_meas = sht.get_measurement_result().unwrap(); 

        let accel_meas = imu.accel_norm().unwrap();
        let accel_mag = accel_meas.magnitude();
        let gyro_meas = imu.gyro_norm().unwrap();

        println!("TEMP: {:.2} °C", th_meas.temperature.as_degrees_celsius());
        println!("HUM: {:.2} %", th_meas.humidity.as_percent());
        print!("ACC:  X: {:.2}, Y: {:.2}, Z: {:.2}", accel_meas.x, accel_meas.y, accel_meas.z);
        if accel_mag < 0.95 || accel_mag > 1.05 {
            println!(", Mag: {:.2} g's", accel_mag);
        } else {
            println!(" g's");
        }
        println!("GYRO: X: {:.2}, Y: {:.2}, Z: {:.2} °/s", gyro_meas.x, gyro_meas.y, gyro_meas.z);
        println!(" ");

        FreeRtos.delay_ms(500u32);
    }
}

