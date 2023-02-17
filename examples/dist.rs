//! # Rust Badge for badger2040

// region: imports and boilerplate
#![no_std]
#![no_main]

use embedded_graphics::pixelcolor;
use embedded_graphics::{image::Image, mono_font::iso_8859_15::FONT_6X13};
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
use tinybmp::Bmp;
use uc8151::Uc8151;

// Graphics library
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, StrokeAlignment::Outside},
    text::{Alignment, Text},
};
// endregion

// region: embedded_graphics extensions
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

    let mut led: Pin<_, Output<PushPull>> = pins.led.into_mode();
    led.set_high().unwrap();

    let mut display = Uc8151::new(spi, spi_cs, dc_pin, busy_pin, reset_pin);

    display.enable();
    display.setup(&mut delay, uc8151::LUT::Normal).unwrap();

    let style_fullname = MonoTextStyle::new(&FONT_6X13, BinaryColor::Off);
    let style_black = MonoTextStyle::new(&FONT_10X20, BinaryColor::Off);
    let style_white = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    let box_style = PrimitiveStyleBuilder::new()
        .stroke_color(BinaryColor::Off)
        .stroke_width(10)
        .stroke_alignment(Outside)
        .fill_color(BinaryColor::On)
        .build();

    let screen_center = Point::new((uc8151::WIDTH / 2) as i32, (uc8151::HEIGHT / 2) as i32);
    let split_at = uc8151::WIDTH / 3;

    for x in 0..split_at {
        for y in 0..uc8151::HEIGHT {
            display.pixel(x, y, true);
        }
    }

    // Include an image from a local path as bytes
    let data = include_bytes!("../gfx/dist_portrait2.bmp");
    let bmp = Bmp::from_slice(data).unwrap();
    let mut image = Image::new(&bmp, Point::zero());
    //image.center_mut(screen_center);
    image.draw(&mut display).unwrap();

    let data = include_bytes!("../gfx/dist_cubefade.bmp");
    let bmp = Bmp::from_slice(data).unwrap();
    let mut image = Image::new(&bmp, Point::new(split_at as i32, 0));
    //image.center_mut(screen_center);
    image.draw(&mut display).unwrap();

    let mut text = Text::with_alignment(
        "Taneli Kaivola",
        Point::new(0, 0),
        style_fullname,
        Alignment::Center,
    );
    text.center_mut(Point::new(
        screen_center.x / 2 * 3,
        screen_center.y - (text.bounding_box().bottom_right().unwrap().y as i32) * 6,
    ));

    text.draw(&mut display).unwrap();
    let mut text = Text::with_alignment("@dist", Point::new(0, 0), style_black, Alignment::Center);
    text.center_mut(Point::new(
        screen_center.x / 2 * 3,
        screen_center.y + (text.bounding_box().bottom_right().unwrap().y as i32) * 2,
    ));
    text.draw(&mut display).unwrap();

    display.update().unwrap();

    led.set_low().unwrap();
    loop {
        delay.delay_ms(1000)
    }
}
