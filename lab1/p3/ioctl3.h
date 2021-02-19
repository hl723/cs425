#include <linux/ioctl.h>

// #define MAJOR 240
#define MYMEM_IOCTL_ALLOC       _IOWR(240, 0, int)
#define MYMEM_IOCTL_FREE        _IOW(240, 1, int)
#define MYMEM_IOCTL_SETREGION   _IOW(240, 2, int)
