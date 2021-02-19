#include "headers.h"

int main()
{
    int fd = open("/dev/mymem_smart", O_RDWR);
    int size[5] = {1, 64, 1024, 65536, 1048576};
    int arg;
    char buf[1048576];

    double readtime[5], writetime[5];
    clock_t start, end;

    for (int i = 0; i < 5; i++)
    {
        readtime[i] = 0;
        writetime[i] = 0;
    }

    for (int i = 0; i < size[4]; i++)
        buf[i] = (char) (rand() % 256);

    arg = size[4];
    ioctl(fd, MYMEM_IOCTL_ALLOC, &arg);

    for (int j = 0; j < 5; j++)
    {
        for (int i = 0; i < 20; i++)
        {
            start = clock();
            for (int k = 0, num = size[4]/size[j]; k < num; k += size[j])
                write(fd, &buf + k, size[j]);
            end = clock();
            writetime[j] += (double) (end - start) / (CLOCKS_PER_SEC);
            
            lseek(fd, 0, SEEK_SET);

            start = clock();
            for (int k = 0, num = size[4]/size[j]; k < num; k += size[j])
                read(fd, &buf + k, size[j]);
            end = clock();
            readtime[j] += (double) (end - start) / (CLOCKS_PER_SEC);
            
            lseek(fd, 0, SEEK_SET);
        }
    }

    ioctl(fd, MYMEM_IOCTL_FREE, &arg);

    close(fd);

    for (int i = 0; i < 5; i++)
    {
        printf("Chunk Size: %7i\n", size[i]);
        printf("Avg Write Time: %6.8fs\n", writetime[i]/20);
        printf("Avg Read  Time: %6.8fs\n", readtime[i]/20);
    }
}
