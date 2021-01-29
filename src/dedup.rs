use std::mem;

/// An extension trait to clear duplicates from a collection.
pub(crate) trait Dedup<T: PartialEq + Clone> {
    fn clear_duplicates(&mut self);
}

/// Clear duplicates from a collection, keep the first one seen.
///
/// For small vectors, this will be faster than a `HashSet`.
impl<T: PartialEq + Clone> Dedup<T> for Vec<T> {
    fn clear_duplicates(&mut self) {
        let orig = mem::replace(self, Vec::with_capacity(self.len()));
        for item in orig {
            if !self.contains(&item) {
                self.push(item);
            }
        }
    }
}
