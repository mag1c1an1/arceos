#include "defs.h"

#pragma pack(push, 1)
typedef struct _kernel_header {
    uint8_t _01f0;
    uint8_t setup_sects;
    uint16_t root_flags;
    uint32_t syssize;
    uint16_t ramsize;
    uint16_t vid_mode;
    uint16_t root_dev;
    uint16_t boot_flag;
    uint16_t jump;
    uint32_t header;
    uint16_t version;
    uint32_t realmode_swtch;
    uint16_t start_sys_seg;
    uint16_t kernel_version;
    uint8_t type_of_loader;
    uint8_t loadflags;
    uint16_t setup_move_size;
    uint32_t code32_start;
    uint32_t ramdisk_image;
    uint32_t ramdisk_size;
    uint32_t bootsect_kludge;
    uint16_t heap_end_ptr;
    uint8_t ext_loader_ver;
    uint8_t ext_loader_type;
    uint32_t cmd_line_ptr;
    uint32_t initrd_addr_max;
    uint32_t kernel_alignment;
    uint8_t relocatable_kernel;
    uint8_t min_alignment;
    uint16_t xloadflags;
    uint32_t cmdline_size;
    uint32_t hardware_subarch;
    uint32_t hardware_subarch_data_l;
    uint32_t hardware_subarch_data_h;
    uint32_t payload_offset;
    uint32_t payload_length;
    uint32_t setup_data_l;
    uint32_t setup_data_h;
    uint32_t pref_address_l;
    uint32_t pref_address_h;
    uint32_t init_size;
    uint32_t handover_offset;
    uint32_t kernel_info_offset;
} kernel_header, *kernel_header_ptr;
#pragma pack(pop)

void cpy4(void *dst, const void *src, uint32_t size) {
    const uint32_t * ptr_src = src;
    uint32_p ptr_dst = dst;
    for (int i = 0; i < size; i += 4) {
        *ptr_dst = *ptr_src;
        ptr_src++;
        ptr_dst++;
    }
}

const char cmd[256] = "console=uart8250,io,0x3f8,115200n8 debug root=/dev/vda\0";

int load_kernel(void *kernel_image, void *loc_real, void *stack_end, void *loc_prot) {
    kernel_header_ptr orig_header = kernel_image + 0x1f0;

    uint32_t kernel_lower_size = ((orig_header->setup_sects ? orig_header->setup_sects : 4) + 1) * 512;
    uint32_t kernel_upper_size = orig_header->syssize * 16;
    void *prot = kernel_image + kernel_lower_size;

    cpy4(loc_real, kernel_image, kernel_lower_size);
    cpy4(loc_prot, prot, kernel_upper_size);

    void *cmd_base = stack_end;
    void *setup_data_base = cmd_base + 256;

    cpy4(cmd_base, cmd, 256);

    kernel_header_ptr header = loc_real + 0x1f0;

    header->vid_mode = 0xffff;
    header->type_of_loader = 0xff;
    header->loadflags = (header->loadflags & 0x1f) | 0x80;
    header->code32_start = (uint32_t)(loc_prot);
    header->ramdisk_image = 0;
    header->ramdisk_size = 0;
    header->heap_end_ptr = (uint16_t)(stack_end - loc_real - 0x200);
    header->cmd_line_ptr = (uint32_t)cmd_base;
    header->setup_data_l = 0;
    header->setup_data_h = 0;

    return 0;
}
