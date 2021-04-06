#include <stdio.h>

int main(int argc, char **argv) {
    FILE *tp = fopen(argv[1], "w+");
    fprintf(tp, "Hello World!\n");
    fclose(tp);
    return 0;
}