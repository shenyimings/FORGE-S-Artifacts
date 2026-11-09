use crate::processor::process_instruction;

#[cfg(all(not(test), feature = "custom-heap", target_os = "solana"))]
use {
    solana_program::msg,
    std::{alloc::Layout, mem::size_of, ptr::null_mut, usize},
};

/// The bump allocator used as the default rust heap when running programs.
pub struct MyBumpAllocator {
    pub start: usize,
    pub len: usize,
}

/// Integer arithmetic in this global allocator implementation is safe when
/// operating on the prescribed `HEAP_START_ADDRESS` and `HEAP_LENGTH`. Any
/// other use may overflow and is thus unsupported and at one's own risk.
#[cfg(all(not(test), feature = "custom-heap", target_os = "solana"))]
#[allow(clippy::arithmetic_side_effects)]
unsafe impl std::alloc::GlobalAlloc for MyBumpAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pos_ptr = self.start as *mut usize;

        let mut pos = *pos_ptr;
        if pos == 0 {
            // First time, set starting position
            pos = self.start + self.len;
        }
        pos = pos.saturating_sub(layout.size());
        pos &= !(layout.align().wrapping_sub(1));
        if pos < self.start + size_of::<*mut u8>() {
            return null_mut();
        }
        *pos_ptr = pos;
        pos as *mut u8
    }

    #[inline]
    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        // I'm a bump allocator, I don't free
    }
}

impl MyBumpAllocator {
    /// Return heap position as of this call
    pub unsafe fn pos(&self) -> usize {
        let pos_ptr = self.start as *mut usize;
        *pos_ptr
    }

    /// Reset heap start cursor to position.
    /// ### This is very unsafe, use wisely
    pub unsafe fn move_cursor(&self, pos: usize) {
        let pos_ptr = self.start as *mut usize;
        *pos_ptr = pos;
    }
}

#[cfg(all(not(test), feature = "custom-heap", target_os = "solana"))]
#[global_allocator]
pub static A: MyBumpAllocator = MyBumpAllocator {
    start: solana_program::entrypoint::HEAP_START_ADDRESS as usize,
    len: solana_program::entrypoint::HEAP_LENGTH,
};

#[doc = r" # Safety"]
#[no_mangle]
pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
    let (program_id, accounts, instruction_data) =
        unsafe { solana_program::entrypoint::deserialize(input) };

    match process_instruction(&program_id, &accounts, &instruction_data) {
        Ok(()) => solana_program::entrypoint::SUCCESS,
        Err(error) => error.into(),
    }
}

#[cfg(all(not(feature = "custom-heap"), target_os = "solana"))]
#[global_allocator]
static A: solana_program::entrypoint::BumpAllocator = solana_program::entrypoint::BumpAllocator {
    start: solana_program::entrypoint::HEAP_START_ADDRESS as usize,
    len: solana_program::entrypoint::HEAP_LENGTH,
};

#[cfg(all(not(feature = "custom-panic"), target_os = "solana"))]
#[no_mangle]
fn custom_panic(info: &core::panic::PanicInfo<'_>) {
    // Full panic reporting
    msg!("{}", info);
}
