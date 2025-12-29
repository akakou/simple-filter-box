use std::io;


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
    fn insert(&mut self, token: &Entry) -> io::Result<()>;
    fn test(&self, token: &Entry) -> bool;
}
