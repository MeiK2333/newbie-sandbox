#include <stdio.h>

int main() {
    int i = 0;
    while (1) {
        i++;
        printf("0123456789");
        if (i < 0) {
            break;
        }
    }
    return 0;
}