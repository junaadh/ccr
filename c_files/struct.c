#include <stdio.h>
struct Ex {
  int index;
};

void printer(void *e) { printf("%d", ((struct Ex *)e)->index); }

int main() {
  struct Ex e = {10};
  printer(&e);
  return 0;
}
