# Minimal example

Create a *main.c* file which contains the following:

```c
#include <stdio.h>
#include <stdint.h>
#include <inttypes.h>

typedef struct arithmetic_parser arithmetic_parser_t;

extern arithmetic_parser_t *
arithmetic_parser_parse(const char *s);

extern void
arithmetic_parser_add_variable(arithmetic_parser_t *, const char *s, double d);

extern double
arithmetic_parser_execute(arithmetic_parser_t *);

extern void
arithmetic_parser_free(const arithmetic_parser_t *);

int main(void) {
  arithmetic_parser_t *parser = arithmetic_parser_parse("x + 3");
  arithmetic_parser_add_variable(parser, "x", 2);
  double result = arithmetic_parser_execute(parser);
  arithmetic_parser_free(parser);

  printf("Result: %f", result);
}
```

Compile it with

    gcc -c main.c -o main.o
    gcc -o main main.o -L/path/to/rust/library/so/folder/ -larithmetic_parser_wrap

Add the library to the *LD_LIBRARY_PATH*

    export LD_LIBRARY_PATH=/path/to/rust/library/so/folder/:$LD_LIBRARY_PATH

Run it

    ./main
