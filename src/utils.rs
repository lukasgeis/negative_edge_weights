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

/// Small wrapper around `Vec<T>` that is meant to be used for cases where `Vec<T>`
/// - has a fixed upper bound on its size
/// - often pushes multiple elements and then clears everything (and repeats...)
///
/// This should be a minor optimization which should be `20ms ~ 100ms` faster
#[derive(Debug, Clone)]
pub struct ReusableVec<T: Default + Clone> {
    vec: Vec<T>,
    len: usize,
}

impl<T: Default + Clone> ReusableVec<T> {
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
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns an iterator over references to all elements in `0..self.len`
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.vec[..self.len].iter()
    }
}
