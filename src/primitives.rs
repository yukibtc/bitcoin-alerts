// Copyright (c) 2021-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target {
    Matrix,
    Nostr,
    Ntfy,
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Matrix => write!(f, "matrix"),
            Self::Nostr => write!(f, "nostr"),
            Self::Ntfy => write!(f, "ntfy"),
        }
    }
}
