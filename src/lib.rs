#![feature(default_field_values)]
#![feature(nonpoison_rwlock)]
#![feature(nonpoison_condvar)]
#![feature(nonpoison_mutex)]
#![feature(sync_nonpoison)]
#![feature(mapped_lock_guards)]

pub mod iter;

use std::{
    mem::MaybeUninit,
    sync::{
        Arc,
        nonpoison::{Condvar, Mutex, RwLock},
    },
};

/// Multiple reader, single writer
#[expect(unevaluated_default_field_value)]
pub struct RingBuffer<T, const N: usize> {
	buf: RwLock<[T; N]> = RwLock::new([const { unsafe { MaybeUninit::uninit().assume_init() } }; N]),
    /// writer
    cursor: usize = 0,
    /// reader's
    condvar: Arc<(Condvar, Mutex<()>)>  ,
}

impl<T, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Self {
        Self {
            condvar: Arc::new(
                (
                    Condvar::new(),
                    Mutex::new(()),
                ),
            ),
            ..
        }
    }

    pub fn iter(&self) -> iter::RingBufferIter<'_, T, N> {
        iter::RingBufferIter::new(
            self,
            Arc::clone(&self.condvar),
        )
    }

    pub fn push(&mut self, item: T) {
        let guard = &mut self.buf.write()[self.cursor];
        if self.cursor >= N {
            self.cursor = 0;
        } else {
            self.cursor += 1;
        }
        *guard = item;
        self.condvar.0.notify_all();
    }
}

impl<T, const N: usize> Default for RingBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
