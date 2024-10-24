#[allow(unused, dead_code)]
use crate::utils::alphanumeric_digit;
use bitvec::prelude::*;

#[derive(Debug, PartialEq)]
pub enum EncodingType {
    Byte,
    Numeric,
    Alphanumeric,
}

#[derive(Debug, PartialEq)]
pub enum EncodingError {
    InvalidInput,
    DataNotProvided,
}

pub fn determine_encoding_type(input: &str) -> Result<EncodingType, EncodingError> {
    if input.is_empty() {
        return Err(EncodingError::DataNotProvided);
    }

    if input.is_ascii() {
        if input.chars().all(|c| c.is_numeric()) {
            return Ok(EncodingType::Numeric);
        }

        let valid_alphanumeric: Vec<char> =
            "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ$%*+-./:"
                .chars()
                .collect();

        // Check if the input contains only valid alphanumeric characters.
        if input
            .chars()
            .all(|c| valid_alphanumeric.contains(&c) || c.is_whitespace())
        {
            return Ok(EncodingType::Alphanumeric);
        } else {
            return Ok(EncodingType::Byte); // Return Byte if invalid alphanumeric but valid byte or ascii.
        }
    }

    Err(EncodingError::InvalidInput)
}

pub fn encode_to_bitvector(data: &str, bitvector: &mut BitVec) {
    let count: u8 = data.chars().count() as u8;
    let mode = determine_encoding_type(&data).unwrap();

    match mode {
        EncodingType::Byte => encode_byte(&data, &count, bitvector),
        EncodingType::Numeric => encode_numeric(&data, &count, bitvector),
        EncodingType::Alphanumeric => encode_alphanumeric(&data, &count, bitvector),
    }

    bitvector.extend([false, false, false, false]); // add padding
}

fn create_header(a: Vec<i32>, length: &u8, bit_vector: &mut BitVec) {
    unimplemented!()
}

fn encode_byte(byte_data: &str, count: &u8, bitvector: &mut BitVec) {
    // Convert count to binary and extend bitvector
    let count_to_bin: Vec<bool> = (0..8).rev().map(|i| (count >> i) & 1 == 1).collect();
    bitvector.extend(count_to_bin); // Add the count in binary

    // Convert alphanumeric data to binary and extend bitvector
    let bit_vec: Vec<bool> = byte_data
        .as_bytes()
        .iter()
        .flat_map(|&byte| {
            (0..8).rev().map(move |i| (byte >> i) & 1 == 1) // Extract each bit from the byte
        })
        .collect();
    bitvector.extend(bit_vec);
}

fn encode_numeric(numeric_data: &str, count: &u8, bitvector: &mut BitVec) {
    for chunk in numeric_data.as_bytes().chunks(3) {
        let length = chunk.len() * 3 + 1; // 123, 10bits 012, 7bits, 001, 4bits
    }
}

fn encode_alphanumeric(alphanumeric_data: &str, count: &u8, bitvector: &mut BitVec) {
    for chunk in alphanumeric_data.as_bytes().chunks(2) {
        let number = chunk
            .iter()
            .map(|b| alphanumeric_digit(*b))
            .fold(0, |a, b| a * 45 + b);
        let length = chunk.len() * 5 + 1;
    }
}
