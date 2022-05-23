// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use sha2::{Digest, Sha512};

pub fn bytes_to_hex_string(bytes: Vec<u8>) -> String {
    let mut hash: String = String::new();
    bytes
        .into_iter()
        .for_each(|b| hash.push_str(format!("{:02X}", b).as_str()));

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
    let number_len: usize = number.len();

    if number_len > 3 {
        let mut counter: u8 = 1;
        loop {
            if num / usize::pow(1000, counter.into()) > 0 {
                counter += 1;
            } else {
                break;
            }
        }

        counter -= 1;

        let mut formatted_number: Vec<String> =
            vec![number[0..(number_len - counter as usize * 3)].into()];

        number.replace_range(0..(number_len - counter as usize * 3), "");

        loop {
            if counter > 0 {
                if !number[0..3].is_empty() {
                    formatted_number.push(number[0..3].into());
                    number.replace_range(0..3, "");
                }

                counter -= 1
            } else {
                break;
            }
        }

        number = formatted_number.join(",");
    }

    number
}
