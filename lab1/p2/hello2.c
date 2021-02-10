#include <linux/init.h>
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/kobject.h>
#include <linux/moduleparam.h>

MODULE_AUTHOR("Hao Li");
MODULE_DESCRIPTION("Hello world in a kernel module with parameters.");
MODULE_LICENSE("GPL");


// sysfs entries
// static int enable_logging = 0;
// module_param(enable_logging, int, 0644);
// static int double_me; 
// module_param(double_me, int, 0644);


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

// // define kernel object
// struct kobject *kobj_ref;

// // sysfs functions declarations
// static ssize_t show_enable_logging(struct kobject *kobj, struct kobj_attribute *attr, char *buf);
// static ssize_t store_enable_logging(struct kobject *kobj, struct kobj_attribute *attr, const char *buf, size_t count);
// static ssize_t show_double_me(struct kobject *kobj, struct kobj_attribute *attr, char *buf);
// static ssize_t store_double_me(struct kobject *kobj, struct kobj_attribute *attr, const char *buf, size_t count);


// struct kobj_attribute sysfs_enable_logging = __ATTR(enable_logging, 0660, show_enable_logging, store_enable_logging);
// struct kobj_attribute sysfs_double_me = __ATTR(double_me, 0660, show_double_me, store_double_me);


// // show and store functions 
// static ssize_t show_enable_logging(struct kobject *kobj, struct kobj_attribute *attr, char *buf)
// {
//     // printk(KERN_INFO "Reading in sysfs show function.\n");
//     return sprintf(buf, "%d", enable_logging);
// }

// static ssize_t store_enable_logging(struct kobject *kobj, struct kobj_attribute *attr, const char *buf, size_t count)
// {
//     sscanf(buf, "%d", &enable_logging);
//     // printk(KERN_INFO "Writing in sysfs store function %d\n", enable_logging);
//     return count;
// }

// static ssize_t show_double_me(struct kobject *kobj, struct kobj_attribute *attr, char *buf)
// {
//     if (enable_logging)
//         printk(KERN_INFO "Reading in sysfs show function.\n");
//     return sprintf(buf, "%d", double_me);
// }

// static ssize_t store_double_me(struct kobject *kobj, struct kobj_attribute *attr, const char *buf, size_t count)
// {   
//     int i = 0;
//     while (i < count - 1)
//     {
//         if (buf[i] < 48 || buf[i] > 57)
//             return -EINVAL;
//         i++;
//     }
//     sscanf(buf, "%d", &double_me);
    
//     // double the value
//     double_me *= 2;
//     if (enable_logging)
//         printk(KERN_INFO "Writing in sysfs store function %d\n", double_me);
//     return count;
// }


static int __init hello2_init (void) {

    // // create the sysfs directory and files
    // kobj_ref = kobject_create_and_add("my_sysfs", kernel_kobj);
    // if (sysfs_create_file(kobj_ref, &sysfs_enable_logging.attr))
    // {
    //     printk(KERN_INFO "Unable to create sysfs file\n");
    //     goto fail_sysfs;
    // }

    // if (sysfs_create_file(kobj_ref, &sysfs_double_me.attr))
    // {
    //     printk(KERN_INFO "Unable to create sysfs file\n");
    //     goto fail_sysfs;
    // }

    if (enable_logging)
        printk(KERN_INFO "Hello, World!\n");

    return 0;

// fail_sysfs:
//     kobject_put(kobj_ref);
//     sysfs_remove_file(kernel_kobj, &sysfs_enable_logging.attr);
//     sysfs_remove_file(kernel_kobj, &sysfs_double_me.attr);
//     return 0;
}

static void __exit hello2_exit (void) {
    if (enable_logging)
        printk(KERN_INFO "Goodbye, World!\n");

    // kobject_put(kobj_ref);
    // sysfs_remove_file(kernel_kobj, &sysfs_enable_logging.attr);
    // sysfs_remove_file(kernel_kobj, &sysfs_double_me.attr);
}

module_init(hello2_init);
module_exit(hello2_exit);
