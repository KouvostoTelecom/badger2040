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
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, StrokeAlignment::Outside},
    text::{Alignment, Text},
    image::Image,
};
// endregion

// region: embedded_graphics extensions
use tinybmp::Bmp;
use badger2040::graphics_extensions::Centering;
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

    let screen_center = Point::new((uc8151::WIDTH / 2) as i32, (uc8151::HEIGHT / 2) as i32);
    
    let text = Text::with_alignment(
        "Heikki\n'Zokol'\nJuva",
        Point::new(0, 0),
        style_black,
        Alignment::Center,
    );

    let avatar_bmp = include_bytes!("../gfx/zokol2.bmp");

    let avatar = Bmp::from_slice(avatar_bmp).unwrap();

    let koteco_bmp = include_bytes!("../gfx/koteco_logo.bmp");

    let koteco = Bmp::from_slice(koteco_bmp).unwrap();

    let text = text.center(screen_center);

    Image::new(&avatar, Point::new(0, 0)).draw(&mut display).unwrap();
    Image::new(&koteco, Point::new(175, 0)).draw(&mut display).unwrap();

    text.bounding_box()
        .into_styled(box_style)
        .draw(&mut display)
        .unwrap();
    text.draw(&mut display).unwrap();

    led.set_low().unwrap();

    for i in 0..40{
        display.update().unwrap();
    }
    
    loop {
        led.set_high().unwrap();
    }
}
