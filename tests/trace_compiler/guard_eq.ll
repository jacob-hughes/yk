; Run-time:
;   env-var: YKD_PRINT_IR=jit-pre-opt
;   env-var: YKT_TRACE_BBS=main:0,main:1
;   stderr:
;      --- Begin jit-pre-opt ---
;      ...
;      define {{type}} @__yk_compiled_trace_0(...
;      entry:
;        %{{0}} = alloca i32, align 4
;        %{{1}} = icmp eq i32 1, 1
;        br i1 %{{1}}, label %{{true}}, label %guardfail
;
;      guardfail:...
;        ...
;        %{{cprtn}} = call {{type}} (...) @llvm.experimental.deoptimize.p0(...
;        ret {{type}} %{{cprtn}}
;
;      {{true}}:...
;        store i32 1, ptr %{{0}}, align 4
;        ret {{type}} null
;      }
;
;      declare {{type}} @llvm.experimental.deoptimize.p0(...)
;      ...
;      --- End jit-pre-opt ---

define void @main() {
entry:
    %0 = alloca i32
    %1 = icmp eq i32 1, 1
    call void (i64, i32, ...) @llvm.experimental.stackmap(i64 1, i32 0, ptr %0, i1 %1)
    br i1 %1, label %true, label %false

true:
    store i32 1, ptr %0
    unreachable

false:
    store i32 0, ptr %0
    unreachable
}
declare void @llvm.experimental.stackmap(i64, i32, ...)
