#![deny(unsafe_code)]

use embedded_hal::blocking::i2c;
use core::marker::PhantomData;

/// ICM42670P device driver, represented by a struct with 2 fields.
/// Datasheet: ..\..\datasheets\DS-000451-ICM-42670-P-v1.0.pdf
#[derive(Debug)]
pub struct ICM42670P<I2C> {
    // The concrete I²C device implementation.
    i2c: I2C,
    // Device address
    address: DeviceAddr,
    // remove the following line as soon as the I2C parameter is used. 
    // rec_type: PhantomData<I2C>,
}

// See Section 3.3.2, Table 4 in Documentation
/// Contains the possible variants of the devices addesses as binary numbers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceAddr {
    AD0 = 0b1101000, // or 0x68
    AD1 = 0b1101001, // or 0x69
}

// impl block with methods
impl<I2C, E>ICM42670P<I2C>
where
    // this defines which error messages will be used
    I2C: i2c::WriteRead<Error = E> + i2c::Write<Error = E>,
{
    /// Creates a new instance of the sensor, taking ownership of the i2c peripheral.
    pub fn new(i2c: I2C, address: DeviceAddr) -> Result<Self, E> {
        // instantiates the ICM42670P struct 
        // return the struct as an Ok value
        Ok(Self { i2c, address })
    }

    /// Returns the device's ID `0x67 
    //(if it doesn't, something is amiss)
    // Public method that can be accessed from outside this file.
    pub fn read_device_id_register(&mut self) -> Result<u8, E> {
        // reads the Device ID register
        self.read_register(Register::WhoAmI)
    }

    /// Writes into a register
    // This method is not public as it is only needed inside this file.
    #[allow(unused)]
    fn write_register(&mut self, register: Register, value: u8) -> Result<(), E> {
        // value that will be written as u8
        // i2c write 
        let byte = value;
        self.i2c
            .write(self.address as u8, &[register.address(), byte])
    }

    /// Reads a register using a `write_read` method.
    // This method is not public as it is only needed inside this file.
    fn read_register(&mut self, register: Register) -> Result<u8, E> {
        // buffer for values
        // i2c write_read
        // return u8 from le bytes
        let mut data = [0];
        self.i2c
            .write_read(self.address as u8, &[register.address()], & mut data)?;
        Ok(u8::from_le_bytes(data))
    }
}

// See Table 14.1 in documentation
/// This enum represents the device's registers
#[derive(Clone, Copy)]
pub enum Register {
    MclkRdy = 0x00,
    WhoAmI = 0x75,
    IntConfig = 0x06,
    TempData1 = 0x09,
    TempData0 = 0x0A,
    AccelDataX1 = 0x0B,
    AccelDataX0 = 0x0C,
    AccelDataY1 = 0x0D,
    AccelDataY0 = 0x0E,
    AccelDataZ1 = 0x0F,
    AccelDataZ0 = 0x10,
    GyroDataX1 = 0x11,
    GyroDataX0 = 0x12,
    GyroDataY1 = 0x13,
    GyroDataY0 = 0x14,
    GyroDataZ1 = 0x15,
    GyroDataZ0 = 0x16,
}

impl Register {
    fn address(&self) -> u8 {
        // Returns Register as u8
        *self as u8
    }
}
