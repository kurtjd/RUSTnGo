#![allow(unused)]
use crate::wrap::*;
use core::slice;

use defmt::*;
use embedded_sdmmc::*;
use heapless::Vec;

use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Blocking;
use embassy_stm32::spi::Spi;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Delay;

type SdCardType = SdCard<
    SpiDevice<'static, NoopRawMutex, Spi<'static, Blocking>, sdcard::DummyCsPin>,
    Output<'static>,
    Delay,
>;
type VolumeMangerType = VolumeManager<SdCardType, Ts>;

type GameListType = Vec<ShortFileName, 8>;

// Don't really care about accurate timestamps right now...
struct Ts;
impl TimeSource for Ts {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

pub struct Loader {
    volume_mgr: VolumeMangerType,
}

impl Loader {
    pub fn start_game() {
        /* The Rust compiler generates a blx instruction here (as opposed to bl),
         * which changes between ARM and Thumb depending on the LSB (if 0, branch and change to ARM
         * mode). This is annoying because all of our code is generated in Thumb mode, so have to
         * make sure to call with LSB set to stay in Thumb mode. There is likely a Rustier
         * alternative since transmute is very unsafe and I'm probably just hitting undefined
         * behavior.
         */
        info!("Starting game...");
        let game: extern "C" fn() = unsafe { core::mem::transmute(0x20001801 as *const ()) };
        game();
    }

    pub fn static_load(game_bin: &[u8]) {
        let game_mem: *mut u8 = 0x20001800 as *mut u8;
        unsafe {
            game_mem.copy_from(game_bin.as_ptr(), game_bin.len());
        }
    }

    pub fn new_sd(sd: SdCardType) -> Self {
        Self {
            volume_mgr: VolumeManager::new(sd, Ts),
        }
    }

    pub fn get_game_list(&mut self) -> GameListType {
        let mut game_list = GameListType::new();
        let mut volume0 = self.volume_mgr.open_volume(VolumeIdx(0)).unwrap();
        let mut root_dir = volume0.open_root_dir().unwrap();

        let mut start = false;
        root_dir.iterate_dir(|f| {
            // Skip name of volume itself
            if start {
                game_list.push(f.name.clone());
            }

            start = true;
        });

        game_list
    }

    /* Using a file manager might be seriously overkill here (had to tweak optimization settings to
     * optimize for space to even get it to fit in Flash) but it's quick and easy for now.
     */
    pub fn sd_load(&mut self, game: ShortFileName) {
        let mut volume0 = self.volume_mgr.open_volume(VolumeIdx(0)).unwrap();
        let mut root_dir = volume0.open_root_dir().unwrap();
        let mut game_bin = root_dir.open_file_in_dir(game, Mode::ReadOnly).unwrap();

        // Just do some nasty hack of converting pointer to slice
        let game_mem: *mut u8 = 0x20001800 as *mut u8;
        unsafe {
            let game_mem: &mut [u8] = slice::from_raw_parts_mut(game_mem, 10240);
            game_bin.read(game_mem).unwrap();
        }
    }

    pub fn title_select(&mut self) {
        let game_list = self.get_game_list();
        let num_games = game_list.len();
        let mut idx = 0;

        // Cycle through available games on the SD card until user selects one
        loop {
            display_clear();
            display_print(20, 35, "<");
            display_print(
                50,
                35,
                core::str::from_utf8(game_list[idx].base_name()).unwrap(),
            );
            display_print(100, 35, ">");
            display_update();

            while !is_pressed('L') && !is_pressed('R') && !is_pressed('A') {}
            if is_pressed('L') {
                if idx > 0 {
                    idx -= 1;
                } else {
                    idx = num_games - 1;
                }
            }
            if is_pressed('R') {
                if idx < num_games - 1 {
                    idx += 1;
                } else {
                    idx = 0;
                }
            }
            if is_pressed('A') {
                self.sd_load(game_list[idx].clone());
                play_tone(440);
                delay(1000);
                play_tone(0);

                // Will never return from here
                Self::start_game();
            }

            delay(300);
        }
    }
}
