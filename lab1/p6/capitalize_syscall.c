#include <linux/kernel.h>
#include <linux/syscalls.h>

SYSCALL_DEFINE2(capitalize_syscall, char __user *, buffer, int, len) 
{
    int i = 0;

    if (buffer == NULL)
        return 0;
    
    printk(KERN_INFO "%s\n", buffer);
    
    while (i < len)
    {
        if (buffer[i] >= 'a' && buffer[i] <= 'z')
            buffer[i] -= 32;
        i++;
    }
    return 0;
}
