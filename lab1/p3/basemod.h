#include <linux/cdev.h>
#include <linux/export.h>
#include <linux/fs.h>
#include <linux/kdev_t.h>
#include <linux/init.h>

#include <linux/kernel.h>
#include <linux/kobject.h>
#include <linux/module.h>
#include <linux/slab.h>
#include <linux/types.h>
#include <linux/uaccess.h>


static int Minor = 0;
static int devno;

#define DEBUG (0)


static const int SIZE_LIMIT = 1048576; // 1MB

struct region {
  int size;
  int id;
  int offset;
  void *data;
  struct region *next;  
};

struct mymem_dev {
    int count;
    int size_alloc;
    struct region *curr_region;
    struct region *head;
    struct cdev my_cdev;
};

struct mymem_dev *dev;
struct mymem_dev *copy;
