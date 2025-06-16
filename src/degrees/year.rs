// SPDX-FileCopyrightText: 2024 Stefano Volpe <foxy@teapot.ovh>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use chrono::Datelike;

const SEPTEMBER: u32 = 9;

pub fn current_academic_year() -> u32 {
    let n = chrono::prelude::Local::now();
    let (_, y) = n.year_ce();
    if n.month() >= SEPTEMBER {
        y
    } else {
        y - 1
    }
}
