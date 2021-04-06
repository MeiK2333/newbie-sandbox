#include <unistd.h>
#include <stdio.h>

int main() {
    for (int i = 0; i < 10; i++) {
        pid_t pid = fork();
        printf("pid = %d\n", pid);
    }
    return 0;
}
