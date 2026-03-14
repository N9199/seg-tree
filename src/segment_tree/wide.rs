const fn height(n: usize, b: usize) -> usize {
    assert!(b != 0);
    if n <= b { 1 } else { height(n / b, b) + 1 }
}

const fn offset(mut n: usize, mut h: usize, b: usize) -> usize {
    let mut s = 0;
    while h > 0 {
        n = n.div_ceil(b);
        s += n * b;
        h -= 1;
    }
    s
}

const fn optimal_from_size(size: usize) -> usize {
    let size = size.next_power_of_two();
    // Assuming 64 bytes cache line
    let b = 64 / size;
    if b < 2 { 2 } else { b }
}

const fn optimal_b<T>() -> usize {
    optimal_from_size(size_of::<T>())
}

const fn round(k: usize, b: usize) -> usize {
    (k / b) * b
}

mod wide_recursive_impl;

#[cfg(feature = "nightly_unstable")]
type WideRecursive<T>
    = wide_recursive_impl::WideRecursive<{ optimal_b::<T>() }, T>
where
    [u8; optimal_b::<T>()]: Sized;

#[cfg(not(feature = "nightly_unstable"))]
const B: usize = 1 << 4;
#[cfg(not(feature = "nightly_unstable"))]
type WideRecursive<T> = wide_recursive_impl::WideRecursive<B, T>;
