#[allow(unused_imports)]
use log::error;
#[allow(unused_imports)]
use log::info;
#[allow(unused_imports)]
use log::warn;

use esp_idf_sys as _;

use esp_idf_hal::delay::Ets;
use esp_idf_hal::spi::SpiDeviceDriver;
use esp_idf_hal::prelude::FromValueType;
use esp_idf_hal::gpio::AnyOutputPin;
use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::gpio::PinDriver;

use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Point;

const MACHINE_NAME: &str = "orc";

//
fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    info!("machine: {MACHINE_NAME:?} -> rust_esp32_std_spi_ili");

    let peripherals = esp_idf_hal::peripherals::Peripherals::take().unwrap();

    warn!("### SPI peripherals");
    let spi = peripherals.spi2; // spi1 doest not impl SpiAnyPins
        
    warn!("### pins");
    // DO NOT USE gpio18 and gpio19 -> will BLOCK USB
    //
    let pin_sclk = peripherals.pins.gpio0; // SCLK
    
    let pin_sdo = peripherals.pins.gpio1; // SDO/MISO
    let _pin_sdi = peripherals.pins.gpio2; // SDI/MOSI
    
    let pin_cs = peripherals.pins.gpio9; // CS
    
    let pin_dc = peripherals.pins.gpio3; // DC
    warn!("### PinDriver DC: gpio3");
    let dc = PinDriver::output(pin_dc)?;
    
    let pin_rst = peripherals.pins.gpio4; // RST
    
    let pin_led = peripherals.pins.gpio5; // LED - BACKLIGHT
    warn!("### PinDriver BACKLIGHT: gpio5");
    let mut backlight = PinDriver::output(pin_led)?;
    warn!("### PinDriver BACKLIGHT set_high()");
    backlight
        .set_high()?;
    
    let spi_driver_config = esp_idf_hal::spi::config::DriverConfig::new()
        .dma(esp_idf_hal::spi::Dma::Disabled);
    
    let spi_config = esp_idf_hal::spi::config::Config::new()
        .baudrate(
            26u32.MHz().into()
        );
    
    if let Ok(spi_device_driver) = SpiDeviceDriver::new_single(
        //spi: impl Peripheral<P = SPI> + 'd,
        spi,
        
        //sclk: impl Peripheral<P = impl OutputPin> + 'd,
        pin_sclk,
        
        //sdo: impl Peripheral<P = impl OutputPin> + 'd,
        pin_sdo,
        
        //sdi: Option<impl Peripheral<P = impl InputPin + OutputPin> + 'd>,
        //Option::<AnyIOPin>::Some(pin_sdi.into()),
        Option::<AnyIOPin>::None,
        
        //cs: Option<impl Peripheral<P = impl OutputPin> + 'd>,
        Option::<AnyOutputPin>::Some(pin_cs.into()),
        //Option::<AnyOutputPin>::None,
        
        //bus_config: &DriverConfig,
        &spi_driver_config,
        
        //config: &Config
        &spi_config,
    ) {
        warn!("### SPI SpiDeviceDriver.new_single() OK");
        
        let di = display_interface_spi::SPIInterfaceNoCS::new(
            // SPI: Write<u8> 
            spi_device_driver,
            // DC: OutputPin
            dc,
        );
        
        // MIPIDSI
        warn!("### PinDriver RST: gpio4");
        let mut rst = esp_idf_hal::gpio::PinDriver::output(pin_rst)?;
        warn!("### PinDriver RST set_high()");
        rst.set_high()?;

        let mut delay_spi = Ets {};
        
        warn!("### MIPI init");
        let mut display_spi = mipidsi::Builder::ili9341_rgb666(di)
            .init(
                // DELAY: DelayUs<u32>
                &mut delay_spi,
                // RST: OutputPin,
                Some(rst),
            )
            .map_err(|e| anyhow::anyhow!("mipidsi.init() ERROR: {:?}", e))?;
        
        // CLEAR
        warn!("### display_spi: clear");
        if let Err(e) = display_spi
            .clear(embedded_graphics::pixelcolor::RgbColor::RED) {
                error!("display_spi.clear() ERROR: {e:?}")
            }

        // DRAW
        warn!("### display_spi: draw");
        if let Err(e) = Text::new(
            //msg,
            &format!("machine: {}", MACHINE_NAME),
            //point,
            Point::new(10, 10),
            MonoTextStyleBuilder::new()
                .font(&FONT_6X10)
                .text_color(embedded_graphics::pixelcolor::RgbColor::YELLOW)
                .background_color(embedded_graphics::pixelcolor::RgbColor::GREEN)
                .build(),
        ).draw(&mut display_spi) {
            error!("text.draw() ERROR: {e:?}")
        }
    }

    Ok(())
}
