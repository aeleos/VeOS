#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]
#![no_std]

extern crate rlibc;
extern crate volatile;
extern crate spin;

#[macro_use]
mod ffi;
#[macro_use]
mod vga_buffer;
mod boot;

static OS_NAME: &str = "VeOS";

#[no_mangle]
pub extern fn main(magic_number: u32, information_structure_address: usize) {
    boot::init(magic_number, information_structure_address);
    vga_buffer::init();
    println!("Booted {} using {}...", OS_NAME, boot::get_bootloader_name());

    loop {
    }
}

#[lang = "eh_personality"]
extern fn eh_personality() {
}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("PANIC! in file '{}' at line {}:", file, line);
    println!("{}", fmt);
    loop {
    }
}
