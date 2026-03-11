use crate::core::{FilterBox, bytes::Long};
use std::collections::HashMap;
use std::io;

pub struct HashMapFilterBox {
    inner: HashMap<Long, ()>,
}

impl HashMapFilterBox {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
        }
    }
}

impl Default for HashMapFilterBox {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterBox for HashMapFilterBox {
    fn from_entries(tokens: &[Long]) -> io::Result<Self> {
        let mut filter = HashMapFilterBox::with_capacity(tokens.len());
        for token in tokens {
            filter.insert(token)?;
        }
        Ok(filter)
    }

    fn insert(&mut self, token: &Long) -> io::Result<()> {
        self.inner.insert(*token, ());
        Ok(())
    }

    fn test(&self, token: &Long) -> bool {
        !self.inner.contains_key(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_changes_test_result() {
        let mut filter = HashMapFilterBox::new();
        let token: Long = [7u8; 32];
        assert!(filter.test(&token));
        filter.insert(&token).unwrap();
        assert!(!filter.test(&token));
    }

    #[test]
    fn duplicate_insert_is_idempotent() {
        let mut filter = HashMapFilterBox::new();
        let token: Long = [9u8; 32];
        filter.insert(&token).unwrap();
        filter.insert(&token).unwrap();
        assert!(!filter.test(&token));
    }

    #[test]
    fn from_entries_populates_filter() {
        let token_a: Long = [1u8; 32];
        let token_b: Long = [2u8; 32];
        let token_c: Long = [3u8; 32];

        let filter = HashMapFilterBox::from_entries(&[token_a, token_b]).unwrap();
        assert!(!filter.test(&token_a));
        assert!(!filter.test(&token_b));
        assert!(filter.test(&token_c));
    }
}
