//! Provides saving and restoring of architecture specific execution context.

use super::gdt::{USER_CODE_SEGMENT, USER_DATA_SEGMENT, TSS};
use super::interrupts::lapic;
use memory::{PhysicalAddress, VirtualAddress};
use memory::address_space::AddressSpace;
use multitasking::Stack;
use multitasking::scheduler::{after_context_switch, idle};
use x86_64::structures::idt::ExceptionStackFrame;

// TODO: Floating point state is not saved yet.
/// Saves the an execution context.
pub struct Context {
    pub kernel_stack_pointer: VirtualAddress,
    base_pointer: VirtualAddress,
    page_table_address: PhysicalAddress
}

impl Context {
    // TODO: Remove me, I'm only for testing.
    pub fn new(function: u64,
                stack_pointer: u64,
                mut kernel_stack_pointer: VirtualAddress,
                address_space: &mut AddressSpace)
                -> Context {
        use x86_64::registers::flags::Flags;

        let stack_frame = ExceptionStackFrame {
            instruction_pointer: ::x86_64::VirtualAddress(function as usize),
            code_segment: USER_CODE_SEGMENT.0 as u64,
            cpu_flags: (Flags::IF | Flags::A1).bits() as u64,
            stack_pointer: ::x86_64::VirtualAddress(stack_pointer as usize),
            stack_segment: USER_DATA_SEGMENT.0 as u64
        };

        set_initial_stack(&mut kernel_stack_pointer,
                          stack_frame,
                          address_space);

        Context {
            kernel_stack_pointer,
            base_pointer: kernel_stack_pointer,
            page_table_address: unsafe { address_space.get_page_table_address() }
        }
    }

    pub fn idle_context(stack_pointer: u64, page_table_address: PhysicalAddress) -> Context {
        let stack_pointer = unsafe { set_idle_stack(stack_pointer) };

        Context {
            kernel_stack_pointer: stack_pointer as usize,
            base_pointer: stack_pointer as usize,
            page_table_address
        }
    }
}

/// This is the first thing that's called by every new thread.
#[naked]
unsafe fn enter_thread() -> ! {
    after_context_switch();
    lapic::set_priority(0x0);
    asm!("xor r15, r15
          xor r14, r14
          xor r13, r13
          xor r12, r12
          xor r11, r11
          xor r10, r10
          xor r9, r9
          xor r8, r8
          xor rbp, rbp
          xor rdi, rdi
          xor rsi, rsi
          xor rdx, rdx
          xor rcx, rcx
          xor rbx, rbx
          xor rax, rax
          iretq" : : : : "intel", "volatile");
    unreachable!();
}

/// Sets the initial idle thread stack.
unsafe fn set_idle_stack(stack_pointer: u64) -> u64 {
    let mut stack_pointer = stack_pointer;
    stack_pointer -= 8;
    *(stack_pointer as *mut u64) = idle as u64;
    stack_pointer
}

/// Sets the initial kernel stack of a thread, so that it can properly start.
fn set_initial_stack(stack_pointer: &mut VirtualAddress,
                     stack_frame: ExceptionStackFrame,
                     address_space: &mut AddressSpace) {
    Stack::push_in(address_space, stack_pointer, stack_frame);
    Stack::push_in(address_space, stack_pointer, enter_thread as u64);
}

/// Switches the context from the old thread to the current thread.
///
/// # Safety
/// - To make sure that everything is properly cleaned up after switching the
/// context this should
/// only be called by the scheduler.
/// - Make sure preemption is disabled while calling this.
#[naked]
pub unsafe fn switch_context(old_context: &mut Context, new_context: &Context) {
    println!("");

    let new_sp = new_context.kernel_stack_pointer;
    let new_bp = new_context.base_pointer;
    let base_sp = ::multitasking::CURRENT_THREAD
        .lock()
        .kernel_stack
        .base_stack_pointer;
    TSS.as_mut().privilege_stack_table[0] = ::x86_64::VirtualAddress(base_sp);

    switch(&mut old_context.kernel_stack_pointer,
           &mut old_context.base_pointer,
           new_sp,
           new_bp,
           new_context.page_table_address);
}

/// This is the function actually performing the switch.
///
/// # Safety
/// - Should only be called by switch_context.
#[naked]
#[inline(never)]
unsafe extern "C" fn switch(old_sp: &mut usize,
                            old_bp: &mut usize,
                            new_sp: usize,
                            new_bp: usize,
                            new_page_table: usize) {
    asm!("mov [rdi], rsp
          mov [rsi], rbp
          mov rsp, rdx
          mov rbp, rcx
          mov cr3, r8"
          : :
          "{rdi}"(old_sp),
          "{rsi}"(old_bp),
          "{rdx}"(new_sp),
          "{rcx}"(new_bp),
          "{r8}"(new_page_table)
          : : "intel", "volatile");
}
