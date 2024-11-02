#[cfg(test)]
mod tests {
    use crate::utils::{cleanse_jwk, decode_jwt, pad_base64_url}; // Ensure correct module path

    #[test]
    #[test]
    fn test_pad_base_url() {
        // Test case for one extra padding
        assert_eq!(pad_base64_url("SGVsbG8gV29ybGQ"), "SGVsbG8gV29ybGQ=");

        // Test case for two extra padding
        assert_eq!(
            pad_base64_url("YW55IGNhcm5hbCBwbGVhc3"),
            "YW55IGNhcm5hbCBwbGVhc3=="
        );
    }

    #[test]
    fn test_decode_jwt() {}
}
