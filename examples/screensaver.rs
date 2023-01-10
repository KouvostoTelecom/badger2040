//! # Rust Badge for badger2040
//! # This example gives 10 slow refresh passes trying to reset eink screen to blank and remove ghosting

#![no_std]
#![no_main]

// Halt if panic
use panic_halt as _;

// Required traits

use fugit::RateExtU32;
use rp2040_hal::clocks::Clock;

// Hardware
use badger2040::bsp;
use bsp::entry;
use bsp::hal;
use bsp::hal::pac;
use uc8151::Uc8151;

#[entry]
fn main() -> ! {
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

    let _sclk: bsp::Sclk = pins.sclk.into_mode();
    let _miso: bsp::Miso = pins.miso.into_mode();
    let _mosi: bsp::Mosi = pins.mosi.into_mode();

    let spi = hal::Spi::<_, _, 8>::new(pac.SPI0);

    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        10_000_000u32.Hz(),
        &embedded_hal::spi::MODE_0,
    );

    let dc_pin: bsp::InkyDc = pins.inky_dc.into_mode();
    let reset_pin: bsp::InkyReset = pins.inky_res.into_mode();
    let busy_pin: bsp::InkyBusy = pins.inky_busy.into_mode();
    let spi_cs: bsp::InkyCs = pins.inky_cs_gpio.into_mode();

    let mut display = Uc8151::new(spi, spi_cs, dc_pin, busy_pin, reset_pin);

    display.enable();
    display.setup(&mut delay, uc8151::LUT::Normal).unwrap();

    for _ in 1..10 {
        display.update().unwrap();
        delay.delay_ms(1000);
    }

    #[allow(clippy::empty_loop)]
    loop {}
}

// End of file
