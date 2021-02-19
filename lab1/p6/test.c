#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <linux/kernel.h>
#include <sys/syscall.h>
#include <unistd.h>

int main()
{
    char buf[20];
    sprintf(buf, "Hello World");
    long ret = syscall(548, buf, strlen(buf) + 1);
    printf("%s\n", buf);
    printf("%ld\n", ret);
}
