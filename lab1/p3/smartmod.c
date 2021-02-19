#include "basemod.h"
#include "ioctl3.h"

MODULE_AUTHOR("Hao Li");
MODULE_DESCRIPTION("Smart module for memory allocation");
MODULE_LICENSE("GPL");

static int Major = 241;
static struct class *dev_class;

// extern int my_get(char *buffer, const struct kernel_param *kp);
extern const struct kernel_param_ops param_ops_allocated;
extern const struct kernel_param_ops param_ops_regions;

static int allocated = 0;
module_param_cb(allocated, &param_ops_allocated, &allocated, 0444);
static char *regions;
module_param_cb(regions, &param_ops_regions, &regions, 0444);

extern int mymem_open(struct inode *inode, struct file *fp);
extern int mymem_close(struct inode *inode, struct file *fp);
static ssize_t mymem_read(struct file *fp, char *buff, size_t count, loff_t *offp);
static ssize_t mymem_write(struct file *fp, const char *buff, size_t count, loff_t *offp);
extern loff_t mymem_llseek(struct file *file, loff_t offset, int whence);
extern long mymem_ioctl(struct file *file, unsigned int cmd, unsigned long arg);

// initialize file_operations
const struct file_operations mymem_fops = {
    .owner      = THIS_MODULE,
    .open       = mymem_open,
    .release    = mymem_close,
    .read       = mymem_read,
    .write      = mymem_write,
    .unlocked_ioctl = mymem_ioctl,
    .llseek     = mymem_llseek
};


static int __init smartmod_init (void) 
{
    devno = MKDEV(Major, Minor);
    if (register_chrdev_region(devno, 1, "mymem_smart"))
        if (DEBUG) printk(KERN_INFO "Failed to register character device.");

    if (DEBUG) printk("HI entering %i", Major);

    dev = kmalloc(sizeof(struct mymem_dev), GFP_KERNEL);

    cdev_init(&dev->my_cdev, &mymem_fops);
    dev->my_cdev.owner = THIS_MODULE;
    cdev_add(&dev->my_cdev, devno, 1);
    dev_class = class_create(THIS_MODULE, "mymem_smart");
    device_create(dev_class, NULL, devno, NULL, "mymem_smart");
    return 0;
}

static void __exit smartmod_exit (void) 
{
    device_destroy(dev_class, devno);
    class_destroy(dev_class);
    cdev_del(&dev->my_cdev);
    unregister_chrdev_region(devno, 1);
}


static ssize_t mymem_read(struct file *file, char *buffer, size_t size, loff_t *offset)
{
    struct region *curr;
    dev = file->private_data; 
    curr = dev->curr_region;
    
    if (DEBUG) printk(KERN_INFO "Reading from device!");

    if (!curr)
    {
        if (DEBUG) printk(KERN_INFO "Error NULL in read!");
        return -1;
    }
    if (curr->offset >= curr->size)
    {
        if (DEBUG) printk(KERN_INFO "Error EINVAL in read!");
        return -EINVAL;
    }
    if(curr->offset + size > curr->size)
        size = curr->size - curr->offset;
    copy_to_user(buffer, curr->data + curr->offset, size);
    curr->offset += size;
    if (DEBUG) printk(KERN_INFO "Reading from device!");
    return size;
}

static ssize_t mymem_write(struct file *file, const char *buffer, size_t size, loff_t *offset)
{   
    struct region *curr;
    // void *byte;
    dev = file->private_data; 
    curr = dev->curr_region;

    if (DEBUG) printk(KERN_INFO "Writing to device!");

    if (!curr)
    {
        if (DEBUG) printk(KERN_INFO "Error null in write!");
        return -1;
    }
        
    
    if (curr->offset >= curr->size)
    {
        if (DEBUG) printk(KERN_INFO "Error EINVAL in write!");
        return -EINVAL;
    }
    if(curr->offset + size > curr->size)
        size = curr->size - curr->offset;

    // byte = ;

    copy_from_user(curr->data + curr->offset, buffer, size);
    curr->offset += size;
    
    if (DEBUG) printk(KERN_INFO "Writing to device!");
    return size;
}

module_init(smartmod_init);
module_exit(smartmod_exit);
