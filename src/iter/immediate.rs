use std::sync::nonpoison::{MappedRwLockReadGuard, RwLockReadGuard};

use crate::iter::RingBufferIter;

pub struct RingBufferIterImmediate<'a, T, const N: usize> {
    parent: &'a mut RingBufferIter<'a, T, N>,
}

impl<'a, T, const N: usize> RingBufferIterImmediate<'a, T, N> {
    pub fn new(parent: &'a mut RingBufferIter<'a, T, N>) -> Self {
        Self {
            parent,
        }
    }
}

impl<'a, T, const N: usize> Iterator for RingBufferIterImmediate<'a, T, N> {
    type Item = MappedRwLockReadGuard<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let pc = self.parent.parent.cursor.lock();
        if self.parent.cursor == *pc || *pc == N {
            return None;
        }

        let i = RwLockReadGuard::map(
            self.parent.parent.buf.read(),
            |buf: &[T; N]| &buf[self.parent.cursor],
        );

        self.parent.adv();

        Some(i)
    }
}
