#![feature(default_field_values)]
#![feature(nonpoison_rwlock)]
#![feature(nonpoison_condvar)]
#![feature(nonpoison_mutex)]
#![feature(sync_nonpoison)]
#![feature(mapped_lock_guards)]

pub mod iter;

use std::{
    mem::zeroed,
    sync::{
        Arc,
        nonpoison::{Condvar, Mutex, RwLock},
    },
};

/// Multiple reader, single writer
#[expect(unevaluated_default_field_value)]
pub struct RingBuffer<T, const N: usize> {
	buf: RwLock<[T; N]> = RwLock::new([const { unsafe { zeroed()} }; N]),
    /// writer
    cursor: Mutex<usize> = Mutex::new(0),
    /// reader's
    condvar: Arc<Condvar> ,
}

impl<T, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Self {
        Self {
            condvar: Arc::new(Condvar::new()),
            ..
        }
    }

    pub fn iter(&self) -> iter::RingBufferIter<'_, T, N> {
        iter::RingBufferIter::new(self, Arc::clone(&self.condvar))
    }

    pub fn push(&self, item: T) {
        let mut c = self.cursor.lock();
        let guard = &mut self.buf.write()[*c];

        if *c >= N - 1 {
            *c = 0;
        } else {
            *c += 1;
        }

        *guard = item;

        self.condvar.notify_all();
    }
}

impl<T, const N: usize> Drop for RingBuffer<T, N> {
    fn drop(&mut self) {
        *self.cursor.get_mut() = N;
        self.condvar.notify_all();
    }
}

// impl<T, const N: usize> From<RingBuffer<T, N>> for Vec<T> {
//     fn from(value: RingBuffer<T, N>) -> Self {
//         value.iter().collect()
//     }
// }

impl<T, const N: usize> Default for RingBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
