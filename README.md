# :crab: RUSTnGo
<img src = "images/rustngo.jpg?raw=true">

Experimenting with the ability to write games in Rust for my [CHIPnGo console](https://github.com/kurtjd/CHIPnGo).

## Goal
The plan is to provide hardware functionality via "syscalls" that the user code written in Rust can call. Games can then be written like normal Rust programs (well, normal as far as baremetal/no-std goes), and make these "syscalls" into the firmware to do things like draw to the screen. On bootup, the console firmware will load these binaries from an SD card into RAM (or perhaps, maybe, into Flash) and begin executing them.

## WARNING
This is just a proof-of-concept for experimenting. The code here is terribly unsafe, not well-written, and full of undefined behavior. The plan however is to use what I learn here for a more polished product in the future.

## How it works
The firmware (located in `rustngo-fw`) essentially sets up and initializes hardware, then loads games
from an SD card into RAM (though I want to also experiment with loading games into Flash). The firmware
also provides helper functions for interacting with hardware (such as drawing to the display) which games can access by making "syscalls". The syscall function is placed at a specific address in Flash
so that games know where to call it (since games are compiled and linked separately, there is no easy
way to include the syscall symbol in the game binary).

With the firmware flashed to the microcontroller, games can now make use of it. Games use the helper
library (located in `rustngo-lib`, which serves as a wrapper around raw syscalls) to interact with the
hardware. This library also includes a Reset handler that is called when a game is first loaded (just your basic zeroing out the .bss section and loading the .data section).

Otherwise, games can be written like any other Rust program (well, like any other no-std, bare-metal Rust program). You now have all the fun of writing games for resource-constrained hardware
but with the help of a modern language and toolchain like Rust! :D

Games should be built in release mode. As an example, try building the Pong demo:

`cd demos`  
`cargo build --bin pong --release`

You can then come back to the root of this repo and run:

`./game2bin pong`

To convert the Pong ELF into a raw binary ready to be loaded and executed by the firmware. Simply place
the resulting pong.bin file onto a FAT32 formatted SD card, insert it into the console, and hopefully
the firmware will detect it!

## Progress
* Can now browse and load games from SD card
* Implementing syscall behavior and now have basic interactable games working that can respond to input, draw to the display, and play sounds.
* Currently just hacking around with linker scripts to experiment with loading and executing arbitrary binaries.
