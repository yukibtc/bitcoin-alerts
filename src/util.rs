// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use bitcoin::hashes::sha512::Hash as Sha512Hash;
use bitcoin::hashes::Hash;

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
    Sha512Hash::hash(value.as_ref()).to_string()
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
