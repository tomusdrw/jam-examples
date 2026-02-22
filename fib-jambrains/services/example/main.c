#include "jb_service.h"
#include "jb_service_types.h"

#include <stdio.h>

void jb_hook_accumulate(jb_accumulate_arguments_t*) {
    // Calculate 10th Fibonacci number
    // F(0)=0, F(1)=1, F(2)=1, F(3)=2, ..., F(10)=55
    unsigned int a = 0, b = 1;
    for (int i = 2; i <= 10; i++) {
        unsigned int temp = a + b;
        a = b;
        b = temp;
    }
    printf("10th Fibonacci number: %u\n", b);
}

void jb_hook_refine(jb_refine_arguments_t*) {
    puts("Hello World from Refine");
}
