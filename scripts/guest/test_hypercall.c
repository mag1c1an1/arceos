#include <signal.h>
#include <stdio.h>
#include <stdlib.h>

#define HYPERCALL "vmcall"

static void in_guest() {
	printf("Execute VMCALL OK.\n");
	printf("You are in the Guest mode.\n");
	exit(0);
}

static void in_host() {
	printf("Execute VMCALL failed.\n");
	printf("You are in the Host mode.\n");
	exit(1);
}

static void sig_handler(int signum) {
	printf("Caught signal %d\n", signum);
	in_host();
}

static inline long hypercall(int num) {
	long ret;
	asm volatile(HYPERCALL : "=a"(ret) : "a"(num) : "memory");
	return ret;
}

static inline long hypercall1(unsigned int nr, unsigned long p1)
{
    long ret;
    asm volatile(HYPERCALL
             : "=a"(ret)
             : "a"(nr), "b"(p1)
             : "memory");
    return ret;
}

static inline long hypercall2(unsigned int nr, unsigned long p1,
                  unsigned long p2)
{
    long ret;
    asm volatile(HYPERCALL
             : "=a"(ret)
             : "a"(nr), "b"(p1), "c"(p2)
             : "memory");
    return ret;
}

int main () {
	signal(SIGSEGV, sig_handler);
	signal(SIGILL, sig_handler);
	// int ret = hypercall(2333);
	int ret = hypercall1(9, 0b10);
	if (ret == 2333) {
		in_guest();
	} else {
		in_host();
	}

	return 0;
}
