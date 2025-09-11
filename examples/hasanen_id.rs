//! # Rust Badge for badger2040

// region: imports and boilerplate
#![no_std]
#![no_main]

// Halt if panic
use panic_halt as _;

// Required traits
use embedded_hal::digital::v2::OutputPin;
use fugit::RateExtU32;
use rp2040_hal::clocks::Clock;

// Hardware
use hal::gpio::{Floating, Function, Input, Output, Pin, PushPull, Spi};

use badger2040::bsp;

use bsp::entry;
use bsp::hal;
use bsp::hal::pac;
use uc8151::Uc8151;

// Graphics library
use embedded_graphics::{
    image::Image,
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, StrokeAlignment::Outside},
    text::{Alignment, Text},
};
// endregion

// region: embedded_graphics extensions
//use badger2040::graphics_extensions::Centering;
use tinybmp::Bmp;
// endregion

#[entry]
fn main() -> ! {
    // region: initialization
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let sio = hal::Sio::new(pac.SIO);

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let _sclk: Pin<_, Function<Spi>> = pins.sclk.into_mode();
    let _miso: Pin<_, Function<Spi>> = pins.miso.into_mode();
    let _mosi: Pin<_, Function<Spi>> = pins.mosi.into_mode();
    let spi_cs: Pin<_, Output<PushPull>> = pins.inky_cs_gpio.into_mode();

    let spi = hal::Spi::<_, _, 8>::new(pac.SPI0);

    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        10_000_000u32.Hz(),
        &embedded_hal::spi::MODE_0,
    );

    let dc_pin: Pin<_, Output<PushPull>> = pins.inky_dc.into_mode();
    let reset_pin: Pin<_, Output<PushPull>> = pins.inky_res.into_mode();
    let busy_pin: Pin<_, Input<Floating>> = pins.inky_busy.into_mode();

    // endregion

    let mut display = Uc8151::new(spi, spi_cs, dc_pin, busy_pin, reset_pin);

    display.enable();
    display.setup(&mut delay, uc8151::LUT::Normal).unwrap();

    let style_black = MonoTextStyle::new(&FONT_10X20, BinaryColor::Off);
    let box_style = PrimitiveStyleBuilder::new()
        .stroke_color(BinaryColor::On)
        .stroke_width(6)
        .stroke_alignment(Outside)
        .fill_color(BinaryColor::On)
        .build();

    let mut led: Pin<_, Output<PushPull>> = pins.led.into_mode();
    led.set_high().unwrap();

    // Text not totally centered so that KTK logo is "complete"
    let text_position = Point::new((uc8151::WIDTH / 3 * 2) as i32, 15);

    let text = Text::with_alignment("hasanen", text_position, style_black, Alignment::Center);

    // width: 102
    // height: 28
    let avatar_bmp = include_bytes!("../gfx/hasanen.bmp");
    let avatar = Bmp::from_slice(avatar_bmp).unwrap();

    // width: 80
    // height: 80
    let qr_hsm_bmp = include_bytes!("../gfx/qr_horseseamen.bmp");
    let qr_hsm = Bmp::from_slice(qr_hsm_bmp).unwrap();

    // width: 80
    // height: 80
    let qr_poc_bmp = include_bytes!("../gfx/qr_pieceofcodeblog.bmp");
    let qr_poc = Bmp::from_slice(qr_poc_bmp).unwrap();

    /*
    for i in 0..5{
        display.update().unwrap();
    }
    */

    Image::new(&avatar, Point::new(0, 0))
        .draw(&mut display)
        .unwrap();

    Image::new(&qr_hsm, Point::new(110, 33))
        .draw(&mut display)
        .unwrap();

    Image::new(&qr_poc, Point::new(201, 33))
        .draw(&mut display)
        .unwrap();

    text.bounding_box()
        .into_styled(box_style)
        .draw(&mut display)
        .unwrap();
    text.draw(&mut display).unwrap();

    display.update().unwrap();

    loop {
        led.set_low().unwrap();
        delay.delay_ms(1000);
        led.set_high().unwrap();
        delay.delay_ms(1000);
    }
}
