use std::sync::{
    Arc,
    nonpoison::{Condvar, MappedRwLockReadGuard, Mutex, RwLockReadGuard},
};

use crate::RingBuffer;

pub struct RingBufferIter<'a, T, const N: usize> {
    parent: &'a RingBuffer<T, N>,
    cursor: usize = 0,
    condvar: Arc<Condvar>
}

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

        let i = RwLockReadGuard::map(self.parent.buf.read(), |buf| &buf[&self.cursor - 1]);

        if self.cursor >= N - 1 {
            self.cursor = 0;
        } else {
            self.cursor += 1;
        }

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

    //     pub fn poll(&mut self) -> Option<T> {
    //         if self.cursor < self.parent.cursor && !self.parent.lapped {
    //             let i = &self.parent.buf.read()[self.cursor];
    //             self.cursor += 1;
    //             Some(i)
    //         } else {
    //             None
    //         }
    //     }
}
