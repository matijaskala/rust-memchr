// This module defines pure Rust platform independent implementations of all
// the memchr routines. We do our best to make them fast. Some of them may even
// get auto-vectorized.

use core::ptr;
use core::usize;

#[cfg(target_pointer_width = "32")]
const USIZE_BYTES: usize = 4;

#[cfg(target_pointer_width = "64")]
const USIZE_BYTES: usize = 8;

// The number of bytes to loop at in one iteration of memchr/memrchr.
const LOOP_SIZE: usize = 2 * USIZE_BYTES;

/// Return `true` if `x` contains any zero byte.
///
/// From *Matters Computational*, J. Arndt
///
/// "The idea is to subtract one from each of the bytes and then look for
/// bytes where the borrow propagated all the way to the most significant
/// bit."
#[inline(always)]
fn contains_zero_byte(x: usize) -> bool {
    const LO_U64: u64 = 0x0101010101010101;
    const HI_U64: u64 = 0x8080808080808080;

    const LO_USIZE: usize = LO_U64 as usize;
    const HI_USIZE: usize = HI_U64 as usize;

    x.wrapping_sub(LO_USIZE) & !x & HI_USIZE != 0
}

/// Repeat the given byte into a word size number. That is, every 8 bits
/// is equivalent to the given byte. For example, if `b` is `\x4E` or
/// `01001110` in binary, then the returned value on a 32-bit system would be:
/// `01001110_01001110_01001110_01001110`.
#[inline(always)]
fn repeat_byte(b: u8) -> usize {
    (b as usize) * (usize::MAX / 255)
}

pub fn rawmemchr(n1: u8, haystack: *const u8) -> usize {
    let vn1 = repeat_byte(n1);
    let confirm = |byte| byte == n1;
    let loop_size = LOOP_SIZE;
    let align = USIZE_BYTES - 1;
    let start_ptr = haystack;
    let mut ptr = start_ptr;

    unsafe {
        let chunk = read_unaligned_usize(ptr);
        if contains_zero_byte(chunk ^ vn1) {
            return forward_search(start_ptr, ptr, confirm);
        }

        ptr = ptr_add(ptr, USIZE_BYTES - (start_ptr as usize & align));
        debug_assert!(ptr > start_ptr);
        while loop_size == LOOP_SIZE {
            debug_assert_eq!(0, (ptr as usize) % USIZE_BYTES);

            let a = *(ptr as *const usize);
            let b = *(ptr_add(ptr, USIZE_BYTES) as *const usize);
            let eqa = contains_zero_byte(a ^ vn1);
            let eqb = contains_zero_byte(b ^ vn1);
            if eqa || eqb {
                break;
            }
            ptr = ptr_add(ptr, LOOP_SIZE);
        }
        forward_search(start_ptr, ptr, confirm)
    }
}

/// Like `memchr`, but searches for two bytes instead of one.
pub fn rawmemchr2(n1: u8, n2: u8, haystack: *const u8) -> usize {
    let vn1 = repeat_byte(n1);
    let vn2 = repeat_byte(n2);
    let confirm = |byte| byte == n1 || byte == n2;
    let align = USIZE_BYTES - 1;
    let start_ptr = haystack;
    let mut ptr = start_ptr;

    unsafe {
        let chunk = read_unaligned_usize(ptr);
        let eq1 = contains_zero_byte(chunk ^ vn1);
        let eq2 = contains_zero_byte(chunk ^ vn2);
        if eq1 || eq2 {
            return forward_search(start_ptr, ptr, confirm);
        }

        ptr = ptr_add(ptr, USIZE_BYTES - (start_ptr as usize & align));
        debug_assert!(ptr > start_ptr);
        loop {
            debug_assert_eq!(0, (ptr as usize) % USIZE_BYTES);

            let chunk = *(ptr as *const usize);
            let eq1 = contains_zero_byte(chunk ^ vn1);
            let eq2 = contains_zero_byte(chunk ^ vn2);
            if eq1 || eq2 {
                break;
            }
            ptr = ptr_add(ptr, USIZE_BYTES);
        }
        forward_search(start_ptr, ptr, confirm)
    }
}

/// Like `memchr`, but searches for three bytes instead of one.
pub fn rawmemchr3(n1: u8, n2: u8, n3: u8, haystack: *const u8) -> usize {
    let vn1 = repeat_byte(n1);
    let vn2 = repeat_byte(n2);
    let vn3 = repeat_byte(n3);
    let confirm = |byte| byte == n1 || byte == n2 || byte == n3;
    let align = USIZE_BYTES - 1;
    let start_ptr = haystack;
    let mut ptr = start_ptr;

    unsafe {
        let chunk = read_unaligned_usize(ptr);
        let eq1 = contains_zero_byte(chunk ^ vn1);
        let eq2 = contains_zero_byte(chunk ^ vn2);
        let eq3 = contains_zero_byte(chunk ^ vn3);
        if eq1 || eq2 || eq3 {
            return forward_search(start_ptr, ptr, confirm);
        }

        ptr = ptr_add(ptr, USIZE_BYTES - (start_ptr as usize & align));
        debug_assert!(ptr > start_ptr);
        loop {
            debug_assert_eq!(0, (ptr as usize) % USIZE_BYTES);

            let chunk = *(ptr as *const usize);
            let eq1 = contains_zero_byte(chunk ^ vn1);
            let eq2 = contains_zero_byte(chunk ^ vn2);
            let eq3 = contains_zero_byte(chunk ^ vn3);
            if eq1 || eq2 || eq3 {
                break;
            }
            ptr = ptr_add(ptr, USIZE_BYTES);
        }
        forward_search(start_ptr, ptr, confirm)
    }
}

#[inline(always)]
unsafe fn forward_search<F: Fn(u8) -> bool>(
    start_ptr: *const u8,
    mut ptr: *const u8,
    confirm: F,
) -> usize {
    debug_assert!(start_ptr <= ptr);

    loop {
        if confirm(*ptr) {
            return sub(ptr, start_ptr);
        }
        ptr = ptr.offset(1);
    }
}

/// Increment the given pointer by the given amount.
unsafe fn ptr_add(ptr: *const u8, amt: usize) -> *const u8 {
    debug_assert!(amt < ::core::isize::MAX as usize);
    ptr.offset(amt as isize)
}

/// Decrement the given pointer by the given amount.
unsafe fn ptr_sub(ptr: *const u8, amt: usize) -> *const u8 {
    debug_assert!(amt < ::core::isize::MAX as usize);
    ptr.offset((amt as isize).wrapping_neg())
}

unsafe fn read_unaligned_usize(ptr: *const u8) -> usize {
    let mut n: usize = 0;
    ptr::copy_nonoverlapping(ptr, &mut n as *mut _ as *mut u8, USIZE_BYTES);
    n
}

/// Subtract `b` from `a` and return the difference. `a` should be greater than
/// or equal to `b`.
fn sub(a: *const u8, b: *const u8) -> usize {
    debug_assert!(a >= b);
    (a as usize) - (b as usize)
}
