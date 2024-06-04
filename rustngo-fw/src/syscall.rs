use core::cell::RefCell;

use defmt::*;
use {defmt_rtt as _, panic_probe as _};

use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_stm32::gpio::{Input, Output};
use embassy_stm32::mode::Blocking;
use embassy_stm32::peripherals::TIM3;
use embassy_stm32::spi::Spi;
use embassy_stm32::time::hz;
use embassy_stm32::timer::{simple_pwm::SimplePwm, Channel};
use embassy_sync::blocking_mutex::{raw::NoopRawMutex, Mutex};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle, StyledDrawable},
    text::Text,
};

use st7567_rs::ST7567;

// Jeesh...
type DisplayType = Mutex<
    NoopRawMutex,
    Option<
        ST7567<
            Output<'static>,
            Output<'static>,
            Output<'static>,
            SpiDevice<'static, NoopRawMutex, Spi<'static, Blocking>, Output<'static>>,
        >,
    >,
>;
type ButtonType = Mutex<NoopRawMutex, Option<Input<'static>>>;
type PwmType = Mutex<NoopRawMutex, Option<SimplePwm<'static, TIM3>>>;
type SpiType = Option<Mutex<NoopRawMutex, RefCell<Spi<'static, Blocking>>>>;

/* We have all these static globals (and hence mutexes) because syscall needs access to all these
 * but syscall is called externally by games so we can't just pass references around easily.
 * In fact, might not really need mutexes here since we have only one thread of execution
 * and no interrupts. I thought we used NoopRawMutex to say "this is safe to access without lock
 * because there is only one thread of execution" but yet accessing them still requires unsafe
 * in blocking mode for some reason. Ah well...
 */
pub static mut BTN_A: ButtonType = Mutex::new(None);
pub static mut BTN_B: ButtonType = Mutex::new(None);
pub static mut BTN_U: ButtonType = Mutex::new(None);
pub static mut BTN_D: ButtonType = Mutex::new(None);
pub static mut BTN_L: ButtonType = Mutex::new(None);
pub static mut BTN_R: ButtonType = Mutex::new(None);
pub static mut PWM: PwmType = Mutex::new(None);
pub static mut DISPLAY_SPI: SpiType = None;
pub static mut DISPLAY: DisplayType = Mutex::new(None);
pub static mut SD_SPI: SpiType = None;

fn bad_syscall() -> u8 {
    info!("Unrecognized syscall");
    1
}

// OP: 1
#[inline(never)]
fn console_print(buf: &[u8]) -> u8 {
    let msg = core::str::from_utf8(buf).unwrap();
    info!("{}", msg);
    0
}

// OP: 2
#[inline(never)]
fn delay(buf: &[u8]) -> u8 {
    let start = embassy_time::Instant::now();
    loop {
        let elapsed = embassy_time::Instant::now() - start;
        let duration: [u8; 4] = buf.try_into().unwrap();
        if elapsed >= embassy_time::Duration::from_millis((u32::from_le_bytes(duration)).into()) {
            break;
        }
    }
    0
}

// OP: 3
#[inline(never)]
fn is_pressed(buf: &[u8]) -> u8 {
    match buf[0] {
        b'A' => unsafe { BTN_A.lock(|m| m.as_ref().unwrap().is_low() as u8) },
        b'B' => unsafe { BTN_B.lock(|m| m.as_ref().unwrap().is_low() as u8) },
        b'U' => unsafe { BTN_U.lock(|m| m.as_ref().unwrap().is_low() as u8) },
        b'D' => unsafe { BTN_D.lock(|m| m.as_ref().unwrap().is_low() as u8) },
        b'L' => unsafe { BTN_L.lock(|m| m.as_ref().unwrap().is_low() as u8) },
        b'R' => unsafe { BTN_R.lock(|m| m.as_ref().unwrap().is_low() as u8) },
        _ => 0,
    }
}

// OP: 4
#[inline(never)]
fn play_tone(buf: &[u8]) -> u8 {
    let freq: [u8; 4] = buf.try_into().unwrap();
    let freq = u32::from_le_bytes(freq);
    let pwm = unsafe { PWM.get_mut().as_mut().unwrap() };

    if freq != 0 {
        pwm.set_frequency(hz(freq));
        pwm.enable(Channel::Ch4);
    } else if pwm.is_enabled(Channel::Ch4) {
        pwm.disable(Channel::Ch4);
    }
    0
}

// OP: 5
#[inline(never)]
fn display_print(buf: &[u8]) -> u8 {
    let x = buf[0] as i32;
    let y = buf[1] as i32;
    let msg = core::str::from_utf8(&buf[2..]).unwrap();
    let display = unsafe { DISPLAY.get_mut().as_mut().unwrap() };

    Text::new(
        msg,
        Point::new(x, y),
        MonoTextStyle::new(&FONT_6X10, BinaryColor::On),
    )
    .draw(display)
    .unwrap();
    0
}

// OP: 6
#[inline(never)]
fn display_draw_rect(buf: &[u8]) -> u8 {
    let x = buf[0] as i32;
    let y = buf[1] as i32;
    let w = buf[2] as u32;
    let h = buf[3] as u32;
    let style = if buf[4] == 1 {
        PrimitiveStyle::with_fill(BinaryColor::On)
    } else {
        PrimitiveStyle::with_stroke(BinaryColor::On, 1)
    };
    let display = unsafe { DISPLAY.get_mut().as_mut().unwrap() };

    Rectangle::new(Point::new(x, y), Size::new(w, h))
        .draw_styled(&style, display)
        .unwrap();
    0
}

// OP: 7
#[inline(never)]
fn display_draw_circle(buf: &[u8]) -> u8 {
    let x = buf[0] as i32;
    let y = buf[1] as i32;
    let d = buf[2] as u32;
    let style = if buf[3] == 1 {
        PrimitiveStyle::with_fill(BinaryColor::On)
    } else {
        PrimitiveStyle::with_stroke(BinaryColor::On, 1)
    };
    let display = unsafe { DISPLAY.get_mut().as_mut().unwrap() };

    Circle::new(Point::new(x, y), d)
        .draw_styled(&style, display)
        .unwrap();
    0
}

// OP: 8
#[inline(never)]
fn display_update() -> u8 {
    let display = unsafe { DISPLAY.get_mut().as_mut().unwrap() };
    display.show().unwrap();
    0
}

// OP: 9
#[inline(never)]
fn display_clear() -> u8 {
    let display = unsafe { DISPLAY.get_mut().as_mut().unwrap() };
    display.clear().unwrap();
    0
}

/* The syscall serves as the single point of entry by games into the firmware.
 * This provides extra functionality to games such as hardware access.
 * This allows us to not have to bloat game binary sizes by copying all this functionality
 * into their binaries. Though it does make for some interesting linker challenges :)
 * May experiment with using the PendSV interrupt instead, but that could make passing arguments
 * tricky.
 */
#[link_section = ".syscall"]
#[inline(never)]
#[no_mangle]
fn syscall(op: u8, buf: &[u8]) -> u8 {
    match op {
        1 => console_print(buf),
        2 => delay(buf),
        3 => is_pressed(buf),
        4 => play_tone(buf),
        5 => display_print(buf),
        6 => display_draw_rect(buf),
        7 => display_draw_circle(buf),
        8 => display_update(),
        9 => display_clear(),
        _ => bad_syscall(),
    }
}
