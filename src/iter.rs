pub mod immediate;

use std::sync::{
    Arc,
    nonpoison::{Condvar, MappedRwLockReadGuard, Mutex, RwLockReadGuard},
};

use crate::{RingBuffer, iter::immediate::RingBufferIterImmediate};

pub struct RingBufferIter<'a, T, const N: usize> {
    parent: &'a RingBuffer<T, N>,
    cursor: usize = 0,
    condvar: Arc<Condvar>
}

/// Most of the functions such as .collect will never return(at least in the current design as this requires &parent so parent can't be dropped)
/// but if parent is dropped this should return
impl<'a, T, const N: usize> Iterator for RingBufferIter<'a, T, N> {
    type Item = MappedRwLockReadGuard<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut pc = self.parent.cursor.lock();
        if self.cursor == *pc {
            self.condvar.wait_while(
                &mut pc,
                // we're just betting writer won't get to next lap before we
                // unpark
                |pc| self.cursor < *pc,
            );
        }
        if *pc == N {
            return None;
        }

        let i = RwLockReadGuard::map(self.parent.buf.read(), |buf| &buf[self.cursor]);

        self.adv();

        Some(i)
    }
}

impl<'a, T, const N: usize> RingBufferIter<'a, T, N> {
    pub(crate) fn new(parent: &'a RingBuffer<T, N>, condvar: Arc<Condvar>) -> Self {
        Self {
            parent,
            condvar,
            ..
        }
    }

    pub fn poll(&mut self) -> Option<MappedRwLockReadGuard<'_, T>> {
        let pc = self.parent.cursor.lock();
        if self.cursor == *pc || *pc == N {
            return None;
        }

        let i = RwLockReadGuard::map(self.parent.buf.read(), |buf| &buf[self.cursor]);

        self.adv();

        Some(i)
    }

    pub fn immediate(&'a mut self) -> RingBufferIterImmediate<'a, T, N> {
        RingBufferIterImmediate::new(self)
    }

    fn adv(&mut self) {
        if self.cursor >= N - 1 {
            self.cursor = 0;
        } else {
            self.cursor += 1;
        }
    }
}
