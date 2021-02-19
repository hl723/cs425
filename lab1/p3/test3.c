#include "headers.h"

int main()
{
    int fd = open("/dev/mymem", O_RDWR);
    int size = 524288;
    int arg;;
    char buf[1];

    double readtime = 0, writetime = 0;
    clock_t start, end;

    arg = size;
    ioctl(fd, MYMEM_IOCTL_ALLOC, &arg);

    for (int i = 0; i < 20; i++)
    {
        start = clock();
        for (int j = 0; j < size; j++)
        {
            buf[0] = (char) (rand() % 256);
            write(fd, buf, 1);
        }
        end = clock();
        writetime += (double) (end - start) / (CLOCKS_PER_SEC);
        
        lseek(fd, 0, SEEK_SET);

        start = clock();
        for (int j = 0; j < size; j++)
            read(fd, &buf, 1);
        end = clock();
        
        readtime += (double) (end - start) / (CLOCKS_PER_SEC);
        
        lseek(fd, 0, SEEK_SET);
    }
    ioctl(fd, MYMEM_IOCTL_FREE, &arg);

    close(fd);

    printf("Avg Write time: %6.3fs\n", writetime/20);
    printf("Avg Read time: %6.3fs\n", readtime/20);

	// int fd=0;
    // char buf[1];

    // printf("%li, %li, %li\n", MYMEM_IOCTL_ALLOC, MYMEM_IOCTL_FREE, MYMEM_IOCTL_SETREGION);
	
	// fd=open("/dev/mymem",O_RDWR);
	
	// printf("fd :%d\n",fd);

    // // close(fd);

    // int arg1 = 10, arg2 = 5;

    // ioctl(fd, MYMEM_IOCTL_ALLOC, &arg1);
    // ioctl(fd, MYMEM_IOCTL_ALLOC, &arg2);

    // printf("Got regions %i and %i\n", arg1, arg2);
    
    // char *rand = "c";

    // write(fd, rand, 1);

    // lseek(fd, 0, SEEK_SET);

    // read(fd, &buf, 1);

    // printf("Read: %s\n", buf);

    

    // ioctl(fd, MYMEM_IOCTL_SETREGION, &arg1);
    
    // write(fd, "h", 1);

    // lseek(fd, 0, SEEK_SET);

    // read(fd, buf, 1);

    // printf("Read: %s\n", buf);

    // ioctl(fd, MYMEM_IOCTL_FREE, &arg1);
    // ioctl(fd, MYMEM_IOCTL_FREE, &arg2);

    // int overflow = 1048576+1;
    // ioctl(fd, MYMEM_IOCTL_ALLOC, &overflow);
	
	// close(fd);
}
