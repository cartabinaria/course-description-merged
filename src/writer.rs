use crate::degrees::{Degree, analyze_degree, degrees};
use itertools::Itertools;
use std::{
    fs::{create_dir, write},
    path::Path,
};

/// Relative path for the output directory
const OUTPUT_DIR: &str = "output";
/// Relative path for the index file
const INDEX_PATH: &str = "output/index.adoc";

/// Attempts to create a new output folder, if it does not exist
/// already. Will panic on failed folder creations.
pub fn write_folder() {
    let output_dir = Path::new(OUTPUT_DIR);
    if !output_dir.exists()
        && let Err(e) = create_dir(output_dir)
    {
        panic!("Output dir creation: {e}")
    }
}

/// Writes the content of a specific year for a certain degrees. It will use the
/// output folder to do so. Will panic if the write fails. If it does not panic,
/// it returns the content to be added to the index file so that the year is
/// listed and linked in the index.
fn write_year(
    acc: String,
    (year, content): &(&u32, &String),
    name: &String,
    slug: &String,
) -> String {
    let output_file = &format!("output/degree-{slug}-{year}.adoc");
    let output_file = Path::new(output_file);
    write(output_file, content).unwrap();
    format!(
        r#"{}

== {} ({})

xref:degree-{}-{}.adoc[web] | link:degree-{}-{}.pdf[PDF] | link:degree-{}-{}.adoc[Asciidoc]

"#,
        acc, name, year, slug, year, slug, year, slug, year
    )
}

/// Scrapes a degree and renders it as a string. Will panic if the scraping
/// fails.
fn analyze_and_write_degree(d: &Degree) -> String {
    let deg = analyze_degree(d).unwrap();
    let mut entries: Vec<_> = deg.iter().collect();
    entries.sort_by_key(|(k, _)| *k);
    entries.iter().fold("".to_string(), |acc, x| {
        write_year(acc, x, &d.name, &d.slug)
    })
}

/// Collects all degrees to be processed from a local config file, scrapes them,
/// writes them to the output directory, and returns the content of a potential
/// index. Will panic if:
/// - the local config file cannot be read from disk;
/// - the scraping fails;
/// - any of the scraped pages cannot be written on disk.
fn compute_index_and_write_degrees() -> String {
    r#"= Unified Course Descriptions for Some UNIBO Degrees

https://cartabinaria.students.cs.unibo.it/en/wiki/web-scraper/course-description-merged/[Documentation]

"#.to_owned() + degrees().unwrap().iter().map(analyze_and_write_degree).join("\n").as_str()
}

/// Collects all degrees to be processed from a local config file, scrapes them,
/// and writes them to the output directory, together with an index file. Will
/// panic if:
/// - the local config file cannot be read from disk;
/// - the scraping fails;
/// - any of the scraped pages cannot be written on disk;
/// - the index cannot be written on disk.
pub fn write_index_and_degrees() {
    let index = compute_index_and_write_degrees();
    if let Err(e) = write(INDEX_PATH, index) {
        panic!("Could not write index: {e}")
    };
}
