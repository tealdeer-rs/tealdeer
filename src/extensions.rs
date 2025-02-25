use std::mem;

/// An extension trait to clear duplicates from a collection.
pub(crate) trait Dedup<T: PartialEq> {
    fn clear_duplicates(&mut self);
}

/// Clear duplicates from a collection, keep the first one seen.
///
/// For small vectors, this will be faster than a `HashSet`.
impl<T: PartialEq> Dedup<T> for Vec<T> {
    fn clear_duplicates(&mut self) {
        let orig = mem::replace(self, Vec::with_capacity(self.len()));
        for item in orig {
            if !self.contains(&item) {
                self.push(item);
            }
        }
    }
}

/// Like `str::find`, but starts searching at `start`.
pub(crate) trait FindFrom {
    fn find_from(&self, needle: &Self, start: usize) -> Option<usize>;
}

impl FindFrom for str {
    fn find_from(&self, needle: &Self, start: usize) -> Option<usize> {
        self.get(start..)
            .and_then(|s| s.find(needle))
            .map(|i| i + start)
    }
}

pub(crate) trait ReplaceInplace {
    fn replace_inplace(&mut self, pattern: &Self, replacement: &Self) -> &mut Self;
}

impl<T: PartialEq + Copy> ReplaceInplace for [T] {
    fn replace_inplace(&mut self, pattern: &Self, replacement: &Self) -> &mut Self {
        assert!(replacement.len() <= pattern.len());
        let mut read_index = 0;
        let mut write_index = 0;
        loop {
            if let Some(match_dist) = self[read_index..]
                .windows(pattern.len())
                .position(|win| win == pattern)
            {
                self.copy_within(read_index..read_index + match_dist, write_index);
                read_index += match_dist + pattern.len();
                write_index += match_dist;

                self[write_index..write_index + replacement.len()].copy_from_slice(replacement);
                write_index += replacement.len();
            } else {
                self.copy_within(read_index.., write_index);
                write_index += self.len() - read_index;

                return &mut self[..write_index];
            }
        }
    }
}

impl ReplaceInplace for str {
    fn replace_inplace(&mut self, pattern: &Self, replacement: &Self) -> &mut Self {
        let end = {
            // SAFETY: At the end of the lifetime of `self_bytes`, we have written to all bytes.
            // The bytes until `end` are valid UTF-8, because UTF-8 substrings matching `pattern`
            // are replaced by the UTF-8 string `replacement`. From `end` ongoing, we overwrite
            // everything with ascii letters.
            let self_bytes = unsafe { self.as_bytes_mut() };
            let end = self_bytes
                .replace_inplace(pattern.as_bytes(), replacement.as_bytes())
                .len();
            // Note that if we wouldn't do this, we could have the end of a multi-byte sequence
            // left at the end of `self_bytes` which doesn't have the, say, first byte anymore.
            self_bytes[end..].fill(b'a');
            end
        };
        &mut self[..end]
    }
}
