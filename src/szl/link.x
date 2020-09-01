ENTRY(Reset);

MEMORY
{
    /* 256 kB On-Chip Memory */
    OCM : ORIGIN = 0, LENGTH = 0x30000
    SDRAM : ORIGIN = 0x00100000, LENGTH = 0x1FF00000
    OCM3 : ORIGIN = 0xFFFF0000, LENGTH = 0x10000
}

SECTIONS
{
    .text :
    {
        KEEP(*(.text.exceptions));
        *(.text.boot);
        *(.text .text.*);
    } > OCM
 
    .rodata : ALIGN(4)
    {
        *(.rodata .rodata.*);
    } > OCM
 
    .data : ALIGN(4)
    {
        *(.data .data.*);
    } > OCM

    .heap (NOLOAD) : ALIGN(8)
    {
        __runtime_start = .;
        . += 0x8000000;
        __runtime_end = .;
        __heap0_start = .;
        . += 0x8000000;
        __heap0_end = .;
    } > SDRAM
 
    .bss (NOLOAD) : ALIGN(4)
    {
        __bss_start = .;
        *(.bss .bss.*);
        . = ALIGN(4);
        __bss_end = .;
    } > OCM3

    .stack1 (NOLOAD) : ALIGN(8)
    {
        __stack1_end = .;
        . += 0x100;
        __stack1_start = .;
    } > OCM3

    .stack0 (NOLOAD) : ALIGN(8)
    {
        __stack0_end = .;
        . += 0x4000;
        __stack0_start = .;
    } > OCM3

    /DISCARD/ :
    {
        /* Unused exception related info that only wastes space */
        *(.ARM.exidx);
        *(.ARM.exidx.*);
        *(.ARM.extab.*);
    }
}
