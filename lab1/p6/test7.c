#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <linux/kernel.h>
#include <asm/unistd.h>
#include <sys/types.h>

int main()
{
    char buf[20];
    sprintf(buf, "Hello World");

    long ret;
    asm volatile (
        "syscall" 
        : "=a" (ret) 
        : "0" (548), "D" (buf), "S" (strlen(buf) + 1)
    );

    printf("%s\n", buf);
    printf("%ld\n", ret);
    return 0;
}

