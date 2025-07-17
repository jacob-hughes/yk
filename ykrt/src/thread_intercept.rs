use std::os::raw::{c_int, c_void};

use libc::{dlsym, free, malloc, pthread_create};
use std::mem::MaybeUninit;
use std::{ffi::CString, ptr, ptr::null_mut};

mod shadow_stack {
    use std::sync::Mutex;

    // The size of the shadow stack. This is the same size as the default shadow stack in ykllvm.
    const SHADOW_STACK_SIZE: usize = 1000000;

    static SHADOW_STACKS: Mutex<Vec<ShadowStack>> = Mutex::new(Vec::new());
}

type ShadowStackPtr = *mut MaybeUninit<u8>;

struct ShadowStacks {
    stacks: Mutex<Vec<ShadowStackPtr>>,
}

impl ShadowStacks {
    fn new() -> Self {
        ShadowStacks {
            stacks: Mutex::new(Vec::new()),
        }
    }

    fn init(&mut self) {
        let head = CString::new("shadowstack_head").unwrap();
        let head_ptr = unsafe { dlsym(null_mut(), head.as_ptr()) as ShadowStackPtr };
        assert!(!head_ptr.is_null());
        self.stacks.get_mut().unwrap().push(head_ptr)
    }

    fn insert(ptr: ShadowStackPtr) {}

    fn register_current_thread() {}
}

#[derive(Debug)]
struct Target {
    pub func: extern "C" fn(*mut c_void) -> *mut c_void,
    pub arg: *mut c_void,
}

pub fn register_shadow_stack(ss_ptr: *mut MaybeUninit<u8>) {
    unsafe {
        SHADOW_STACKS[NEXT_SHADOW_STACK] = ss_ptr;
        NEXT_SHADOW_STACK += 1;
        assert!(NEXT_SHADOW_STACK < 10);
    }
}

pub fn yk_foreach_shadowstack(f: extern "C" fn(*mut c_void, *mut c_void)) {
    for ptr in SHADOW_STACKS.into_iter() {
        let end = ptr.wrapping_byte_add(SHADOW_STACK_SIZE);
        unsafe { f(ptr as *mut c_void, end as *mut c_void) };
    }
}

pub fn yk_init() {
    let head = CString::new("shadowstack_head").unwrap();
    let head_ptr = unsafe { dlsym(null_mut(), head.as_ptr()) as *mut MaybeUninit<u8> };
    assert!(!head_ptr.is_null());
    register_shadow_stack(head_ptr);
    eprintln!("Initializing");
}

pub fn yk_pthread_create(
    thread: *mut libc::pthread_t,
    attr: *const libc::pthread_attr_t,
    f: extern "C" fn(*mut c_void) -> *mut c_void,
    value: *mut c_void,
) -> c_int {
    let head = CString::new("shadowstack_head").unwrap();
    let curr = CString::new("shadowstack_0").unwrap();

    let head_ptr = unsafe { dlsym(null_mut(), head.as_ptr()) as ShadowStackPtr };
    let curr_ptr = unsafe { dlsym(null_mut(), curr.as_ptr()) as ShadowStackPtr };

    assert!(!head_ptr.is_null());
    assert!(!curr_ptr.is_null());

    ShadowStacks::register_current_thread();

    let s = Box::<[u8]>::new_uninit_slice(SHADOW_STACK_SIZE);
    unsafe {
        ptr::write(head_ptr, s[0]);
        ptr::write(curr_ptr, s[0]);
        libc::pthread_create(thread, attr, f, value)
    }
}

extern "C" fn wrap_thread_routine(tgt: *mut c_void) -> *mut c_void {
    let str = CString::new("shadowstack_0").unwrap();
    let tgt = unsafe { Box::from_raw(tgt as *mut Target) };
    // Obtain address of a shadowstack_0 symbol
    let shadowstack_symbol_addr = unsafe { dlsym(null_mut(), str.as_ptr()) };
    if shadowstack_symbol_addr.is_null() {
        panic!("Unable to find shadowstack address")
    }
    let newsstack = unsafe { malloc(SHADOW_STACK_SIZE) };
    if newsstack.is_null() {
        panic!("Unable to allocate stack")
    }
    unsafe {
        // Set shadowstack symbol with new allocated stack
        *(shadowstack_symbol_addr as *mut *mut c_void) = newsstack;
    }
    let ret = (tgt.func)(tgt.arg);
    unsafe { free(newsstack) };
    ret
}

#[no_mangle]
pub extern "C" fn __wrap_pthread_create(
    thread: *mut libc::pthread_t,
    attr: *const libc::pthread_attr_t,
    start_routine: extern "C" fn(*mut c_void) -> *mut c_void,
    arg: *mut c_void,
) -> c_int {
    let tgt = Box::new(Target {
        func: start_routine,
        arg,
    });
    unsafe {
        pthread_create(
            thread,
            attr,
            wrap_thread_routine,
            Box::into_raw(tgt) as *mut c_void,
        )
    }
}

#[no_mangle]
pub extern "C" fn __wrap_pthread_exit(_retval: *mut c_void) {
    // FIXME: Using `pthread_exit` doesn't return to `wrap_thread_routine` and thus doesn't free
    // the newly created shadowstack.
    todo!("No support for `pthread_exit` yet.");
}
