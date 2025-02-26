// Run-time:
//   env-var: YKD_PRINT_IR=jit-pre-opt
//   env-var: YKD_SERIALISE_COMPILATION=1
//   env-var: YKD_PRINT_JITSTATE=1
//   stderr:
//     jit-state: start-tracing
//     i=3
//     jit-state: stop-tracing
//     --- Begin jit-pre-opt ---
//     ...
//       %{{cond}} = icmp...
//       br i1 %{{cond}}, label %{{succ-bb}}, label %guardfail
//
//     guardfail:...
//     ...
//     --- End jit-pre-opt ---
//     i=2
//     jit-state: enter-jit-code
//     i=1
//     jit-state: deoptimise
//     ...

// Check that tracing a non-default switch arm works correctly.

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <yk.h>
#include <yk_testing.h>

int main(int argc, char **argv) {
  YkMT *mt = yk_mt_new(NULL);
  yk_mt_hot_threshold_set(mt, 0);
  YkLocation loc = yk_location_new();
  int i = 3;
  int j = 300;
  NOOPT_VAL(i);
  NOOPT_VAL(j);
  while (i > 0) {
    yk_mt_control_point(mt, &loc);
    fprintf(stderr, "i=%d\n", i);
    switch (j) {
    case 100:
      i = 997;
    case 200:
      i = 998;
    case 300:
      i--;
      break;
    default:
      i = 999;
    }
  }
  yk_location_drop(loc);
  yk_mt_drop(mt);

  return (EXIT_SUCCESS);
}
