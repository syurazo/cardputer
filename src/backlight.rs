//! LCD backlight controller
use anyhow::Result;
use esp_idf_hal::{
    gpio::{Gpio38, Level, Output, PinDriver},
    peripheral::Peripheral,
};

/// Backlight controller
///
/// # Examples
///
/// ```
/// use cardputer::backlight::Backlight;
///
/// let peripherals = Peripherals::take().unwrap();
///
/// let mut backlight = Backlight::new(peripherals.pins.gpio38).unwrap();
/// backlight.on().unwrap();
/// ```
pub struct Backlight<'a> {
    driver: PinDriver<'a, Gpio38, Output>,
}

impl<'a> Backlight<'a> {
    /// Create new controller.
    pub fn new(gpio: impl Peripheral<P = Gpio38> + 'a) -> Result<Backlight<'a>> {
        let driver = PinDriver::output(gpio)?;

        Ok(Self { driver })
    }

    /// Turn on the backlight.
    pub fn on(&mut self) -> Result<()> {
        self.driver.set_level(Level::High)?;
        Ok(())
    }

    /// Turn off the backlight.
    pub fn off(&mut self) -> Result<()> {
        self.driver.set_level(Level::Low)?;
        Ok(())
    }
}
