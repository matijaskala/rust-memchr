// This code is unused. PCMPESTRI is gratuitously slow. I imagine it might
// start winning with a hypothetical memchr4 (or greater). This technique might
// also be good for exposing searches over ranges of bytes, but that departs
// from the standard memchr API, so it's not clear whether we actually want
// that or not.
//
// N.B. PCMPISTRI appears to be about twice as fast as PCMPESTRI, which is kind
// of neat. Unfortunately, UTF-8 strings can contain NUL bytes, which means
// I don't see a way of effectively using PCMPISTRI unless there's some fast
// way to replace zero bytes with a byte that is not not a needle byte.

use core::arch::x86_64::*;
use core::mem::size_of;

const VECTOR_SIZE: usize = size_of::<__m128i>();
const CONTROL_ANY: i32 =
    _SIDD_UBYTE_OPS
    | _SIDD_CMP_EQUAL_ANY
    | _SIDD_POSITIVE_POLARITY
    | _SIDD_LEAST_SIGNIFICANT;

#[target_feature(enable = "sse4.2")]
pub unsafe fn rawmemchr3(
    n1: u8, n2: u8, n3: u8,
    haystack: *const u8
) -> usize {
    let vn1 = _mm_set1_epi8(n1 as i8);
    let vn2 = _mm_set1_epi8(n2 as i8);
    let vn3 = _mm_set1_epi8(n3 as i8);
    let vn = _mm_setr_epi8(
        n1 as i8, n2 as i8, n3 as i8, 0,
        0, 0, 0, 0,
        0, 0, 0, 0,
        0, 0, 0, 0,
    );
    let start_ptr = haystack;
    let mut ptr = start_ptr;

    loop {
        let chunk = _mm_loadu_si128(ptr as *const __m128i);
        let res = _mm_cmpestri(vn, 3, chunk, 16, CONTROL_ANY);
        if res < 16 {
            return sub(ptr, start_ptr) + res as usize;
        }
        ptr = ptr.add(VECTOR_SIZE);
    }
}

/// Subtract `b` from `a` and return the difference. `a` should be greater than
/// or equal to `b`.
fn sub(a: *const u8, b: *const u8) -> usize {
    debug_assert!(a >= b);
    (a as usize) - (b as usize)
}
