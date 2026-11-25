#![feature(default_field_values)]
#![feature(const_convert)]
#![feature(nonpoison_rwlock)]
#![feature(nonpoison_condvar)]
#![feature(nonpoison_mutex)]
#![feature(sync_nonpoison)]
#![feature(mapped_lock_guards)]

pub mod iter;

use std::{
    mem::MaybeUninit,
    sync::nonpoison::{Condvar, RwLock},
};

/// Multiple reader, single writer
#[expect(unevaluated_default_field_value)]
pub struct RingBuffer<T, const N: usize> {
	buf: RwLock<[T; N]> = RwLock::new([const { unsafe { MaybeUninit::uninit().assume_init() } }; N]),
    // TODO: optimize this wasted 7bits
    // TODO: maybe remove im not sure how is the reader supposed to propagate back to the writer
    lapped: bool = false,
    /// writer
    cursor: usize = 0,
    /// reader's
    condvar: Condvar = Condvar::new(),
}

impl<T, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Self {
        Self { .. }
    }

    pub fn iter(&self) -> iter::RingBufferIter<'_, T, N> {
        iter::RingBufferIter::new(self)
    }

    pub fn push(&mut self, item: T) {
        let guard = &mut self.buf.write()[self.cursor];
        self.cursor += 1;
        *guard = item;
    }
}

impl<T, const N: usize> Default for RingBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
