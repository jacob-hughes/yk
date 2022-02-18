// Compiler:
//   env-var: YKD_PRINT_JITSTATE=1
// Run-time:
//   env-var: YKD_PRINT_IR=jit-pre-opt
//   env-var: YKD_SERIALISE_COMPILATION=1
//   stderr:
//     jit-state: start-tracing
//     jit-state: stop-tracing
//     --- Begin jit-pre-opt ---
//     ...
//     define internal void @__yk_compiled_trace_0(%YkCtrlPointVars* %0, i64* %1, i64 %2) {
//        ...
//     }
//     ...
//     --- End jit-pre-opt ---
//     jit-state: enter-jit-code
//     ...
//     jit-state: stopgap
//     ...
//     Indirect: 997...
//     ...

// Check that basic trace compilation works.

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <yk.h>
#include <yk_testing.h>

int main(int argc, char **argv) {
  int res = 0;
  int src = 1000;
  YkMT *mt = yk_mt_new();
  yk_set_hot_threshold(mt, 0);
  YkLocation loc = yk_location_new();
  int i = 3;
  NOOPT_VAL(res);
  NOOPT_VAL(i);
  NOOPT_VAL(src);
  while (i > 0) {
    yk_control_point(mt, &loc);
    memcpy(&res, &src, 4);
    src--;
    i--;
  }
  NOOPT_VAL(res);
  assert(res == 996);
  yk_location_drop(loc);
  yk_mt_drop(mt);

  return (EXIT_SUCCESS);
}