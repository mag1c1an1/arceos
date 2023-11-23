OUTPUT_ARCH(i386)

BASE_ADDRESS = 0x7c00;

ENTRY(entry)
SECTIONS
{
    . = BASE_ADDRESS;
    .text : {
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    /DISCARD/ : {
        *(.eh_frame) *(.eh_frame_hdr)
    }
}
