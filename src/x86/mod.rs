use fallback;

// We only use AVX when we can detect at runtime whether it's available, which
// requires std.
#[cfg(feature = "use_std")]
mod avx;
mod sse2;

// This macro employs a gcc-like "ifunc" trick where by upon first calling
// `memchr` (for example), CPU feature detection will be performed at runtime
// to determine the best implementation to use. After CPU feature detection
// is done, we replace `memchr`'s function pointer with the selection. Upon
// subsequent invocations, the CPU-specific routine is invoked directly, which
// skips the CPU feature detection and subsequent branch that's required.
//
// While this typically doesn't matter for rare occurrences or when used on
// larger haystacks, `memchr` can be called in tight loops where the overhead
// of this branch can actually add up *and is measurable*. This trick was
// necessary to bring this implementation up to glibc's speeds for the 'tiny'
// benchmarks, for example.
//
// At some point, I expect the Rust ecosystem will get a nice macro for doing
// exactly this, at which point, we can replace our hand-jammed version of it.
//
// N.B. The ifunc strategy does prevent function inlining of course, but on
// modern CPUs, you'll probably end up with the AVX2 implementation, which
// probably can't be inlined anyway---unless you've compiled your entire
// program with AVX2 enabled. However, even then, the various memchr
// implementations aren't exactly small, so inlining might not help anyway!
#[cfg(feature = "use_std")]
macro_rules! ifunc {
    ($fnty:ty, $name:ident, $haystack:ident, $($needle:ident),+) => {{
        use std::mem;
        use std::sync::atomic::{AtomicPtr, Ordering};

        type FnRaw = *mut ();

        static FN: AtomicPtr<()> = AtomicPtr::new(detect as FnRaw);

        fn detect($($needle: u8),+, haystack: *const u8) -> usize {
            let fun =
                if cfg!(memchr_runtime_avx) && is_x86_feature_detected!("avx2") {
                    avx::$name as FnRaw
                } else if cfg!(memchr_runtime_sse2) {
                    sse2::$name as FnRaw
                } else {
                    fallback::$name as FnRaw
                };
            FN.store(fun as FnRaw, Ordering::Relaxed);
            unsafe {
                mem::transmute::<FnRaw, $fnty>(fun)($($needle),+, haystack)
            }
        }

        unsafe {
            let fun = FN.load(Ordering::Relaxed);
            mem::transmute::<FnRaw, $fnty>(fun)($($needle),+, $haystack)
        }
    }}
}

// When std isn't available to provide runtime CPU feature detection, or if
// runtime CPU feature detection has been explicitly disabled, then just call
// our optimized SSE2 routine directly. SSE2 is avalbale on all x86_64 targets,
// so no CPU feature detection is necessary.
#[cfg(not(feature = "use_std"))]
macro_rules! ifunc {
    ($fnty:ty, $name:ident, $haystack:ident, $($needle:ident),+) => {{
        if cfg!(memchr_runtime_sse2) {
            unsafe { sse2::$name($($needle),+, $haystack) }
        } else {
            fallback::$name($($needle),+, $haystack)
        }
    }}
}

#[inline(always)]
pub fn rawmemchr(n1: u8, haystack: *const u8) -> usize {
    ifunc!(fn(u8, *const u8) -> usize, rawmemchr, haystack, n1)
}

#[inline(always)]
pub fn rawmemchr2(n1: u8, n2: u8, haystack: *const u8) -> usize {
    ifunc!(fn(u8, u8, *const u8) -> usize, rawmemchr2, haystack, n1, n2)
}

#[inline(always)]
pub fn rawmemchr3(n1: u8, n2: u8, n3: u8, haystack: *const u8) -> usize {
    ifunc!(fn(u8, u8, u8, *const u8) -> usize, rawmemchr3, haystack, n1, n2, n3)
}
