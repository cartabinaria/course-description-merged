// SPDX-FileCopyrightText: 2024 Stefano Volpe <foxy@teapot.ovh>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use chrono::{Datelike, prelude::Local};

/// 1-based index of the month of September.
const SEPTEMBER: u32 = 9;

/// Returns the opening solar year of the current academic year, e.g. 2025 for
/// 2025-26. Recall that academic years start in September, and end in August.
/// The current year is based on the local system clock.
pub fn current_academic_year() -> u32 {
    let now = Local::now();
    let (_, y) = now.year_ce();
    if now.month() >= SEPTEMBER { y } else { y - 1 }
}
