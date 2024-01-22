//! Initialize I2C driver for Grove I/F
use anyhow::Result;
use esp_idf_hal::{
    gpio::{Gpio1, Gpio2},
    i2c::I2C0,
    i2c::{I2cConfig, I2cDriver},
    peripheral::Peripheral,
    units::Hertz,
};

pub fn build<'a>(
    i2c: impl Peripheral<P = I2C0> + 'a,
    sda: impl Peripheral<P = Gpio2> + 'a,
    scl: impl Peripheral<P = Gpio1> + 'a,
    hz: Hertz,
) -> Result<I2cDriver<'a>> {
    let config = I2cConfig::new().baudrate(hz);
    let i2c = I2cDriver::new(i2c, sda, scl, &config)?;

    Ok(i2c)
}
