#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::*;
use embassy_stm32::gpio::{Input, OutputType, Pull};
use embassy_stm32::peripherals::TIM3;
use embassy_stm32::time::hz;
use embassy_stm32::timer::{
    simple_pwm::{PwmPin, SimplePwm},
    Channel,
};
use embassy_sync::blocking_mutex::{raw::NoopRawMutex, Mutex};
use {defmt_rtt as _, panic_probe as _};

static mut BTN_A: Mutex<NoopRawMutex, Option<Input<'static>>> = Mutex::new(None);
static mut BTN_B: Mutex<NoopRawMutex, Option<Input<'static>>> = Mutex::new(None);
static mut BTN_U: Mutex<NoopRawMutex, Option<Input<'static>>> = Mutex::new(None);
static mut BTN_D: Mutex<NoopRawMutex, Option<Input<'static>>> = Mutex::new(None);
static mut BTN_L: Mutex<NoopRawMutex, Option<Input<'static>>> = Mutex::new(None);
static mut BTN_R: Mutex<NoopRawMutex, Option<Input<'static>>> = Mutex::new(None);

static mut PWM: Mutex<NoopRawMutex, Option<SimplePwm<'static, TIM3>>> = Mutex::new(None);

fn bad_syscall() -> u8 {
    info!("Unrecognized syscall");
    1
}

// OP: 1
fn print(buf: &[u8]) -> u8 {
    let msg = core::str::from_utf8(buf).unwrap();
    info!("{}", msg);
    0
}

// OP: 2
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

/* The syscall serves as the single point of entry by games into the firmware.
 * This provides extra functionality to games such as hardware access.
 * This allows us to not have to bloat game binary sizes by copying all this functionality
 * into their binaries. Though it does make for some interesting linker challenges :)
 */
#[link_section = ".syscall"]
#[inline(never)]
#[no_mangle]
fn syscall(op: u8, buf: &[u8]) -> u8 {
    match op {
        1 => print(buf),
        2 => delay(buf),
        3 => is_pressed(buf),
        4 => play_tone(buf),
        _ => bad_syscall(),
    }
}

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

    /* For now, load the game binary directly at compile-time
     * (goal is to load at runtime from SD card) into RAM.
     * Also want to experiment with loading into Flash eventually.
     * Once binary is loaded, attempt to branch to it.
     */
    let game_mem: *mut u8 = 0x20001800 as *mut u8;
    let game_bin = include_bytes!("../game.bin");
    unsafe {
        game_mem.copy_from(game_bin.as_ptr(), game_bin.len());
    }

    /* The Rust compiler generates a blx instruction here (as opposed to bl),
     * which changes between ARM and Thumb depending on the LSB (if 0, branch and change to
     * ARM mode). This is annoying because all of our code is generated in Thumb mode, so have to
     * make sure to call with LSB set to stay in Thumb mode. There is likely a Rustier alternative
     * since transmute is very unsafe and I'm probably just hitting undefined behavior.
     */
    info!("Starting game...");
    let game: extern "C" fn() = unsafe { core::mem::transmute(0x20001801) };
    game();

    defmt::panic!("If you are seeing this the universe imploded!");
}
