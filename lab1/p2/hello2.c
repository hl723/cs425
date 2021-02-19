#include <linux/init.h>
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/kobject.h>
#include <linux/moduleparam.h>

MODULE_AUTHOR("Hao Li");
MODULE_DESCRIPTION("Hello world in a kernel module with parameters.");
MODULE_LICENSE("GPL");


// sysfs entries
static int my_set(const char *val, const struct kernel_param *kp)
{
	int n = 0, ret;
 
	ret = kstrtoint(val, 10, &n);
	if (ret != 0)
		return -EINVAL;
 
    char buf[20];
    sprintf(buf, "%i", n*2);
	return param_set_int(buf, kp);
}

static const struct kernel_param_ops param_ops_logging = {
	.set	= param_set_int,
	.get	= param_get_int,
}; 

static const struct kernel_param_ops param_ops_double_me = {
	.set	= my_set,
	.get	= param_get_int,
};
 
static int enable_logging;
module_param_cb(enable_logging, &param_ops_logging, &enable_logging, 0664);
static int double_me;
module_param_cb(double_me, &param_ops_double_me, &double_me, 0664);


static int __init hello2_init (void) {
    if (enable_logging)
        printk(KERN_INFO "Hello, World!\n");

    return 0;
}

static void __exit hello2_exit (void) {
    if (enable_logging)
        printk(KERN_INFO "Goodbye, World!\n");
}

module_init(hello2_init);
module_exit(hello2_exit);
