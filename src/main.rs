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

use rp2040_hal::gpio::FunctionSpi;
//use rp2040_hal::spi::Spi;

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
    pixelcolor::BinaryColor,
    prelude::*,
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
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let _sclk = pins.gpio18.into_mode::<FunctionSpi>(); // sclk
    let _miso = pins.gpio16.into_mode::<FunctionSpi>(); // miso
    let _mosi = pins.gpio19.into_mode::<FunctionSpi>(); // mosi
    let spi_cs = pins.gpio17.into_push_pull_output();

    let spi = hal::Spi::<_, _, 8>::new(pac.SPI0);

    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        2_500_000u32.Hz(),
        &embedded_hal::spi::MODE_1,
    );
 
    let mut dc_pin = pins.gpio20.into_push_pull_output();
    let mut busy_pin = pins.gpio26.into_bus_keep_input();
    let mut reset_pin = pins.gpio21.into_push_pull_output();

    let mut display = Uc8151::new(spi, spi_cs, dc_pin, busy_pin, reset_pin);
    
    display.enable();
    display.setup(&mut delay, uc8151::LUT::Normal);

    let raw_image = ImageRaw::<BinaryColor>::new(DATA, 12);
    let image = Image::new(&raw_image, Point::zero());

    // Configure GPIO25 as an output
    let mut led_pin = pins.gpio25.into_push_pull_output();
    loop {
        led_pin.set_high().unwrap();
        
        for y in 0..100 {
            for x in 0..100 {
                display.pixel(x,y,true);
            }
        }
        display.pixel(0,0,true);
        delay.delay_ms(100);
        display.update().unwrap();
        // TODO: Replace with proper 1s delays once we have clocks working
        delay.delay_ms(10000);
        
        led_pin.set_low().unwrap();
        for y in 0..100 {
            for x in 0..100 {
                display.pixel(x,y,true);
            }
        }
        
        image.draw(&mut display).unwrap();

        display.update().unwrap();
        delay.delay_ms(20000);
    }
}

// End of file
