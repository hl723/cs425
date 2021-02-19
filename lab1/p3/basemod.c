#include "basemod.h"
#include "ioctl3.h"

MODULE_AUTHOR("Hao Li");
MODULE_DESCRIPTION("Base module for memory allocation");
MODULE_LICENSE("GPL");

static int Major = 240;
static struct class *dev_class;

int my_get(char *buffer, const struct kernel_param *kp)
{   
    struct region *tmp;
    if (!dev)
        return 0;

    tmp = dev->head;
    while(tmp)
    {   
        sprintf(buffer + strlen(buffer), "%i: %i\n", tmp->id, tmp->size);
        tmp = tmp->next;
    }
	return strlen(buffer);
}
EXPORT_SYMBOL(my_get);

const struct kernel_param_ops param_ops_allocated = {
	.get	= param_get_int,
}; 
EXPORT_SYMBOL(param_ops_allocated);

const struct kernel_param_ops param_ops_regions = {
	.get	= my_get,
};
EXPORT_SYMBOL(param_ops_regions);
 
static int allocated = 0;
module_param_cb(allocated, &param_ops_allocated, &allocated, 0444);
static char *regions;
module_param_cb(regions, &param_ops_regions, &regions, 0444);

int mymem_open(struct inode *inode, struct file *fp);
int mymem_close(struct inode *inode, struct file *fp);
static ssize_t mymem_read(struct file *fp, char *buff, size_t count, loff_t *offp);
static ssize_t mymem_write(struct file *fp, const char *buff, size_t count, loff_t *offp);
loff_t mymem_llseek(struct file *file, loff_t offset, int whence);
long mymem_ioctl(struct file *file, unsigned int cmd, unsigned long arg);

EXPORT_SYMBOL(mymem_open);
EXPORT_SYMBOL(mymem_close);
EXPORT_SYMBOL(mymem_llseek);
EXPORT_SYMBOL(mymem_ioctl);


// initialize file_operations
static const struct file_operations mymem_fops = {
    .owner      = THIS_MODULE,
    .open       = mymem_open,
    .release    = mymem_close,
    .read       = mymem_read,
    .write      = mymem_write,
    .unlocked_ioctl = mymem_ioctl,
    .llseek     = mymem_llseek
};


static int __init basemod_init (void) 
{
    devno = MKDEV(Major, Minor);
    if (register_chrdev_region(devno, 1, "mymem"))
        if (DEBUG) printk(KERN_INFO "Failed to register character device.");

    if (DEBUG) printk("HI entering %i", Major);

    dev = kmalloc(sizeof(struct mymem_dev), GFP_KERNEL);

    cdev_init(&dev->my_cdev, &mymem_fops);
    dev->my_cdev.owner = THIS_MODULE;
    cdev_add(&dev->my_cdev, devno, 1);
    dev_class = class_create(THIS_MODULE, "mymem");
    device_create(dev_class, NULL, devno, NULL, "mymem");
    return 0;
}

static void __exit basemod_exit (void) 
{
    device_destroy(dev_class, devno);
    class_destroy(dev_class);
    cdev_del(&dev->my_cdev);
    unregister_chrdev_region(devno, 1);
}

int mymem_open(struct inode *inode, struct file *file)
{
    if (DEBUG) printk(KERN_INFO "Opening device file!");

    dev = container_of(inode->i_cdev, struct mymem_dev, my_cdev);
    dev->count = 0;
    dev->curr_region = NULL;
    dev->size_alloc = 0;
    dev->head = NULL;
    file->private_data = dev;
    // regions = kmalloc(sizeof(struct region *) * (++regions_alloc), GFP_KERNEL);
    return 0;
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
    if (curr->offset >= curr->size || size != 1)
    {
        if (DEBUG) printk(KERN_INFO "Error EINVAL in read!");
        return -EINVAL;
    }
    if(curr->offset + size > curr->size)
        size = curr->size - *offset;

    copy_to_user(buffer, curr->data + curr->offset, size);
    curr->offset += size;
    if (DEBUG) printk(KERN_INFO "Reading from device!");
    return size;
}

static ssize_t mymem_write(struct file *file, const char *buffer, size_t size, loff_t *offset)
{   
    struct region *curr;
    void *byte;
    dev = file->private_data; 
    curr = dev->curr_region;

    if (DEBUG) printk(KERN_INFO "Writing to device!");

    if (!curr)
    {
        if (DEBUG) printk(KERN_INFO "Error null in write!");
        return -1;
    }
        
    
    if (curr->offset >= curr->size || size != 1)
    {
        if (DEBUG) printk(KERN_INFO "Error EINVAL in write!");
        return -EINVAL;
    }
    if(curr->offset + size > curr->size)
        size = curr->size - curr->offset;

    byte = curr->data + curr->offset;

    copy_from_user(byte, buffer, size);
    curr->offset += size;
    
    if (DEBUG) printk(KERN_INFO "Writing to device!");
    return size;
}

int mymem_close(struct inode *inode, struct file *file)
{
    struct region *tmp;
    dev = file->private_data;

    if (DEBUG) printk(KERN_INFO "Closing device file!");
    
    while (dev->head)
    {
        if (dev->head->data)
            kfree(dev->head->data);
        tmp = dev->head;
        dev->head = dev->head->next;
        kfree(dev->head);
    }
    return 0;
}

loff_t mymem_llseek (struct file *file, loff_t offset, int whence)
{
    struct region *curr = dev->curr_region;
    dev = file->private_data;

    switch (whence)
    {
        case SEEK_END:
            curr->offset = curr->size;
            break;
        case SEEK_CUR:
            curr->offset += offset;
            break;
        case SEEK_SET:
            curr->offset = 0;
            break;
    }
    return curr->offset;
}

long mymem_ioctl (struct file *file, unsigned int cmd, unsigned long arg)
{
    int size, id;
    struct region *tmp, *prev;
    int *t;

    dev = file->private_data;
    
    if (cmd == MYMEM_IOCTL_ALLOC)
    {
        t = (int *) arg;
        copy_from_user(&size, t, sizeof(int));

        if (DEBUG) printk("GOT SIZE: %d", size);

        if (size + dev->size_alloc > SIZE_LIMIT)
            return -ENOMEM;
            
        tmp = kmalloc(sizeof(struct region *), GFP_KERNEL);
        tmp->data = kmalloc(size, GFP_KERNEL);
        tmp->size = size;
        tmp->offset = 0;
        tmp->id = dev->count;

        if (dev->head == NULL)
            tmp->next = NULL;
        else
            tmp->next = dev->head;
        
        dev->head = tmp;
        dev->curr_region = dev->head;
        dev->size_alloc += size;
        copy_to_user(t, &(dev->count), sizeof(int));
        arg = (long) t;
        dev->count++;

        allocated = dev->size_alloc;
        copy = dev;
    }
    else if (cmd == MYMEM_IOCTL_FREE)
    {
        t = (int *) arg;
        copy_from_user(&id, t, sizeof(int));

        tmp = dev->head;
        prev = NULL;
        while (tmp && tmp->id != id)
        {
            prev = tmp;
            tmp = tmp->next;
        }        
        
        if (!tmp)
        {
            if (DEBUG) printk("Something is wrong ID NOT FOUND.");
            return 0;
        }

        if (prev)
            prev->next = tmp->next;
        else
            dev->head = tmp->next;

        dev->size_alloc -= tmp->size;

        if (DEBUG) printk("FREEING ID: %i", tmp->id);
        kfree(tmp->data);
        kfree(tmp);
        allocated = dev->size_alloc;
        copy = dev;
    }
    else if (cmd == MYMEM_IOCTL_SETREGION)
    {
        t = (int *) arg;
        copy_from_user(&id, t, sizeof(int));
        tmp = dev->head;
        while (tmp->id != id)
            tmp = tmp->next;
        dev->curr_region = tmp;
        dev->curr_region->offset = 0;
    }
    return 0;
}

module_init(basemod_init);
module_exit(basemod_exit);
