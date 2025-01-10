//! An AtomicU64 type that can be used in platforms where 64bit atomic is not supported.

use std::sync::atomic::Ordering;

pub(crate) struct AtomicU64 {
    #[cfg(target_has_atomic = "64")]
    inner: std::sync::atomic::AtomicU64,

    #[cfg(not(target_has_atomic = "64"))]
    inner: std::sync::atomic::AtomicU32,
}

impl AtomicU64 {
    #[cfg(target_has_atomic = "64")]
    pub(crate) fn new(x: u64) -> Self {
        Self {
            inner: std::sync::atomic::AtomicU64::new(x),
        }
    }

    #[cfg(not(target_has_atomic = "64"))]
    pub(crate) fn new(x: u32) -> Self {
        Self {
            inner: std::sync::atomic::AtomicU64::new(x as u64),
        }
    }

    pub(crate) fn load(&self, order: Ordering) -> u64 {
        #[cfg(target_has_atomic = "64")]
        {
            self.inner.load(order)
        }

        #[cfg(not(target_has_atomic = "64"))]
        {
            self.inner.load(order) as u64
        }
    }
}
