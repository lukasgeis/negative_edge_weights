//! A RadixMinHeap implementation based on the `radix-heap` crate

use crate::weight::Weight;

/// A Bucket is simply a vector of key-value-pairs
type Bucket<K, V> = Vec<(K, V)>;

/// A RadixMinHeap
pub struct RadixHeap<K, V>
where
    K: Weight,
    [(); K::NUM_BITS + 1]: Sized,
{
    /// Current size of the heap
    len: usize,
    /// Current top-value: all elements pushed must be greater or equal too `top`
    top: K,
    /// The buckets of the heap
    ///
    /// TODO: Use `Vec` for stable channel
    buckets: [Bucket<K, V>; K::NUM_BITS + 1],
}

impl<K, V> RadixHeap<K, V>
where
    K: Weight,
    [(); K::NUM_BITS + 1]: Sized,
{
    /// Creates a new Heap
    #[inline]
    pub fn new() -> Self {
        Self {
            len: 0,
            top: K::zero(),
            buckets: array_init::array_init(|_| Vec::new()),
        }
    }

    /// Resets the heap
    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
        self.top = K::zero();
        for bucket in &mut self.buckets {
            bucket.clear();
        }
    }

    /// Pushes an element to the heap
    #[inline]
    pub fn push(&mut self, key: K, value: V) {
        self.buckets[key.radix_distance(&self.top)].push((key, value));
        self.len += 1;
    }

    /// Gets the smallest element from the heap
    #[inline]
    pub fn pop(&mut self) -> Option<(K, V)> {
        let ret = self.buckets[0].pop().or_else(|| {
            self.update();
            self.buckets[0].pop()
        });

        self.len -= ret.is_some() as usize;
        ret
    }

    /// Updates the heaps by updating the `top` value and refilling the necessary buckets
    fn update(&mut self) {
        let (buckets, repush) = match self.buckets.iter().position(|bucket| !bucket.is_empty()) {
            None | Some(0) => return,
            Some(index) => {
                let (buckets, rest) = self.buckets.split_at_mut(index);
                (buckets, &mut rest[0])
            }
        };

        self.top = repush
            .iter()
            .min_by(|(k1, _), (k2, _)| k1.partial_cmp(k2).unwrap())
            .unwrap()
            .0;

        repush
            .drain(..)
            .for_each(|(key, value)| buckets[key.radix_distance(&self.top)].push((key, value)));
    }

    /// Returns the current top-value of the heap
    #[inline]
    pub fn top(&self) -> K {
        self.top
    }

    /// Returns *true* if there are no items on the heap
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}
