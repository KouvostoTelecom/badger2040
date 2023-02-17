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
use embedded_graphics::primitives::Circle;
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

// region: trig
use libm::{cos, sin};
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
    display.setup(&mut delay, uc8151::LUT::Ultrafast).unwrap();

    let mut t = 0;
    loop {
        t += 1;
        for x in 0..uc8151::WIDTH {
            for y in 0..uc8151::HEIGHT {
                display.pixel(x, y, true);
            }
        }

        let r = 50;
        let x = sin(t as f64 / 11.0) * 100.0 + uc8151::WIDTH as f64 / 2.0 - r as f64 / 2.0;
        let y = cos(t as f64 / 13.0) * 50.0 + uc8151::HEIGHT as f64 / 2.0 - r as f64 / 2.0;
        let circle = Circle::new(Point::new(x as i32, y as i32), r).into_styled(
            PrimitiveStyleBuilder::new()
                .stroke_color(BinaryColor::On)
                .stroke_width(1)
                .stroke_alignment(Outside)
                .build(),
        );
        circle.draw(&mut display).unwrap();

        display.update().unwrap();
    }
}
