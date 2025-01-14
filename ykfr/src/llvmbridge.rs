use crate::SGValue;
use llvm_sys::core::*;
use llvm_sys::prelude::{LLVMBasicBlockRef, LLVMModuleRef, LLVMTypeRef, LLVMValueRef};
use llvm_sys::target::{LLVMGetModuleDataLayout, LLVMTargetDataRef};
use llvm_sys::{LLVMTypeKind, LLVMValueKind};
use std::ffi::CStr;

pub struct Module(LLVMModuleRef);

// Replicates struct of same name in `ykllvmwrap.cc`.
#[repr(C)]
pub struct BitcodeSection {
    data: *const u8,
    len: u64,
}

extern "C" {
    pub fn LLVMGetThreadSafeModule(bs: BitcodeSection) -> LLVMModuleRef;
}

impl Module {
    pub unsafe fn from_bc() -> Self {
        let (data, len) = ykutil::obj::llvmbc_section();
        let module = LLVMGetThreadSafeModule(BitcodeSection { data, len });
        Self(module)
    }

    pub fn function(&self, name: *const i8) -> Function {
        let func = unsafe { LLVMGetNamedFunction(self.0, name) };
        debug_assert!(!func.is_null());
        unsafe { Function::new(func) }
    }

    pub fn datalayout(&self) -> LLVMTargetDataRef {
        unsafe { LLVMGetModuleDataLayout(self.0) }
    }
}

pub struct Function(LLVMValueRef);

impl Function {
    pub unsafe fn new(func: LLVMValueRef) -> Self {
        debug_assert!(!LLVMIsAFunction(func).is_null());
        Self(func)
    }

    pub fn bb(&self, bbidx: usize) -> BasicBlock {
        let mut bb = unsafe { LLVMGetFirstBasicBlock(self.0) };
        for _ in 0..bbidx {
            bb = unsafe { LLVMGetNextBasicBlock(bb) };
        }
        unsafe { BasicBlock::new(bb) }
    }
}

pub struct BasicBlock(LLVMBasicBlockRef);

impl BasicBlock {
    pub unsafe fn new(bb: LLVMBasicBlockRef) -> Self {
        Self(bb)
    }

    pub fn instruction(&self, instridx: usize) -> Value {
        let mut instr = unsafe { LLVMGetFirstInstruction(self.0) };
        for _ in 0..instridx {
            instr = unsafe { LLVMGetNextInstruction(instr) };
        }
        unsafe { Value::new(instr) }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct Type(LLVMTypeRef);
impl Type {
    pub unsafe fn new(tref: LLVMTypeRef) -> Self {
        Type(tref)
    }

    pub fn get_element_type(&self) -> Self {
        unsafe { Type::new(LLVMGetElementType(self.0)) }
    }

    pub fn kind(&self) -> LLVMTypeKind {
        unsafe { LLVMGetTypeKind(self.0) }
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self.kind(), LLVMTypeKind::LLVMPointerTypeKind)
    }

    pub fn is_struct(&self) -> bool {
        matches!(self.kind(), LLVMTypeKind::LLVMStructTypeKind)
    }

    pub fn is_vector(&self) -> bool {
        matches!(self.kind(), LLVMTypeKind::LLVMVectorTypeKind)
    }

    pub fn is_integer(&self) -> bool {
        matches!(self.kind(), LLVMTypeKind::LLVMIntegerTypeKind)
    }

    pub fn is_void(&self) -> bool {
        matches!(self.kind(), LLVMTypeKind::LLVMVoidTypeKind)
    }

    pub fn get_int_width(&self) -> u32 {
        debug_assert!(self.is_integer());
        unsafe { LLVMGetIntTypeWidth(self.0) }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Value(LLVMValueRef);
impl Value {
    pub unsafe fn new(vref: LLVMValueRef) -> Self {
        Value(vref)
    }

    pub fn get(&self) -> LLVMValueRef {
        self.0
    }

    pub fn is_gep(&self) -> bool {
        unsafe { !LLVMIsAGetElementPtrInst(self.0).is_null() }
    }

    pub fn is_instruction(&self) -> bool {
        unsafe { !LLVMIsAInstruction(self.0).is_null() }
    }

    pub fn is_alloca(&self) -> bool {
        unsafe { !LLVMIsAAllocaInst(self.0).is_null() }
    }

    pub fn is_load(&self) -> bool {
        unsafe { !LLVMIsALoadInst(self.0).is_null() }
    }

    pub fn is_store(&self) -> bool {
        unsafe { !LLVMIsAStoreInst(self.0).is_null() }
    }

    pub fn is_call(&self) -> bool {
        unsafe { !LLVMIsACallInst(self.0).is_null() }
    }

    pub fn is_intrinsic(&self) -> bool {
        unsafe { !LLVMIsAIntrinsicInst(self.0).is_null() }
    }

    pub fn get_type(&self) -> Type {
        unsafe { Type(LLVMTypeOf(self.0)) }
    }

    pub fn get_operand(&self, idx: u32) -> Value {
        unsafe {
            debug_assert!(!LLVMIsAUser(self.0).is_null());
            Value(LLVMGetOperand(self.0, idx))
        }
    }

    pub fn as_str(&self) -> &CStr {
        unsafe { CStr::from_ptr(LLVMPrintValueToString(self.0)) }
    }

    pub fn kind(&self) -> LLVMValueKind {
        unsafe { LLVMGetValueKind(self.0) }
    }
}

pub fn llvm_const_to_sgvalue(c: Value) -> SGValue {
    let ty = c.get_type();
    match ty.kind() {
        LLVMTypeKind::LLVMIntegerTypeKind => {
            // FIXME: Add tests to check there's no silent sign extension going on.
            let val = unsafe { LLVMConstIntGetZExtValue(c.get()) };
            SGValue::new(val, ty)
        }
        LLVMTypeKind::LLVMPointerTypeKind => match c.kind() {
            LLVMValueKind::LLVMConstantPointerNullValueKind => SGValue::new(0, ty),
            _ => todo!(),
        },
        _ => todo!("{:?}", c.as_str()),
    }
}

/// Some live variables (e.g. pointers) have two representations: before they are copied into the
/// YKCtrlVars struct, and after. When initialising such a variable we need to assign to both
/// representations so that we can interpret IR outside of the main interpreter loop. This function
/// takes a live variable and tries to find its other representation if there is one.
pub unsafe fn get_aot_original(instr: &Value) -> Option<Value> {
    if instr.is_load() {
        let gep = instr.get_operand(0);
        if !gep.is_gep() {
            return None;
        }
        let ykcpvars = gep.get_operand(0);
        if !ykcpvars.is_alloca() {
            return None;
        }
        let ty = Type(LLVMGetAllocatedType(ykcpvars.0));
        if !ty.is_struct() {
            // If this isn't a struct it can't be YKCtrlPointVars.
            return None;
        }
        let name = CStr::from_ptr(LLVMGetStructName(ty.0));
        if name.to_str().unwrap() != "YkCtrlPointVars" {
            // This isn't the YKCtrlPointVars struct.
            return None;
        }
        // We found the YKCtrlPointVars struct. Now iterate over all it's uses to find the
        // corresponding store instruction from which we can extract the original AOT variable.
        let tgtoff = llvm_const_to_sgvalue(gep.get_operand(2));

        let mut varuse = LLVMGetFirstUse(ykcpvars.0);
        while !varuse.is_null() {
            let varuser = Value(LLVMGetUser(varuse));
            if varuser.is_gep() {
                let curoff = llvm_const_to_sgvalue(varuser.get_operand(2));
                if tgtoff == curoff {
                    let aotuse = LLVMGetFirstUse(varuser.0);
                    let aotstore = Value(LLVMGetUser(aotuse));
                    if aotstore.is_store() {
                        return Some(aotstore.get_operand(0));
                    }
                }
            }
            varuse = LLVMGetNextUse(varuse);
        }
    }
    None
}
