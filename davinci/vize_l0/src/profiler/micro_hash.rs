//! Counters for selected `DefaultHasher` callsites, never a blanket hasher replacement.
//!
//! Rust 1.99's DefaultHasher uses Hasher's primitive defaults. Its string
//! override feeds UTF8 bytes plus 0xff, exactly the trait default used here.
//! Digest parity is checked for those methods and prefix-free strings. This
//! adapter must not be applied to other hashers with different overrides.

use std::hash::{DefaultHasher, Hasher};

use super::micro::{Site, count};

/// Constructed only by `micro_default_hasher!` in instrumentation builds.
pub struct DefaultHasherProbe {
    inner: DefaultHasher,
    writes: &'static Site,
    finishes: &'static Site,
}

impl DefaultHasherProbe {
    /// The builder, individual byte writes and finalizations have separate sites.
    pub fn new(builds: &'static Site, writes: &'static Site, finishes: &'static Site) -> Self {
        count(builds, 1);
        Self {
            inner: DefaultHasher::new(),
            writes,
            finishes,
        }
    }
}

impl Hasher for DefaultHasherProbe {
    fn write(&mut self, bytes: &[u8]) {
        count(self.writes, bytes.len() as u64);
        self.inner.write(bytes);
    }

    fn finish(&self) -> u64 {
        count(self.finishes, 1);
        self.inner.finish()
    }
}

#[cfg(test)]
mod tests {
    use std::hash::{DefaultHasher, Hash, Hasher};

    fn same_digest(value: &impl Hash) {
        let mut expected = DefaultHasher::new();
        let mut instrumented = crate::micro_default_hasher!("test.digest.parity");
        value.hash(&mut expected);
        value.hash(&mut instrumented);
        assert_eq!(instrumented.finish(), expected.finish());
        assert_eq!(instrumented.finish(), expected.finish());
    }

    #[test]
    fn known_std_hash_methods_keep_their_exact_digest() {
        for value in ["", "a", "a\0", "𠮷\r\n東京", "12345678", "123456789"] {
            same_digest(&value);
        }
        for length in [7, 8, 9, 15, 16, 17, 128, 1024] {
            same_digest(&"x".repeat(length));
        }
        same_digest(&(true, false, -37_i8, 99_u16, -73_i32));
        same_digest(&(99_u64, -901_i64, u128::MAX, i128::MIN, usize::MAX));
        same_digest(&(Some("text"), None::<u64>, vec![1_u8, 2, 3]));
        same_digest(&std::path::Path::new("workspace/東京/App.vue"));
        same_digest(&("a", "bc"));
        same_digest(&("ab", "c"));
    }

    #[test]
    fn direct_writes_keep_boundaries_and_non_resetting_finish() {
        let mut expected = DefaultHasher::new();
        let mut instrumented = crate::micro_default_hasher!("test.write.parity");
        for bytes in [b"abc".as_slice(), b"", b"defghijklmn"] {
            expected.write(bytes);
            instrumented.write(bytes);
            assert_eq!(instrumented.finish(), expected.finish());
        }
        expected.write_i128(i128::MIN);
        instrumented.write_i128(i128::MIN);
        expected.write_usize(37);
        instrumented.write_usize(37);
        assert_eq!(instrumented.finish(), expected.finish());
    }
}
