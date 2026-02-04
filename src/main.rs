// SPDX-FileCopyrightText: 2023 Luca Tagliavini <luca@teapot.ovh>
// SPDX-FileCopyrightText: 2023 Eyad Issa <eyadlorenzo@gmail.com>
// SPDX-FileCopyrightText: 2023 Gabriele Genoveses <gabriele.genovese2@studio.unibo.it>
// SPDX-FileCopyrightText: 2024 Samuele Musiani <samu@teapot.ovh>
// SPDX-FileCopyrightText: 2024 Stefano Volpe <foxy@teapot.ovh>
//
// SPDX-License-Identifier: GPL-3.0-or-later

mod degrees;
mod writer;

use writer::{write_folder, write_index_and_degrees};

/// Entrypoint of the application. Creates an output folder populated with
/// multiple pages for the scraped content, as well as an index file listing
/// said pages. Will panic if:
/// - the output folder does not exist yet, and it cannot be created;
/// - the local list of degrees to be scraped cannot be read from disk;
/// - the scraping fails;
/// - any of the scraped pages cannot be written on disk;
/// - the index cannot be written on disk.
fn main() {
    write_folder();
    write_index_and_degrees();
}
