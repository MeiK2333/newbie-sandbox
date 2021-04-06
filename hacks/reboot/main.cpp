#include <stdlib.h>
#include <stdio.h>
#include <unistd.h>
#include <sys/reboot.h>
#include <sys/syscall.h>

int main() {
    syscall(SYS_reboot, 0);
    return 0;
}