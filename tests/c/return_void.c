// Run-time:
//   env-var: YKD_PRINT_IR=aot,jit-pre-opt
//   env-var: YKD_SERIALISE_COMPILATION=1
//   env-var: YKD_PRINT_JITSTATE=1
//   stderr:
//     jit-state: start-tracing
//     jit-state: stop-tracing
//     ...
//     %{{1}} = call ptr @__ykrt_control_point(ptr %{{2}}, ptr %{{3}}, ptr %{{4}}, ptr %{{retval}})...
//     ...
//     define {{ty2}} @__yk_compiled_trace_0(ptr %0, ptr %1, i64 %2, ptr %3, ptr %4) {
//     ...
//     jit-state: enter-jit-code
//     ...
//     jit-state: deoptimise
//     ...
//     jit-state: exit-jit-code
//  stdout:
//     f
//     f
//     f
//     f
//     f

// Check that we can stopgap outside of nested, inlined calls.

#include <stdio.h>
#include <stdlib.h>
#include <yk.h>
#include <yk_testing.h>

__attribute__((noinline)) void f() {
  YkMT *mt = yk_mt_new(NULL);
  yk_mt_hot_threshold_set(mt, 0);
  YkLocation loc = yk_location_new();

  int i = 5;
  NOOPT_VAL(i);
  while (i > 0) {
    yk_mt_control_point(mt, &loc);
    printf("f\n");
    i--;
  }

  yk_location_drop(loc);
  yk_mt_drop(mt);
}

int main(int argc, char **argv) {
  f();
  return (EXIT_SUCCESS);
}
