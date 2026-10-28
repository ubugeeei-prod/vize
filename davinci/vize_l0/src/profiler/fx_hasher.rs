//! Opt-in Fx interface observations for explicitly named private maps.
//!
//! Each counter names a wrapper interface entry, not internal Fx helper calls:
//! delegated write(bytes) does not count FxHasher's internal write_u64 again.
//! Logical input bytes are not copied bytes, allocation bytes, or RSS.
//! hash_one includes its nested build/write/finish observations; finish may be
//! called repeatedly and never means a distinct complete hash computation.
//! Stable Rust cannot override write_str/write_length_prefix; those interface
//! counts are unsupported. Their stable default lowerings remain unchanged.
//! Scope counters aggregate participating Rust threads through micro::Site.
//! Profiler registries/maps keep the original raw hasher, avoiding recursion.

use rustc_hash::{FxBuildHasher, FxHasher};
use std::hash::{BuildHasher, Hash, Hasher};
use std::marker::PhantomData;

#[derive(Clone, Copy)]
pub enum Operation {
    Build,
    HashOne,
    Write,
    Finish,
    WriteU8,
    WriteU16,
    WriteU32,
    WriteU64,
    WriteU128,
    WriteUsize,
    WriteI8,
    WriteI16,
    WriteI32,
    WriteI64,
    WriteI128,
    WriteIsize,
}

/// Implemented only by an explicit owner declaration, with static source sites.
pub trait Scope {
    fn record(operation: Operation, units: u64);
}

/// A zero-sized scope marker; default construction does not allocate or hash.
pub struct Builder<S: Scope>(PhantomData<fn() -> S>);

impl<S: Scope> Default for Builder<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<S: Scope> Copy for Builder<S> {}
impl<S: Scope> Clone for Builder<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: Scope> BuildHasher for Builder<S> {
    type Hasher = Delegate<S>;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        S::record(Operation::Build, 0);
        Delegate {
            inner: FxBuildHasher.build_hasher(),
            _scope: PhantomData,
        }
    }

    #[inline]
    fn hash_one<T: Hash>(&self, value: T) -> u64 {
        S::record(Operation::HashOne, 0);
        // Exact pinned BuildHasher default, while observing its delegate calls.
        let mut hasher = self.build_hasher();
        value.hash(&mut hasher);
        hasher.finish()
    }
}

/// Delegates every stable entry directly, preserving signed/default semantics.
pub struct Delegate<S: Scope> {
    inner: FxHasher,
    _scope: PhantomData<fn() -> S>,
}

impl<S: Scope> Clone for Delegate<S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _scope: PhantomData,
        }
    }
}

impl<S: Scope> Hasher for Delegate<S> {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        S::record(Operation::Write, bytes.len() as u64);
        self.inner.write(bytes);
    }

    #[inline]
    fn finish(&self) -> u64 {
        S::record(Operation::Finish, 0);
        self.inner.finish()
    }

    #[inline]
    fn write_u8(&mut self, value: u8) {
        S::record(Operation::WriteU8, std::mem::size_of::<u8>() as u64);
        self.inner.write_u8(value);
    }

    #[inline]
    fn write_u16(&mut self, value: u16) {
        S::record(Operation::WriteU16, std::mem::size_of::<u16>() as u64);
        self.inner.write_u16(value);
    }

    #[inline]
    fn write_u32(&mut self, value: u32) {
        S::record(Operation::WriteU32, std::mem::size_of::<u32>() as u64);
        self.inner.write_u32(value);
    }

    #[inline]
    fn write_u64(&mut self, value: u64) {
        S::record(Operation::WriteU64, std::mem::size_of::<u64>() as u64);
        self.inner.write_u64(value);
    }

    #[inline]
    fn write_u128(&mut self, value: u128) {
        S::record(Operation::WriteU128, std::mem::size_of::<u128>() as u64);
        self.inner.write_u128(value);
    }

    #[inline]
    fn write_usize(&mut self, value: usize) {
        S::record(Operation::WriteUsize, std::mem::size_of::<usize>() as u64);
        self.inner.write_usize(value);
    }

    #[inline]
    fn write_i8(&mut self, value: i8) {
        S::record(Operation::WriteI8, std::mem::size_of::<i8>() as u64);
        self.inner.write_i8(value);
    }

    #[inline]
    fn write_i16(&mut self, value: i16) {
        S::record(Operation::WriteI16, std::mem::size_of::<i16>() as u64);
        self.inner.write_i16(value);
    }

    #[inline]
    fn write_i32(&mut self, value: i32) {
        S::record(Operation::WriteI32, std::mem::size_of::<i32>() as u64);
        self.inner.write_i32(value);
    }

    #[inline]
    fn write_i64(&mut self, value: i64) {
        S::record(Operation::WriteI64, std::mem::size_of::<i64>() as u64);
        self.inner.write_i64(value);
    }

    #[inline]
    fn write_i128(&mut self, value: i128) {
        S::record(Operation::WriteI128, std::mem::size_of::<i128>() as u64);
        self.inner.write_i128(value);
    }

    #[inline]
    fn write_isize(&mut self, value: isize) {
        S::record(Operation::WriteIsize, std::mem::size_of::<isize>() as u64);
        self.inner.write_isize(value);
    }
}
