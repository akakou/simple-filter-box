pub mod core;
#[cfg(feature = "binary_search")]

pub mod binary_search;

#[cfg(feature = "bloom_filter")]
pub mod bloom_filter;

#[cfg(feature = "dashmap")]
pub mod dashmap;

#[cfg(feature = "hashmap")]
pub mod hashmap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
