#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::{determine_encoding_type, EncodingType, EncodingError}; // Ensure correct module path

    #[test]
    fn test_determine_encoding_type() {
        // Test case for a valid alphanumeric string
        assert_eq!(determine_encoding_type("Hello 123"), Ok(EncodingType::Alphanumeric), "Failed 1");

        // Test case for a string containing only digits (numeric)
        assert_eq!(determine_encoding_type("12345"), Ok(EncodingType::Numeric), "Failed 2");

        // Test case for a string with invalid characters (should fall back to Byte)
        assert_eq!(determine_encoding_type("HELLO!"), Ok(EncodingType::Byte), "Failed 3");

        // Test case for an empty input (should return DataNotProvided error)
        assert_eq!(determine_encoding_type(""), Err(EncodingError::DataNotProvided), "Failed 4");

        // Test case for a non-ASCII input (should return InvalidInput error)
        assert_eq!(determine_encoding_type("こんにちは"), Err(EncodingError::InvalidInput), "Failed 5");

        // Test case for a valid alphanumeric string with special characters
        assert_eq!(determine_encoding_type("HELLO 123$%*+-./:"), Ok(EncodingType::Alphanumeric), "Failed 6");
    }
}
