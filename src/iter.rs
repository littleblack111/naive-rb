use std::sync::nonpoison::{Condvar, MappedRwLockReadGuard, Mutex, RwLockReadGuard};

use crate::RingBuffer;

#[expect(unevaluated_default_field_value)]
pub struct RingBufferIter<'a, T, const N: usize> {
    parent: &'a RingBuffer<T, N>,
    cursor: usize = 0,
    condvar: Condvar = Condvar::new(),
    mutex: Mutex<()> = Mutex::new(()),
}

impl<'a, T, const N: usize> Iterator for RingBufferIter<'a, T, N> {
    type Item = MappedRwLockReadGuard<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.parent.lapped && self.cursor >= self.parent.cursor {
            self.condvar.wait_while(
                &mut self.mutex.lock(),
                |_| self.parent.lapped || self.cursor < self.parent.cursor,
            );
        }
        // i'm pretty sure this is supposed to only be set by the last reader
        // self.parent.lapped = false;
        let i = RwLockReadGuard::map(
            self.parent.buf.read(),
            |buf| &buf[&self.cursor - 1],
        );
        if self.cursor >= N {
            // self.parent.lapped = false;
            self.cursor = 0;
        } else {
            self.cursor += 1;
        }
        Some(i)
    }
}

impl<'a, T, const N: usize> RingBufferIter<'a, T, N> {
    pub(crate) fn new(parent: &'a RingBuffer<T, N>) -> Self {
        Self {
            parent,
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
