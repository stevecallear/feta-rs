use std::io::Cursor;

/// Calculates a hash value for the given feature and user key using the Murmur3 algorithm.
pub fn calculate(feature: &str, user_key: &str) -> u32 {
    let mut key = String::new();
    key.push_str(feature);
    key.push_str(user_key);

    // there are no error paths for Cursor::read, so we can assume this will succeed
    murmur3::murmur3_32(&mut Cursor::new(&key), 0).expect("failed to calculate hash")
}
