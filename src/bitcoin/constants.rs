// Copyright (c) 2021-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Duration;

// pub const SUPPLY_ALERTS: &[f64] = &[
//     19_200_000.0,
//     19_300_000.0,
//     19_400_000.0,
//     19_500_000.0,
//     19_600_000.0,
//     19_700_000.0,
//     19_800_000.0,
//     19_900_000.0,
//     20_000_000.0,
// ];

pub const DEFAULT_RPC_TIMEOUT: Duration = Duration::from_secs(60);
pub const BLOCK_HEIGHT_ROUND_MAGNITUDE: u32 = 4; // Notify blocks divisible by 10^4
