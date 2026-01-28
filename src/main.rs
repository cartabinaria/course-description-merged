// SPDX-FileCopyrightText: 2023 Luca Tagliavini <luca@teapot.ovh>
// SPDX-FileCopyrightText: 2023 Eyad Issa <eyadlorenzo@gmail.com>
// SPDX-FileCopyrightText: 2023 Gabriele Genoveses <gabriele.genovese2@studio.unibo.it>
// SPDX-FileCopyrightText: 2024 Samuele Musiani <samu@teapot.ovh>
// SPDX-FileCopyrightText: 2024 Stefano Volpe <foxy@teapot.ovh>
//
// SPDX-License-Identifier: GPL-3.0-or-later

mod degrees;
mod logging;
mod writer;

use logging::setup;
use writer::write_all;

fn main() {
    setup();
    write_all();
}
