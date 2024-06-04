#![no_std]
#![no_main]

mod syscall;
use syscall::*;
mod loader;
use loader::*;
mod wrap;

use core::cell::RefCell;

use cortex_m_rt::entry;
use {defmt_rtt as _, panic_probe as _};

use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_stm32::gpio::{Input, Level, Output, OutputType, Pull, Speed};
use embassy_stm32::spi::{self, Config, Spi};
use embassy_stm32::time::hz;
use embassy_stm32::timer::{
    simple_pwm::{PwmPin, SimplePwm},
    Channel,
};
use embassy_sync::blocking_mutex::NoopMutex;
use embassy_time::Delay;

use embedded_sdmmc::{sdcard, SdCard};
use st7567_rs::ST7567;

#[entry]
fn main() -> ! {
    // Initialize the PAC
    let p = embassy_stm32::init(Default::default());

    /* Setup buttons
     * For now, games can only poll button status via syscall.
     * Also not sure why I need unsafe to access the values of blocking mutexes (even with lock).
     * Not the case for async mutexes, but whatever, gotta figure out why there is like a dozen
     * different types of mutexes.
     */
    let a_btn = Input::new(p.PB11, Pull::Up);
    let b_btn = Input::new(p.PB10, Pull::Up);
    let u_btn = Input::new(p.PB7, Pull::Up);
    let d_btn = Input::new(p.PB4, Pull::Up);
    let l_btn = Input::new(p.PB5, Pull::Up);
    let r_btn = Input::new(p.PB6, Pull::Up);
    unsafe {
        *BTN_A.get_mut() = Some(a_btn);
        *BTN_B.get_mut() = Some(b_btn);
        *BTN_U.get_mut() = Some(u_btn);
        *BTN_D.get_mut() = Some(d_btn);
        *BTN_L.get_mut() = Some(l_btn);
        *BTN_R.get_mut() = Some(r_btn);
    }

    /* Setup PWM for buzzer
     * User can change frequency of PWM signal and enable/disable it, but duty cycle is fixed.
     */
    let ch4 = PwmPin::new_ch4(p.PB1, OutputType::PushPull);
    let mut pwm = SimplePwm::new(
        p.TIM3,
        None,
        None,
        None,
        Some(ch4),
        hz(440),
        Default::default(),
    );
    pwm.set_duty(Channel::Ch4, pwm.get_max_duty() / 2);
    unsafe {
        *PWM.get_mut() = Some(pwm);
    }

    // SPI config for display
    let mut spi_config = Config::default();
    spi_config.frequency = hz(1_000_000);
    spi_config.mode = spi::Mode {
        polarity: spi::Polarity::IdleHigh,
        phase: spi::Phase::CaptureOnSecondTransition,
    };

    // Initialize SPI pins and create SPI device for display
    let rst = Output::new(p.PB14, Level::Low, Speed::Low);
    let bl = Output::new(p.PB3, Level::Low, Speed::Low);
    let dc = Output::new(p.PA8, Level::Low, Speed::Low);
    let cs = Output::new(p.PB12, Level::High, Speed::Low);
    let spi = Spi::new_blocking_txonly(p.SPI2, p.PB13, p.PB15, spi_config);
    unsafe {
        DISPLAY_SPI = Some(NoopMutex::new(RefCell::new(spi)));
    }
    let spi = SpiDevice::new(unsafe { DISPLAY_SPI.as_ref().unwrap() }, cs);

    // Initialize display driver
    let mut display = ST7567::new(dc, bl, rst, spi);
    display.init().unwrap();
    unsafe {
        *DISPLAY.get_mut() = Some(display);
    }

    // SPI config for SD card
    let mut spi_config = Config::default();
    spi_config.frequency = hz(1_000_000);
    spi_config.mode = spi::Mode {
        polarity: spi::Polarity::IdleLow,
        phase: spi::Phase::CaptureOnFirstTransition,
    };

    // Initialize SPI pins and create SPI device for SD card
    let cs = Output::new(p.PA4, Level::Low, Speed::Low);
    let spi = Spi::new_blocking(p.SPI1, p.PA5, p.PA7, p.PA6, spi_config);
    unsafe {
        SD_SPI = Some(NoopMutex::new(RefCell::new(spi)));
    }
    let spi = SpiDevice::new(unsafe { SD_SPI.as_ref().unwrap() }, sdcard::DummyCsPin);
    let sdcard = SdCard::new(spi, cs, Delay);

    // Initialize loader and go to title select menu
    let mut loader = Loader::new_sd(sdcard);
    loader.title_select();

    defmt::panic!("If you are seeing this the universe imploded!");
}
