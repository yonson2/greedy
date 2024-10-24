use moka::future::Cache as InnerCache;

/// `Cache` is a re-export of `moka::future::Cache` with its concrete type set.
pub type Cache = InnerCache<String, Vec<u8>>;

/// `new` returns a new `Cache` instance that is used by our http service
pub fn new(c: &crate::config::Cache) -> Cache {
    Cache::builder()
        .weigher(|_key, value: &Vec<u8>| size_of_val(value).try_into().unwrap_or(u32::MAX))
        .max_capacity(c.capacity)
        .build()
}
