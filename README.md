# :crab: RUSTnGo
Experimenting with the ability to write games in Rust for my [CHIPnGo console](https://github.com/kurtjd/CHIPnGo).

## Goal
The plan is to provide hardware functionality via "syscalls" that the user code written in Rust can call. Games can then be written like normal Rust programs (well, normal as far as baremetal/no-std goes), and make these "syscalls" into the firmware to do things like draw to the screen. On bootup, the console firmware will load these binaries from an SD card into RAM (or perhaps, maybe, into Flash) and begin executing them.

## WARNING
This is just a proof-of-concept for experimenting. The code here is terribly unsafe, not well-written, and full of undefined behavior.  
The plan however is to use what I learn here for a more polished product in the future.

## Progress
* Implementing syscall behavior and now have basic interactable games working that can respond to input, draw to the display, and play sounds.
* Currently just hacking around with linker scripts to experiment with loading and executing arbitrary binaries.
