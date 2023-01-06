//! # GPIO 'Blinky' Example
//!
//! This application demonstrates how to control a GPIO pin on the RP2040.
//!
//! It may need to be adapted to your particular board layout and/or pin assignment.
//!
//! See the `Cargo.toml` file for Copyright and license details.

#![no_std]
#![no_main]

use fugit::RateExtU32;

use hal::gpio::{Floating, Function, Input, Output, Pin, PushPull, Spi};
use pimoroni_badger2040::entry;
// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// Alias for our HAL crate
use pimoroni_badger2040::hal;

// A shorter alias for the Peripheral Access Crate, which provides low-level
// register access
use pimoroni_badger2040::hal::pac;
use pimoroni_badger2040::hal::Timer;

// Some traits we need
use embedded_hal::digital::v2::OutputPin;
use rp2040_hal::clocks::Clock;

use uc8151::Uc8151;

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
/// Note: This boot block is not necessary when using a rp-hal based BSP
/// as the BSPs already perform this step.
//#[link_section = ".boot2"]
//#[used]
//pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

/// External high-speed crystal on the Raspberry Pi Pico board is 12 MHz. Adjust
/// if your board has a different frequency
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::{ascii::FONT_10X20, ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Text},
};

#[rustfmt::skip]
const DATA: &[u8] = &[
    0b11001011, 0b1110_0000,
    0b10101010, 0b0100_0000,
    0b10101011, 0b0100_0000,
    0b10101001, 0b0100_0000,
    0b11001011, 0b0100_0000,
];

#[entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
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

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = pimoroni_badger2040::Pins::new(
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
        50_000_000u32.Hz(),
        &embedded_hal::spi::MODE_0,
    );

    let dc_pin: Pin<_, Output<PushPull>> = pins.inky_dc.into_mode();
    let reset_pin: Pin<_, Output<PushPull>> = pins.inky_res.into_mode();
    let busy_pin: Pin<_, Input<Floating>> = pins.inky_busy.into_mode();

    let mut display = Uc8151::new(spi, spi_cs, dc_pin, busy_pin, reset_pin);

    display.enable();
    display.setup(&mut delay, uc8151::LUT::Fast).unwrap();

    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::Off);

    Text::with_alignment(
        "KYBERHAX0R 3000",
        Point::new((uc8151::WIDTH / 2) as i32, (uc8151::HEIGHT / 2) as i32),
        style,
        Alignment::Center,
    )
    .draw(&mut display)
    .unwrap();

    // Configure GPIO25 as an output
    let mut led_pin = pins.led.into_push_pull_output();
    led_pin.set_high().unwrap();

    display.update().unwrap();
    led_pin.set_low().unwrap();

    loop {
        delay.delay_ms(20000);
    }
}

// End of file
