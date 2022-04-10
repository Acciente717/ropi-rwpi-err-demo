//! This file is derived from `cortex-m-rt` crate, with `Reset`
//! function modified.

use core::arch::asm;

#[link_section = ".vector_table.reset_vector"]
#[no_mangle]
pub static __RESET_VECTOR: unsafe extern "C" fn() -> ! = Reset;

/// Registers stacked (pushed into the stack) during an exception
#[derive(Clone, Copy)]
#[repr(C)]
pub struct ExceptionFrame {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r12: u32,
    pub lr: u32,
    pub pc: u32,
    pub xpsr: u32,
}

extern "C" {
    // These symbols come from `link.ld`
    static mut __sbss: u32;
    static mut __ebss: u32;
    static mut __sdata: u32;
    static mut __edata: u32;
    static __sidata: u32;
}

#[link_section = ".Reset"]
#[allow(non_snake_case)]
#[export_name = "Reset"]
#[naked]
unsafe extern "C" fn Reset() -> ! {
    asm!(
        "ldr r9, ={sdata}",
        "ldr r0, ={sbss}",
        "ldr r1, ={ebss}",
        "sub r1, r1, r0",
        "bl  {memclr}",
        "ldr r0, ={sdata}",
        "ldr r1, ={sidata}",
        "ldr r2, ={edata}",
        "sub r2, r2, r0",
        "bl  {memcpy}",
        "mov lr, #0",
        "b  main",
        sbss = sym __sbss,
        ebss = sym __ebss,
        sdata = sym __sdata,
        edata = sym __edata,
        sidata = sym __sidata,
        memclr = sym r_memclr,
        memcpy = sym r_memcpy,
        options(noreturn)
    );
}

unsafe extern "C" fn r_memcpy(mut dst: *mut u8, mut src: *const u8, mut len: usize) {
    while len > 0 {
        *dst = *src;
        dst = dst.offset(1);
        src = src.offset(1);
        len -= 1;
    }
}

unsafe extern "C" fn r_memclr(mut dst: *mut u8, mut len: usize) {
    while len > 0 {
        *dst = 0;
        dst = dst.offset(1);
        len -= 1;
    }
}

pub union Vector {
    handler: unsafe extern "C" fn(),
    reserved: usize,
}

#[link_section = ".vector_table.exceptions"]
#[no_mangle]
pub static __EXCEPTIONS: [Vector; 14] = [
    // Exception 2: Non Maskable Interrupt.
    Vector {
        handler: NonMaskableInt,
    },
    // Exception 3: Hard Fault Interrupt.
    Vector {
        handler: HardFaultTrampoline,
    },
    // Exception 4: Memory Management Interrupt [not on Cortex-M0 variants].
    #[cfg(not(armv6m))]
    Vector {
        handler: MemoryManagement,
    },
    #[cfg(armv6m)]
    Vector { reserved: 0 },
    // Exception 5: Bus Fault Interrupt [not on Cortex-M0 variants].
    #[cfg(not(armv6m))]
    Vector { handler: BusFault },
    #[cfg(armv6m)]
    Vector { reserved: 0 },
    // Exception 6: Usage Fault Interrupt [not on Cortex-M0 variants].
    #[cfg(not(armv6m))]
    Vector {
        handler: UsageFault,
    },
    #[cfg(armv6m)]
    Vector { reserved: 0 },
    // Exception 7: Secure Fault Interrupt [only on Armv8-M].
    #[cfg(armv8m)]
    Vector {
        handler: SecureFault,
    },
    #[cfg(not(armv8m))]
    Vector { reserved: 0 },
    // 8-10: Reserved
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    // Exception 11: SV Call Interrupt.
    Vector { handler: SVCall },
    // Exception 12: Debug Monitor Interrupt [not on Cortex-M0 variants].
    #[cfg(not(armv6m))]
    Vector {
        handler: DebugMonitor,
    },
    #[cfg(armv6m)]
    Vector { reserved: 0 },
    // 13: Reserved
    Vector { reserved: 0 },
    // Exception 14: Pend SV Interrupt [not on Cortex-M0 variants].
    Vector { handler: PendSV },
    // Exception 15: System Tick Interrupt.
    Vector { handler: SysTick },
];

extern "C" {
    fn NonMaskableInt();

    #[cfg(not(armv6m))]
    fn MemoryManagement();

    #[cfg(not(armv6m))]
    fn BusFault();

    #[cfg(not(armv6m))]
    fn UsageFault();

    #[cfg(armv8m)]
    fn SecureFault();

    fn SVCall();

    #[cfg(not(armv6m))]
    fn DebugMonitor();

    fn PendSV();

    fn SysTick();
}

#[link_section = ".vector_table.interrupts"]
#[no_mangle]
pub static __INTERRUPTS: [unsafe extern "C" fn(); 240] = [{
    extern "C" {
        fn DefaultHandler();
    }

    DefaultHandler
}; 240];

#[allow(non_snake_case)]
#[export_name = "HardFaultTrampoline"]
pub extern "C" fn HardFaultTrampoline() {
    unsafe {
        asm!(
            "mov r0, lr",
            "mov r1, #4",
            "tst r0, r1",
            "bne 0f",
            "mrs r0, MSP",
            "b {hardfault_handler}",
            "0:",
            "mrs r0, PSP",
            "b {hardfault_handler}",
            hardfault_handler = sym hardfault_handler
        )
    }
}

#[allow(non_snake_case)]
#[export_name = "DefaultHandler_"]
pub extern "C" fn DefaultHandler_() {
    loop {}
}

#[export_name = "HardFault"]
pub unsafe extern "C" fn hardfault_handler(_ef: &ExceptionFrame) -> ! {
    loop {}
}

#[export_name = "DefaultHandler"]
pub unsafe extern "C" fn default_handler(_ex_num: i16) {
    loop {}
}
