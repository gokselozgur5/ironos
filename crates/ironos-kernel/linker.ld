OUTPUT_ARCH(riscv)
ENTRY(_start)

BASE_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;
    __kernel_start = .;

    .text : {
        *(.text.entry)
        *(.text .text.*)
    }

    . = ALIGN(4K);
    __rodata_start = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(4K);
    __data_start = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);
    __bss_start = .;
    .bss : {
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }
    . = ALIGN(4K);
    __bss_end = .;

    . = ALIGN(4K);
    __stack_bottom = .;
    . += 64K;
    __stack_top = .;

    . = ALIGN(4K);
    __kernel_end = .;
}
