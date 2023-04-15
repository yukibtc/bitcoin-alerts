// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use sha2::{Digest, Sha512};

pub fn bytes_to_hex_string(bytes: Vec<u8>) -> String {
    let mut hash: String = String::new();
    bytes
        .into_iter()
        .for_each(|b| hash.push_str(format!("{b:02X}").as_str()));

    hash.to_lowercase()
}

pub fn bytes_to_number<T>(bytes: Vec<u8>) -> Option<T>
where
    T: FromStr,
{
    match String::from_utf8(bytes) {
        Ok(result) => match result.parse::<T>() {
            Ok(num) => Some(num),
            Err(_) => None,
        },
        Err(_) => None,
    }
}

pub fn sha512<T>(value: T) -> String
where
    T: AsRef<[u8]>,
{
    let hasher = Sha512::digest(value);
    bytes_to_hex_string(hasher.to_vec())
}

pub fn format_number(num: usize) -> String {
    let mut number: String = num.to_string();

    if number.len() > 3 {
        let reversed: String = number.chars().rev().collect();
        number.clear();
        for (index, char) in reversed.chars().enumerate() {
            if index != 0 && index % 3 == 0 {
                number.push(',');
            }
            number.push(char);
        }
        number = number.chars().rev().collect();
    }

    number
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(100), "100".to_string());
        assert_eq!(format_number(1000), "1,000".to_string());
        assert_eq!(format_number(10000), "10,000".to_string());
        assert_eq!(format_number(100000), "100,000".to_string());
        assert_eq!(format_number(1000000), "1,000,000".to_string());
        assert_eq!(format_number(1000000000), "1,000,000,000".to_string());
    }
}
