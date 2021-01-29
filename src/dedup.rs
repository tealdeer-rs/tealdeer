/// An extension trait to clear duplicates from a collection.
pub(crate) trait Dedup<T: PartialEq + Clone> {
    fn clear_duplicates(&mut self);
}

/// Clear duplicates from a collection, keep the first one seen.
///
/// For small vectors, this will be faster than a `HashSet`.
/// Based on <https://stackoverflow.com/a/57889826/284318>
impl<T: PartialEq + Clone> Dedup<T> for Vec<T> {
    fn clear_duplicates(&mut self) {
        let mut already_seen = Vec::with_capacity(self.len());
        self.retain(|item| {
            if already_seen.contains(item) {
                false
            } else {
                already_seen.push(item.clone());
                true
            }
        })
    }
}
