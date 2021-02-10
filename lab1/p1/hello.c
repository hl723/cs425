#include <linux/init.h>
#include <linux/module.h>
#include <linux/kernel.h>

MODULE_AUTHOR("Hao Li");
MODULE_DESCRIPTION("Hello world in a kernel module.");
MODULE_LICENSE("GPL");

static int __init mod_init (void) {
    printk(KERN_INFO "Hello, World!\n");
    return 0;
}
static void __exit mod_exit (void) {
    printk(KERN_INFO "Goodbye, World!\n");
}

module_init(mod_init);
module_exit(mod_exit);