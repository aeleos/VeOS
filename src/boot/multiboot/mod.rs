//! Handles the multiboot information structure.

use core::mem;
use memory::{FreeMemoryArea, get_kernel_end_address, get_kernel_start_address};

/// Represents the multiboot information structure.
#[repr(C)]
struct MultibootInformation {
    flags: u32,
    mem_lower: u32,
    mem_upper: u32,
    boot_device: u32,
    cmdline: u32,
    mods_count: u32,
    mods_addr: u32,
    elf_num: u32, // Only elf tags are supported, because this kernel is an ELF file.
    elf_size: u32,
    elf_addr: u32,
    elf_shndx: u32,
    mmap_length: u32,
    mmap_addr: u32,
    drives_length: u32,
    drives_addr: u32,
    config_table: u32,
    boot_loader_name: u32,
    apm_table: u32,
    vbe_control_info: u32,
    vbe_mode_info: u32,
    vbe_mode: u16,
    vbe_interface_seg: u16,
    vbe_interface_off: u16,
    vbe_interface_len: u16
}

bitflags! {
    ///The possible flags in the flags field.
    flags MultibootFlags: u32 {
        ///Basic memory information is available.
        const BASIC_MEMORY = 1 << 0,
        ///Boot device information is available.
        const BOOT_DEVICE = 1 << 1,
        ///A command line is available.
        const CMDLINE = 1 << 2,
        ///Module information is available.
        const MODULES = 1 << 3,
        ///a.out information is available.
        const A_OUT = 1 << 4,
        ///Elf information is available.
        const ELF = 1 << 5,
        ///A memory map is available.
        const MMAP = 1 << 6,
        ///Information about drives  is available.
        const DRIVES = 1 << 7,
        ///A config table is available.
        const CONFIG_TABLE = 1 << 8,
        ///The boot loader name is available.
        const BOOT_LOADER_NAME = 1 << 9,
        ///An APM table is available.
        const APM_TABLE = 1 << 10,
        ///VBE information is available.
        const VBE = 1 << 11
    }
}

/// Represents an entry in the given memory map.
#[derive(Debug)]
#[repr(C, packed)]
struct MmapEntry {
    size: u32,
    base_addr: usize,
    length: usize,
    mem_type: u32
}

/// The base address for the information strucuture.
// This is only valid after init was called.
static mut STRUCT_BASE_ADDRESS: *const MultibootInformation = 0 as *const MultibootInformation;

/// Initializes the multiboot module.
pub fn init(information_structure_address: usize) {
    assert_has_not_been_called!("The multiboot module should only be initialized once.");

    unsafe {
        STRUCT_BASE_ADDRESS = to_virtual!(information_structure_address) as
                              *const MultibootInformation
    };
    assert!(!get_flags().contains(A_OUT | ELF));
}

/// Returns the name of the boot loader.
pub fn get_bootloader_name() -> &'static str {
    if get_flags().contains(BOOT_LOADER_NAME) {
        from_c_str!(to_virtual!(get_info().boot_loader_name)).unwrap()
    } else {
        // When no specific name was given by the boot loader.
        "a multiboot compliant bootloader"
    }
}

/// Returns the flags of the multiboot structure.
fn get_flags() -> MultibootFlags {
    MultibootFlags::from_bits_truncate(get_info().flags)
}

/// Returns the multiboot structure.
fn get_info() -> &'static MultibootInformation {
    unsafe { &*STRUCT_BASE_ADDRESS }
}

/// Provides an iterator for the memory map.
pub struct MemoryMapIterator {
    /// The address of the current entry in the memory map.
    address: usize,
    /// The address after the last entry in the memory map.
    max_address: usize,
    /// The length of the segment withing the current entry after the kernel.
    next_length: usize
}

impl MemoryMapIterator {
    /// Creates a new iterator through the memory map.
    fn new() -> MemoryMapIterator {
        if get_flags().contains(MMAP) {
            MemoryMapIterator {
                address: to_virtual!(get_info().mmap_addr),
                max_address: to_virtual!(get_info().mmap_addr + get_info().mmap_length),
                next_length: 0
            }
        } else {
            MemoryMapIterator {
                address: 0,
                max_address: 0,
                next_length: 0
            }
        }
    }
}

impl Iterator for MemoryMapIterator {
    type Item = FreeMemoryArea;

    fn next(&mut self) -> Option<FreeMemoryArea> {
        while self.address < self.max_address {
            let current_entry = unsafe { &*(self.address as *const MmapEntry) };

            if self.next_length > 0 {
                let next_length = self.next_length;
                self.next_length = 0;
                return Some(FreeMemoryArea::new(get_kernel_end_address(), next_length));
            }

            self.address += mem::size_of::<u32>() + current_entry.size as usize;

            // Only a type of 1 is usable memory.
            if current_entry.mem_type == 1 {
                if get_kernel_end_address() < current_entry.base_addr ||
                   get_kernel_start_address() > current_entry.base_addr + current_entry.length {
                    // If the kernel is before or after this segment.
                    return Some(FreeMemoryArea::new(current_entry.base_addr, current_entry.length));
                } else if current_entry.base_addr <= get_kernel_start_address() {
                    // This should handle all other cases.
                    self.next_length = current_entry.base_addr + current_entry.length -
                                       get_kernel_end_address();
                    if current_entry.base_addr != get_kernel_start_address() {
                        return Some(FreeMemoryArea::new(current_entry.base_addr,
                                                        get_kernel_start_address() -
                                                        current_entry.base_addr));
                    }
                } else {
                    panic!("There's a bug in the multiboot code.");
                }
            }
        }
        None
    }
}

/// Returns the memory map given by the boot loader.
pub fn get_memory_map() -> MemoryMapIterator {
    MemoryMapIterator::new()
}