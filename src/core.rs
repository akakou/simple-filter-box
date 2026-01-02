use std::io;

use crate::core::bytes::Long;


pub type Entry = [u8; 32];

pub mod bytes {
    use super::Entry;

    pub type Long = Entry;
}

pub enum FilterResult {
    Exist,
    NotExist,
}

pub trait FilterBox {
    fn from_entries(token: &[Long]) -> io::Result<Self> where Self: Sized;
    fn insert(&mut self, token: &Entry) -> io::Result<()>;
    fn test(&self, token: &Entry) -> bool;
}