ENTRY(_boot_cores);

/* Provide some defaults */
PROVIDE(Reset = _boot_cores);
PROVIDE(UndefinedInstruction = Reset);
PROVIDE(SoftwareInterrupt = Reset);
PROVIDE(PrefetchAbort = Reset);
PROVIDE(DataAbort = Reset);
PROVIDE(ReservedException = Reset);
PROVIDE(IRQ = Reset);
PROVIDE(FIQ = Reset);

MEMORY
{
    SDRAM : ORIGIN = 0x00100000, LENGTH = 0x1FF00000
}

SECTIONS
{
    .text :
    {
        KEEP(*(.text.exceptions));
        *(.text.boot);
        *(.text .text.*);
    } > SDRAM
 
    .rodata : ALIGN(4)
    {
        *(.rodata .rodata.*);
    } > SDRAM
 
    .data : ALIGN(4)
    {
        *(.data .data.*);
    } > SDRAM
 
    .bss (NOLOAD) : ALIGN(4)
    {
        __bss_start = .;
        *(.bss .bss.*);
        . = ALIGN(4);
        __bss_end = .;
    } > SDRAM

    .heap (NOLOAD) : ALIGN(8)
    {
        __heap_start = .;
        . += 0x8000000;
        __heap_end = .;
    } > SDRAM

    .stack1 (NOLOAD) : ALIGN(8)
    {
        __stack1_end = .;
        . += 0x8000000;
        __stack1_start = .;
    } > SDRAM

    .stack0 (NOLOAD) : ALIGN(8)
    {
        __stack0_end = .;
        . += 0x10000;
        __stack0_start = .;
    } > SDRAM

    /DISCARD/ :
    {
        /* Unused exception related info that only wastes space */
        *(.ARM.exidx);
        *(.ARM.exidx.*);
        *(.ARM.extab.*);
    }
}
