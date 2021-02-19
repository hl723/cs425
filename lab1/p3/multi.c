#include <pthread.h>
#include "headers.h"

static int NUM_THREADS = 1;
static int N = 1;
static int counter = 0;


void *thread_func(void *arg)
{
    int fd = *(int *) arg;
    unsigned long num;
    char buf[8];
    char *ptr;

    for (int i = 0; i < N; i++)
    {   
        while (counter != 0);
        counter++;
        lseek(fd, 0, SEEK_SET);
        read(fd, &buf, 8);
        num = strtoul(buf, &ptr, 10);
        num++;
        sprintf(buf, "%lu", num);
        lseek(fd, 0, SEEK_SET);
        write(fd, &buf, 8);
        counter--;
    }
    return 0;
}

int main(int argc, char **argv)
{
    if (argc > 1)
    {
        NUM_THREADS = atoi(argv[2]);
        N = atoi(argv[1]);
    }

    pthread_t threads[NUM_THREADS];
    int fd = open("/dev/mymem_smart", O_RDWR);
    int size = 20;
    int arg = size;
    char buf[8], *ptr;

    ioctl(fd, MYMEM_IOCTL_ALLOC, &arg);

    sprintf(buf, "%lu", (unsigned long) 0);
    lseek(fd, 0, SEEK_SET);
    write(fd, &buf, 8);
    
    for (int i = 0; i < NUM_THREADS; i++)
    {
        if (pthread_create(&threads[i], NULL, thread_func, &fd))
        {
            printf("Failed to create thread %i\n", i);
            return 1;
        }
    }

    for (int i = 0; i < NUM_THREADS; i++)
        pthread_join(threads[i], NULL);


    lseek(fd, 0, SEEK_SET);
    read(fd, &buf, 8);
    unsigned long num = strtoul(buf, &ptr, 10);
    unsigned long prod = (unsigned long)N*NUM_THREADS;
    unsigned long diff = prod - num;
    
    printf("result = %lu\n", num);
    printf("N = %d | W = %d | N * W = %lu\n", N, NUM_THREADS, prod);
    printf("diff: %lu\n", diff);

    ioctl(fd, MYMEM_IOCTL_FREE, &arg);
    close(fd);
}
