// Copyright (c) 2021-2024 Yuki Kishimoto
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

/// Check if a number is a palindrome using pure math
pub fn is_palindrome(mut n: u64) -> bool {
    if n == 0 {
        return true;
    }

    let original: u64 = n;
    let mut reversed: u64 = 0;

    // Reverse the number mathematically
    while n > 0 {
        reversed = reversed * 10 + n % 10;
        n /= 10;
    }

    original == reversed
}

/// Check if a number is "round"
#[inline]
pub fn is_round_number(n: u64, magnitude: u32) -> bool {
    n % (10u64.pow(magnitude)) == 0
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

    #[test]
    fn test_is_palindrome() {
        assert!(is_palindrome(900009));
        assert!(is_palindrome(888888));
        assert!(is_palindrome(999999));
        assert!(is_palindrome(1110111));
        assert!(is_palindrome(1_111_111));
        assert!(is_palindrome(990099));
        assert!(is_palindrome(12321));
        assert!(is_palindrome(7));
        assert!(is_palindrome(0));

        assert!(!is_palindrome(123456));
        assert!(!is_palindrome(900000));
    }

    #[test]
    fn test_round_numbers() {
        assert!(is_round_number(900_000, 4));
        assert!(is_round_number(1_000_000, 4));
        assert!(is_round_number(1_200_000, 4));
        assert!(is_round_number(1_500_000, 4));
        assert!(is_round_number(2_100_000, 4));
        assert!(is_round_number(10_500_000, 4));
        assert!(is_round_number(50_000, 4));
        assert!(is_round_number(10_000, 4));

        assert!(!is_round_number(7_000, 4));
        assert!(!is_round_number(999_000, 4));
        assert!(!is_round_number(123_456, 4));
        assert!(!is_round_number(900_009, 4));
        assert!(!is_round_number(999, 4));
        assert!(!is_round_number(1200, 4));
    }
}
