#![no_std]
#![feature(llvm_asm)]

pub mod interrupt;

#[macro_use]
mod macros;

/// Move the vector base
#[inline]
pub unsafe fn set_vecbase(base: *const u32) {
    llvm_asm!("wsr.vecbase $0" ::"r"(base) :: "volatile");
}

/// Get the core cycle count
#[inline]
pub fn get_cycle_count() -> u32 {
    let x: u32;
    unsafe { llvm_asm!("rsr.ccount $0" : "=r"(x) ::: "volatile" ) };
    x
}

/// Cycle count compare register
pub enum CycleCompare {
    Register0,
    Register1,
    Register2,
}

/// Set cycle count compare
#[inline]
pub fn set_cycle_compare(compare_register: CycleCompare, value: u32) {
    unsafe {
        match compare_register {
            CycleCompare::Register0 => {
                llvm_asm!("wsr.ccompare0 $0; isync" ::"r"(value):: "volatile" )
            }
            CycleCompare::Register1 => {
                llvm_asm!("wsr.ccompare1 $0; isync" ::"r"(value):: "volatile" )
            }
            CycleCompare::Register2 => {
                llvm_asm!("wsr.ccompare2 $0; isync" ::"r"(value):: "volatile" )
            }
        }
    };
}

/// Get cycle count compare
#[inline]
pub fn get_cycle_compare(compare_register: CycleCompare) -> u32 {
    let x;
    unsafe {
        match compare_register {
            CycleCompare::Register0 => llvm_asm!("rsr.ccompare0 $0" :"=r"(x)::: "volatile" ),
            CycleCompare::Register1 => llvm_asm!("rsr.ccompare1 $0" :"=r"(x)::: "volatile" ),
            CycleCompare::Register2 => llvm_asm!("rsr.ccompare2 $0" :"=r"(x)::: "volatile" ),
        }
    };
    x
}

/// Get the core stack pointer
#[inline(always)]
pub fn get_stack_pointer() -> *const u32 {
    let x: *const u32;
    unsafe { llvm_asm!("mov $0,sp" : "=r"(x) ::: "volatile") };
    x
}

/// Set the core stack pointer
///
/// *This is highly unsafe!*
/// It should be used with care at e.g. program start or when building a task scheduler
///
/// `stack` pointer to the non-inclusive end of the stack (must be 16-byte aligned)
#[inline(always)]
pub unsafe fn set_stack_pointer(stack: *mut u32) {
    llvm_asm!("
        movi a0,0
        mov sp,$0
        " :: "r"(stack):"a0" ::: "volatile" );
}

/// Get the core current program counter
#[inline(always)]
pub fn get_program_counter() -> *const u32 {
    let x: *const u32;
    let _y: u32;
    unsafe {
        llvm_asm!("
            mov $1,a0
            call0 1f
            .align 4
            1: 
            mov $0,a0
            mov a0,$1
            " : "=r"(x),"=r"(_y)::"a0" : "volatile" )
    };
    x
}

/// cycle accurate delay using the cycle counter register
#[inline]
pub fn delay(clocks: u32) {
    let start = get_cycle_count();
    loop {
        if get_cycle_count().wrapping_sub(start) >= clocks {
            break;
        }
    }
}

/// Get the id of the current core
#[inline]
pub fn get_processor_id() -> u32 {
    let mut x: u32;
    unsafe { llvm_asm!("rsr.prid $0" : "=r"(x) ::: "volatile") };
    x
}

const XDM_OCD_DCR_SET: u32 = 0x10200C;
const DCR_ENABLEOCD: u32 = 0x01;

/// Returns true if a debugger is attached
#[inline]
pub fn is_debugger_attached() -> bool {
    let mut x: u32;
    unsafe { llvm_asm!("rer $0,$1" : "=r"(x): "r"(XDM_OCD_DCR_SET) :: "volatile" ) };
    (x & DCR_ENABLEOCD) != 0
}

/// Insert debug breakpoint
#[inline(always)]
pub fn debug_break() {
    unsafe { llvm_asm!("break 1,15"::::"volatile") };
}
