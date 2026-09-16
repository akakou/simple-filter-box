use std::{collections::HashMap, io};
use crate::core::{Entry, FilterBox};


#[derive(Default)]
pub struct HashMapFilterBox {
    inner: HashMap<Entry, ()>,
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


impl FilterBox for HashMapFilterBox {
    
    fn from_entries(tokens: &[Entry]) -> io::Result<Self> {
        Ok(Self {
            inner: 
                tokens
                    .iter()
                    .copied()
                    .map(|token| (token, ()))
                    .collect(),
        })
    }

    fn insert(&mut self, token: &Entry) -> io::Result<()> {
        self.inner.insert(*token, ());
        Ok(())
    }

    fn test(&self, token: &Entry) -> bool {
        !self.inner.contains_key(token)
    }
}
