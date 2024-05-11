/* as we do not have 64-bit codes here, it's safe to assume that */
typedef unsigned char uint8_t, *uint8_p;
typedef unsigned short int uint16_t, *uint16_p;
typedef unsigned long int uint32_t, *uint32_p;
typedef signed char int8_t, *int8_p;
typedef signed short int int16_t, *int16_p;
typedef signed long int int32_t, *int32_p;

#ifndef bool
typedef uint8_t bool;
# define true (1)
# define false (0)
#endif

static inline void outb(uint8_t value, uint16_t port) {
	asm volatile("outb %b0, %w1" : : "a"(value), "Nd"(port));
}

#define COM1 0x3f8
#define COM2 0x2f8

static inline void putchar(char c) {
    outb((uint8_t)c, COM1);
}

static inline void putsi(const char *s) {
    while (*s != '\0') {
        putchar(*(s++));
    }
}

static inline void puts(const char *s) {
    putsi(s);
    putchar('\n');
}

static inline void putud(uint32_t num) {
    static char buf[12];
    int8_t cnt = 0;
    while (num > 0) {
        buf[cnt++] = '0' + num % 10;
        num /= 10;
    }
    
    while (--cnt >= 0) {
        putchar(buf[cnt]);
    }
}

static inline void putux(uint32_t num, bool prefix, int8_t padding) {
    static char buf[10];
    const char *chars = "0123456789abcdef";
    int8_t cnt = 0;
    while (num > 0) {
        buf[cnt++] = chars[num % 16];
        num /= 16;
    }
    
    if (padding > cnt && padding <= 10) {
        while (cnt < padding) {
            buf[cnt++] = '0';
        }
    }

    if (prefix) {
        putchar('0');
        putchar('x');
    }

    while (--cnt >= 0) {
        putchar(buf[cnt]);
    }
}
