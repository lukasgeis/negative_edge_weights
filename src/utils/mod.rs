mod csv;
mod radix;

use std::{fmt::Debug, io::Write, ops::Deref};

pub use csv::*;
pub use radix::*;

/// Quick hack to allow a function to return two different iterators over the same item
pub enum DoubleIterator<I, A, B>
where
    A: Iterator<Item = I>,
    B: Iterator<Item = I>,
{
    IterA(A),
    IterB(B),
}

impl<I, A, B> Iterator for DoubleIterator<I, A, B>
where
    A: Iterator<Item = I>,
    B: Iterator<Item = I>,
{
    type Item = I;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            DoubleIterator::IterA(iter) => iter.next(),
            DoubleIterator::IterB(iter) => iter.next(),
        }
    }
}

// Quick hack to allow a function to return two different structs that both implement `Write`
pub enum DoubleWriter<A, B>
where
    A: Write,
    B: Write,
{
    WriterA(A),
    WriterB(B),
}

impl<A, B> Write for DoubleWriter<A, B>
where
    A: Write,
    B: Write,
{
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            DoubleWriter::WriterA(writer) => writer.write(buf),
            DoubleWriter::WriterB(writer) => writer.write(buf),
        }
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            DoubleWriter::WriterA(writer) => writer.flush(),
            DoubleWriter::WriterB(writer) => writer.flush(),
        }
    }
}

/// Small wrapper around `Vec<T>` that is meant to be used for cases where `Vec<T>`
/// - has a fixed upper bound on its size
/// - often pushes multiple elements and then clears everything (and repeats...)
#[derive(Debug, Clone)]
pub struct ReusableVec<T: Default + Clone, const ASYMPTOTIC_FACTOR: usize = 8> {
    /// Data
    vec: Vec<T>,
    /// Current number of elements in `vec`
    len: usize,
}

impl<T: Default + Clone, const ASYMPTOTIC_FACTOR: usize> Deref
    for ReusableVec<T, ASYMPTOTIC_FACTOR>
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.vec[..self.len]
    }
}

impl<T: Default + Clone, const ASYMPTOTIC_FACTOR: usize> ReusableVec<T, ASYMPTOTIC_FACTOR> {
    /// Creates an instance and fills up the whole vector
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            vec: vec![T::default(); cap],
            len: 0usize,
        }
    }

    /// Pushes an element onto the vector: replaces the value at `self.len` with the current one
    ///
    /// Note that this method does not perform Bound-Checks which can lead to undefined behaviour.
    /// This wrapper is only meant to be used if it is known how many elements will be at most in
    /// the vector at any point in time.
    #[inline]
    pub fn push(&mut self, item: T) {
        unsafe {
            let dst = self.vec.as_mut_ptr().add(self.len);
            core::ptr::write(dst, item);
            self.len += 1;
        }
    }

    /// *Clears* the vector by setting `self.len` to `0`
    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Returns the length of the vector
    #[allow(unused)]
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over references to all elements in `0..self.len`
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.vec[..self.len].iter()
    }

    /// Returns *true* if we have seen `Omega(n)` nodes
    #[inline]
    pub fn is_asymptotically_full(&self) -> bool {
        self.len > self.vec.len() / ASYMPTOTIC_FACTOR
    }
}

/// Stack-Allocated Version of `ReusableVec`
#[derive(Copy, Clone)]
pub struct CappedVec<T, const SIZE: usize> {
    pub data: [T; SIZE],
    pub size: usize,
}

impl<T, const SIZE: usize> Deref for CappedVec<T, SIZE> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.data[..self.size]
    }
}

impl<T: Default + Copy, const SIZE: usize> Default for CappedVec<T, SIZE> {
    fn default() -> Self {
        CappedVec {
            data: [T::default(); SIZE],
            size: 0,
        }
    }
}

impl<T: Debug, const SIZE: usize> Debug for CappedVec<T, SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CappedVec")
            .field("data", &&self.data[..self.size])
            .field("size", &self.size)
            .field("cap", &SIZE)
            .finish()
    }
}

impl<T, const SIZE: usize> CappedVec<T, SIZE> {
    /// Pushes an element to the vector
    pub fn push(&mut self, elem: T) {
        assert!(self.size < SIZE);
        self.data[self.size] = elem;
        self.size += 1;
    }

    /// Clears the vector
    pub fn clear(&mut self) {
        self.size = 0;
    }

    /// Returns *true* if the vector is full
    pub fn is_full(&self) -> bool {
        self.size == SIZE
    }

    /// Current size of the vector
    pub fn size(&self) -> usize {
        self.size
    }

    /// Remove the first `len` elements of the vector
    pub fn remove_prefix(&mut self, len: usize) {
        self.data.rotate_left(len);
        self.size = self.size.saturating_sub(len);
    }

    /// Get a reference to the element at index `idx`
    pub fn get(&self, idx: usize) -> &T {
        &self.data[idx]
    }

    /// Get a mutable reference to the element at index `idx`
    pub fn get_mut(&mut self, idx: usize) -> &mut T {
        &mut self.data[idx]
    }
}
