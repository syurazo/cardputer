//! Create and initialize ST7789 display driver
use anyhow::{anyhow, Result};
use display_interface_spi::SPIInterfaceNoCS;
use esp_idf_hal::{
    delay::Delay,
    gpio::{AnyIOPin, Output, PinDriver},
    gpio::{Gpio33, Gpio34, Gpio35, Gpio36, Gpio37},
    peripheral::Peripheral,
    prelude::*,
    spi::{config::DriverConfig, SpiAnyPins, SpiConfig, SpiDeviceDriver, SpiDriver},
};
use mipidsi::{models::ST7789, options::Orientation, Builder, ColorInversion, Display};

type Drawable<'a> = Display<
    SPIInterfaceNoCS<SpiDeviceDriver<'a, SpiDriver<'a>>, PinDriver<'a, Gpio34, Output>>,
    ST7789,
    PinDriver<'a, Gpio33, Output>,
>;

/// Display width
pub const DISPLAY_SIZE_WIDTH: u16 = 240;
/// Display height
pub const DISPLAY_SIZE_HEIGHT: u16 = 135;

/// Create and initialize display driver
///
/// # Examples
///
/// ```
/// use embedded_graphics::pixelcolor::Rgb565;
/// use cardputer::display;
///
/// let peripherals = Peripherals::take().unwrap();
///
/// let mut display = display::build(
///     peripherals.spi2,
///     peripherals.pins.gpio36,
///     peripherals.pins.gpio35,
///     peripherals.pins.gpio37,
///     peripherals.pins.gpio34,
///     peripherals.pins.gpio33,
/// )
/// .unwrap();
/// display.clear(Rgb565::WHITE).unwrap();
/// ```
pub fn build<'a, SPI>(
    spi: impl Peripheral<P = SPI> + 'a,
    sck: impl Peripheral<P = Gpio36> + 'a,
    dc: impl Peripheral<P = Gpio35> + 'a,
    cs: impl Peripheral<P = Gpio37> + 'a,
    rs: impl Peripheral<P = Gpio34> + 'a,
    rst: impl Peripheral<P = Gpio33> + 'a,
) -> Result<Drawable<'a>>
where
    SPI: SpiAnyPins,
{
    let spi_config = SpiConfig::new().baudrate(80.MHz().into());
    let device_config = DriverConfig::new();
    let spi = SpiDeviceDriver::new_single(
        spi,
        sck,
        dc,
        Option::<AnyIOPin>::None,
        Some(cs),
        &device_config,
        &spi_config,
    )?;

    let rs = PinDriver::output(rs)?;
    let rst = PinDriver::output(rst)?;
    let mut drawable = Builder::st7789(SPIInterfaceNoCS::new(spi, rs))
        .with_invert_colors(ColorInversion::Inverted)
        .with_display_size(DISPLAY_SIZE_WIDTH, DISPLAY_SIZE_HEIGHT)
        .with_window_offset_handler(|_| (41, 53))
        .init(&mut Delay::new_default(), Some(rst))
        .map_err(|e| anyhow!("{:?}", e))?;

    drawable
        .set_orientation(Orientation::Landscape(true))
        .map_err(|e| anyhow!("{:?}", e))?;
    drawable
        .set_scroll_offset(0)
        .map_err(|e| anyhow!("{:?}", e))?;

    Ok(drawable)
}
