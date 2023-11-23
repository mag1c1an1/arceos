#include "defs.h"

void dump_int_args(uint32_t i, uint32_p eax, uint32_p ecx, uint32_p edx, uint32_p ebx, uint32_p ebp, uint32_p esi, uint32_p edi, uint16_p flags) {
    putsi("[BIOS] int: ");
    putux(i, true, 2);
    putsi(", eax: ");
    putux(*eax, true, 8);
    putsi(", ecx: ");
    putux(*ecx, true, 8);
    putsi(", edx: ");
    putux(*edx, true, 8);
    putsi(", ebx: ");
    putux(*ebx, true, 8);
    putsi(", ebp: ");
    putux(*ebp, true, 8);
    putsi(", esi: ");
    putux(*esi, true, 8);
    putsi(", edi: ");
    putux(*edi, true, 8);
    putsi(", flags: ");
    putux(*flags, true, 4);
    puts("");
}

void log(const char *s) {
    putsi("[BIOS] ");
    puts(s);
}

extern void cpy_to_es4(uint32_t to, uint32_t from, uint32_t length);

extern void cpy_to_es_eg(uint32_p to, uint32_p from, uint32_t length) {
    for (int i = 0; i < length; i += 4) {
        *to = *from;
        to++;
        from++;
    }
}

struct mem_range {
    uint32_t start;
    uint32_t length;
    uint32_t type;
};

/*
our memory layout now:
00000000 ~ 00007000, free
00007000 ~ 00010000, for this BIOS
00010000 ~ 01000000, free (64k ~ 1m ~ 16m)
01000000 ~ 10000000, free (16m ~ 256m)
70000000 ~ 80000000, free
fec00000 ~ fec01000, mmio
fed00000 ~ fed01000, mmio
fee00000 ~ fee01000, mmio
*/
const struct mem_range mem_ranges[] = {
    { 0x0, 0x7000, 1 },
    { 0x7000, 0x9000, 2 },
    { 0x10000, 0xff0000, 1 },
    { 0x1000000, 0xf000000, 1 },
    { 0x70000000, 0x10000000, 1 },
    { 0xfec00000, 0x1000, 2 },
    { 0xfed00000, 0x1000, 2 },
    { 0xfee00000, 0x1000, 2 },
};

const uint32_t mem_range_count = sizeof(mem_ranges) / sizeof(struct mem_range);

void find_next_mem_range(uint32_p ebx, uint32_p next, uint32_p length, uint32_p type) {
    if (*ebx >= mem_range_count) {
        *next = *length = *type = *ebx = 0;
        return;
    }

    *next = mem_ranges[*ebx].start;
    *length = mem_ranges[*ebx].length;
    *type = mem_ranges[*ebx].type;
    (*ebx)++;

    return;
}

void handler(uint32_t i, uint32_t eax_addr, uint32_t ecx_addr, uint32_t edx_addr, uint32_t ebx_addr, uint32_t ebp_addr, uint32_t esi_addr, uint32_t edi_addr, uint32_t flags_addr) {
    uint32_p eax = (uint32_p)eax_addr;
    uint32_p ecx = (uint32_p)ecx_addr;
    uint32_p edx = (uint32_p)edx_addr;
    uint32_p ebx = (uint32_p)ebx_addr;
    uint32_p ebp = (uint32_p)ebp_addr;
    uint32_p esi = (uint32_p)esi_addr;
    uint32_p edi = (uint32_p)edi_addr;
    uint16_p flags = (uint16_p)flags_addr;

    if (i == 0x10) {
        // 0x10, ignored
        // puts("0x10, ignored");
        return;
    } 
    
    dump_int_args(i, eax, ecx, edx, ebx, ebp, esi, edi, flags);
    uint32_t fn = *eax;
        
    if (i == 0x15) {
        // see http://www.uruk.org/orig-grub/mem64mb.html for helps about int 15h, ax=e820h/e801h/8800h
        if (fn == 0xec00) {
            if (*ebx == 2) {
                log("OS tells BIOS it'll be 64-bit, ok");
            } else {
                log("Unknown ebx for ec00!");
            }
        } else if (fn == 0xe820) {
            static uint32_t buf[5];

            find_next_mem_range(ebx, &buf[0], &buf[2], &buf[4]);
            buf[1] = buf[3] = 0;

            *eax = *edx; // SMAP signature
            *edx = 0;
            *flags &= 0xfffe;

            cpy_to_es4(*edi, (uint32_t)(&buf[0]), 0x14);
        } else if (fn == 0xe801) {
            *eax = *ecx = 0x3c00;
            *ebx = *edx = 0;
            *flags &= 0xfffe;
        } else if (fn == 0x8800) {
            *eax = 0;
            *flags &= 0xfffe;
        } else {
            log("Unknown eax for int 15h!");
        }
    } else if (i == 0x16) {
        if (fn == 0x0200) {
            // Read keyboard status, return 0
            *eax = 0;
        } else if (fn == 0x0305) {
            // Set typematic rate/delay, ignore
        } else {
            log("Unknown eax for int 16h!");
        }
    } else {
        log("Unsupported int!");
    }
    
}
