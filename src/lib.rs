/*!
The `memchr` crate provides heavily optimized routines for searching bytes.

The `memchr` function is traditionally provided by libc, however, the
performance of `memchr` can vary significantly depending on the specific
implementation of libc that is used. They can range from manually tuned
Assembly implementations (like that found in GNU's libc) all the way to
non-vectorized C implementations (like that found in MUSL).

To smooth out the differences between implementations of libc, at least
on `x86_64` for Rust 1.27+, this crate provides its own implementation of
`memchr` that should perform competitively with the one found in GNU's libc.
The implementation is in pure Rust and has no dependency on a C compiler or an
Assembler.

Additionally, GNU libc also provides an extension, `memrchr`. This crate
provides its own implementation of `memrchr` as well, on top of `memchr2`,
`memchr3`, `memrchr2` and `memrchr3`. The difference between `memchr` and
`memchr2` is that that `memchr2` permits finding all occurrences of two bytes
instead of one. Similarly for `memchr3`.
*/

#![cfg_attr(not(feature = "use_std"), no_std)]

#![deny(missing_docs)]
#![doc(html_root_url = "https://docs.rs/memchr/2.0.0")]

// Supporting 16-bit would be fine. If you need it, please submit a bug report
// at https://github.com/BurntSushi/rust-memchr
#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error!("memchr currently not supported on non-32 or non-64 bit");

#[cfg(feature = "use_std")]
extern crate core;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

// N.B. If you're looking for the cfg knobs for libc, see build.rs.
#[cfg(memchr_libc)]
mod c;
#[allow(dead_code)]
mod fallback;
mod naive;
#[cfg(all(target_arch = "x86_64", memchr_runtime_simd))]
mod x86;
#[cfg(test)]
mod tests;

/// Search for the first occurrence of a byte in a slice.
///
/// This returns the index corresponding to the first occurrence of `needle` in
/// `haystack`, or `None` if one is not found.
///
/// While this is operationally the same as something like
/// `haystack.iter().position(|&b| b == needle)`, `memchr` will use a highly
/// optimized routine that can be up to an order of magnitude faster in some
/// cases.
///
/// # Example
///
/// This shows how to find the first position of a byte in a byte string.
///
/// ```
/// use memchr::memchr;
///
/// let haystack = b"the quick brown fox";
/// assert_eq!(memchr(b'k', haystack), Some(8));
/// ```
#[inline]
pub fn rawmemchr(needle: u8, haystack: *const u8) -> usize {
    #[cfg(all(target_arch = "x86_64", memchr_runtime_simd))]
    #[inline(always)]
    fn imp(n1: u8, haystack: *const u8) -> usize {
        x86::rawmemchr(n1, haystack)
    }

    #[cfg(not(all(target_arch = "x86_64", memchr_runtime_simd)))]
    #[inline(always)]
    fn imp(n1: u8, haystack: *const u8) -> usize {
        fallback::rawmemchr(n1, haystack)
    }

    imp(needle, haystack)
}

/// Like `memchr`, but searches for two bytes instead of one.
#[inline]
pub fn rawmemchr2(needle1: u8, needle2: u8, haystack: *const u8) -> usize {
    #[cfg(all(target_arch = "x86_64", memchr_runtime_simd))]
    #[inline(always)]
    fn imp(n1: u8, n2: u8, haystack: *const u8) -> usize {
        x86::rawmemchr2(n1, n2, haystack)
    }

    #[cfg(not(all(target_arch = "x86_64", memchr_runtime_simd)))]
    #[inline(always)]
    fn imp(n1: u8, n2: u8, haystack: *const u8) -> usize {
        fallback::rawmemchr2(n1, n2, haystack)
    }

    imp(needle1, needle2, haystack)
}

/// Like `memchr`, but searches for three bytes instead of one.
#[inline]
pub fn rawmemchr3(needle1: u8, needle2: u8, needle3: u8, haystack: *const u8) -> usize {
    #[cfg(all(target_arch = "x86_64", memchr_runtime_simd))]
    #[inline(always)]
    fn imp(n1: u8, n2: u8, n3: u8, haystack: *const u8) -> usize {
        x86::rawmemchr3(n1, n2, n3, haystack)
    }

    #[cfg(not(all(target_arch = "x86_64", memchr_runtime_simd)))]
    #[inline(always)]
    fn imp(n1: u8, n2: u8, n3: u8, haystack: *const u8) -> usize {
        fallback::rawmemchr3(n1, n2, n3, haystack)
    }

    imp(needle1, needle2, needle3, haystack)
}
