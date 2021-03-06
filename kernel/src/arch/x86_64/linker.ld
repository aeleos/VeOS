ENTRY(start)
OUTPUT_FORMAT(elf64-x86-64)

PAGE_SIZE = CONSTANT(COMMONPAGESIZE);
STACK_SIZE = 128K;
LOADER_START = 1M;
KERNEL_OFFSET = 0xffff800000000000;

SECTIONS {
    . = LOADER_START;

    .init :
    {
        KEEP(*(.multiboot_header))
        *(.init)
        . = ALIGN(PAGE_SIZE);
    }

    _text_start = .;

    . += KERNEL_OFFSET;

    .text : AT(ADDR(.text) - KERNEL_OFFSET)
    {
        *(.text .text.*)
        . = ALIGN(PAGE_SIZE);
    }

    _rodata_start = . - KERNEL_OFFSET;

    .rodata : AT(ADDR(.rodata) - KERNEL_OFFSET)
    {
        *(.rodata .rodata.*)
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
        . = ALIGN(PAGE_SIZE);
    }

    _data_start = . - KERNEL_OFFSET;

    .data : AT(ADDR(.data) - KERNEL_OFFSET)
    {
        *(.data .data.*)
        KERNEL_END = .;
        QUAD(_kernel_end);
        TEXT_START = .;
        QUAD(_text_start);
        RODATA_START = .;
        QUAD(_rodata_start);
        DATA_START = .;
        QUAD(_data_start);
        BSS_START = .;
        QUAD(_bss_start);
        BSS_END = .;
        QUAD(_bss_end);
        TEMPORARY_MAP_TABLE = .;
        QUAD(temporary_map);
        L4_TABLE = .;
        QUAD(l4_table);
        L3_TABLE = .;
        QUAD(l3_table);
        L2_TABLE = .;
        QUAD(l2_table);
        STACK_L2_TABLE = .;
        QUAD(stack_l2_table);
        STACK_L1_TABLE = .;
        QUAD(stack_l1_table);
        STACK_BOTTOM = .;
        QUAD(stack_bottom);
        STACK_TOP = .;
        QUAD(stack_top);
        . = ALIGN(PAGE_SIZE);
    }

    _bss_start = . - KERNEL_OFFSET;

    .bss : AT(ADDR(.bss) - KERNEL_OFFSET)
    {
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(PAGE_SIZE);
        temporary_map = . - KERNEL_OFFSET;
        . += PAGE_SIZE;
        _bss_end = . - KERNEL_OFFSET;
        stack_bottom = . - KERNEL_OFFSET;
        . += STACK_SIZE;
        stack_top = . - KERNEL_OFFSET;
        . = ALIGN(PAGE_SIZE);
        l4_table = . - KERNEL_OFFSET;
        . += PAGE_SIZE;
        l3_table = . - KERNEL_OFFSET;
        . += PAGE_SIZE;
        l2_table = . - KERNEL_OFFSET;
        . += PAGE_SIZE;
        stack_l2_table = . - KERNEL_OFFSET;
        . += PAGE_SIZE;
        stack_l1_table = . - KERNEL_OFFSET;
        . += PAGE_SIZE;
    }
    
    _kernel_end = . - KERNEL_OFFSET;
}
