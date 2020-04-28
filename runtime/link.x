ENTRY(_boot_cores);

STACK_SIZE = 0x8000;
HEAP_SIZE = 0x1000000;

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
 
    .bss (NOLOAD) : ALIGN(0x4000)
    {
        /* Aligned to 16 kB */
        KEEP(*(.bss.l1_table));
        *(.bss .bss.*);
        . = ALIGN(4);
    } > SDRAM
    __bss_start = ADDR(.bss);
    __bss_end = ADDR(.bss) + SIZEOF(.bss);

    .heap (NOLOAD) : ALIGN(0x1000)
    {
        . += HEAP_SIZE;
    } > SDRAM
    __heap_start = ADDR(.heap);
    __heap_end = ADDR(.heap) + SIZEOF(.heap);

    .stack (NOLOAD) : ALIGN(0x1000)
    {
        . += STACK_SIZE;
    } > SDRAM
    __stack_end = ADDR(.stack);
    __stack_start = ADDR(.stack) + SIZEOF(.stack);

  /DISCARD/ :
  {
    /* Unused exception related info that only wastes space */
    *(.ARM.exidx);
    *(.ARM.exidx.*);
    *(.ARM.extab.*);
  }
}
