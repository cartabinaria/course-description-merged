// SPDX-FileCopyrightText: 2024 Stefano Volpe <foxy@teapot.ovh>
//
// SPDX-License-Identifier: GPL-3.0-or-later

mod teachings;
mod year;

use itertools::Itertools;
use regex::Regex;
use reqwest::blocking::get;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json::from_reader;
use std::{
    collections::HashMap, error::Error, fmt::Display, fs::File, iter::repeat, sync::LazyLock,
};
use teachings::get_desc_teaching_page;
use year::current_academic_year;

/// The relative path at which the predregrees are saved.
const DEGREES_PATH: &str = "config/degrees.json";

/// The number of enrollment years to scrape for each degree.
const YEARS_PER_DEGREE: u32 = 3;

/// Selector for table titles.
static TABLE: LazyLock<Selector> = LazyLock::new(|| Selector::parse("td.title").unwrap());
/// Selector for first link of a list
static FIRST_LINK: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(".no-bullet > li:first-child > a").unwrap());
/// Italian-to-English dictionary for teachings whose name has not been properly
/// translated on their English course page.
static MISSING_TRANSLATIONS: LazyLock<[(String, String); 5]> = LazyLock::new(|| {
    [
        ("BASI DI DATI".to_string(), "DATABASES".to_string()),
        (
            "INTRODUZIONE ALL'APPRENDIMENTO AUTOMATICO".to_string(),
            "Introduction to machine learning".to_string(),
        ),
        ("FONDAMENTI DI".to_string(), "".to_string()),
        (
            "Learning outcomes".to_string(),
            "=== Learning outcomes".to_string(),
        ),
        (
            "Teaching contents".to_string(),
            "=== Teaching contents".to_string(),
        ),
    ]
});

/// A predegree represents the metadata on a degree which is already available
/// in the `config` submodule. This needs to be preprocessed before becoming the
/// matdata we require.
#[derive(Deserialize, Debug, Clone)]
struct Predegree {
    /// The unique kebab-case name used by Cartabinaria software to refer to the
    /// degree.
    id: String,
    /// The human-readable name of the degree
    name: String,
    /// The unique code used by Unibo to refer to the degree, usually in the
    /// format 1234/567
    code: String,
}

/// The degree metadata necessary for the scraping process. This information
/// cannot be stored in the `config` submodule, as part of it is updated yearly.
pub struct Degree {
    /// The human-readable name of the degree
    pub name: String,
    /// The slug used by webpages on unibo.it
    pub slug: String,
    /// For each (recent) academic year, a URL to the description of the courses
    /// of the programme for students enrolled in it. The key represents the
    /// solar year during which the academic year started (recall that academic
    /// years start in September).
    year_urls: HashMap<u32, String>,
}

/// Takes a slug used by unibo.it to represent the degree level (usually
/// "laurea" for a B.Sc., and "magistrale" for a M.Sc.), as well as a slug used
/// for a degree of such level. Returns the URLs (dictionary values) to the
/// descriptions of the courses of the programme for students enrolled at
/// various recent years (diciontary keys, see [YEARS_PER_DEGREE]). The URLs are
/// collected via web scraping. If a URL cannot be collected, it will not be
/// added to the returned dictionary.
///
/// This project is mainly dedicated to B.Sc. students applying for M.Sc.
/// degrees. Because you don't usually apply for a M.Sc. in your first and
/// second year as a B.Sc. student, the current and previous academic years
/// (with reference to the current system clock) are excluded from the scraping.
fn get_degree_structure_urls(degree_level: &str, degree_name: String) -> HashMap<u32, String> {
    let previous_academic_year = current_academic_year() - 1;
    let first_scraped_year = previous_academic_year - YEARS_PER_DEGREE;

    let get_degree_structure_url = |year: u32| -> Option<(u32, String)> {
        let url =
            format!("https://corsi.unibo.it/{degree_level}/{degree_name}/insegnamenti?year={year}");
        eprintln!("Visiting: {url}");

        let res = get(&url).ok()?.error_for_status().ok()?;
        let text = res.text().ok()?;
        let document = Html::parse_document(&text);
        let link = document.select(&FIRST_LINK).next()?;
        let href = link.value().attr("href")?.to_string();
        eprintln!("Got link: {href}");
        Some((year, href))
    };

    (first_scraped_year..previous_academic_year)
        .filter_map(get_degree_structure_url)
        .collect()
}

/// Infers a level ("laurea" for B.Sc., "magistrale" for M.Sc.) from a
/// human-readable degree name.
fn name_to_level(name: &str) -> &'static str {
    if name.contains("Magistrale") || name.contains("Master") {
        "magistrale"
    } else {
        "laurea"
    }
}

/// Converts the string to lowercase, if the second argument is `true`.
fn to_lowercase_maybe(s: String, b: bool) -> String {
    if b { s.to_lowercase() } else { s }
}

/// Infers a slug used by `unibo.it` for a certain degree. We are hardcoding
/// some exceptions.
fn name_and_code_to_slug(name: &str, code: &String) -> String {
    to_lowercase_maybe(
        Regex::new(r"( (e|per il|in) )|Magistrale|Master")
            .unwrap()
            .replace_all(name, "")
            .to_string(),
        // CS Engineering is PascalCase
        !code.eq("9254/000"),
    )
    // AI's slug is kebab-case
    .replace(' ', if code.eq("9063/000") { "-" } else { "" })
}

/// Attempts converting a [Predegree] into a [Degree]. Fails if either field of
/// the input is empty. To do so, it performs some webscraping.
fn parse_degree(predegree: &Predegree) -> Option<Degree> {
    let Predegree { name, id, code } = predegree;
    if name.is_empty() || id.is_empty() || code.is_empty() {
        None
    } else {
        let level = name_to_level(name);
        let unibo_slug = name_and_code_to_slug(name, code);
        Some(Degree {
            name: name.to_string(),
            slug: id.to_string(),
            year_urls: get_degree_structure_urls(level, unibo_slug),
        })
    }
}

/// Converts a vector of predegrees into one of degrees. Failed conversions will
/// not be part of the output. To perform the conversion, some webscraping is
/// necessary.
fn to_degrees(predegrees: Vec<Predegree>) -> Vec<Degree> {
    predegrees.iter().filter_map(parse_degree).collect()
}

/// Performs a replacement in a string
fn replace(s: String, (find, replace): &(String, String)) -> String {
    s.replace(find, replace)
}

/// Renders a teaching description, using [MISSING_TRANSLATIONS] to translate
/// teaching names.
fn render_description(desc: String) -> String {
    let entry_doc = "\n".to_string() + desc.as_str();
    MISSING_TRANSLATIONS.iter().fold(entry_doc, replace)
}

/// Scrapes a teaching page and renders it.
fn scrape_link(url: &str, degree_slug: &String, year: &u32) -> Result<String, String> {
    get_desc_teaching_page(degree_slug, year, url)
        .map_err(|e| format!("\t\tWARN: Cannot get description: {e:?}"))
        .map(render_description)
}

/// Converts a Result to an Option. In case of error, the error is printed to
/// `stderr`.
fn result_to_option_with_print<T, E: Display>(r: Result<T, E>) -> Option<T> {
    match r {
        Ok(r) => Some(r),
        Err(e) => {
            eprintln!("{e}");
            None
        }
    }
}

/// Renders a single course given the corresponding HTML in the degree structure
/// webpage. To do so, it looks for a link to the course page in the HTML, and
/// scrapes that.
fn render_course(
    (course_html, (degree_slug, year)): (scraper::ElementRef<'_>, (&String, &u32)),
) -> Option<String> {
    let a_el = course_html
        .children()
        .filter_map(|f| f.value().as_element())
        .find(|r| r.name() == "a")
        .and_then(|a_el| a_el.attr("href"));
    let temp_name = course_html.text().join("");
    let name = temp_name.trim();
    eprintln!("\tVisiting {name}");
    let res = a_el
        .ok_or(format!("\t\tWARN: Missing link: {name}"))
        .and_then(|url| scrape_link(url, degree_slug, year));
    result_to_option_with_print(res)
}

/// Scrapes a yearly degree structure URL, returning textual content for it.
fn analyze_year_with_error(
    ((year, url), (degree_slug, degree_name)): ((&u32, &String), (&String, &String)),
) -> Result<(u32, String), String> {
    eprintln!("Analysing {year} link: {url}");
    let res = get(url).map_err(|e| format!("\tNetwork error: {e}"))?;
    let res = res
        .error_for_status()
        .map_err(|e| format!("\tServer error: {e}"))?;
    let text = res.text().map_err(|e| format!("\tDecoding error: {e}"))?;
    let document = Html::parse_document(&text);
    let title = format!("= {degree_name} ({year})\n\n");
    let courses = document
        .select(&TABLE)
        .zip(repeat((degree_slug, year)))
        .filter_map(render_course)
        .join("");
    Ok((*year, title + courses.as_str()))
}

/// Given a year, a degree structure URL, a degree slug, and a degree name,
/// attempts to perform webscraping and return a textual representation of the
/// response.
fn analyze_year(x: ((&u32, &String), (&String, &String))) -> Option<(u32, String)> {
    result_to_option_with_print(analyze_year_with_error(x))
}

/// Scrapes the yearly degree structure URLs specified by a degree, returning
/// textual content for each.
pub fn analyze_degree(degree: &Degree) -> Result<HashMap<u32, String>, Box<dyn Error>> {
    let Degree {
        slug,
        name,
        year_urls,
    } = degree;
    let res = year_urls
        .iter()
        .zip(repeat((slug, name)))
        .filter_map(analyze_year)
        .collect();
    Ok(res)
}

/// Attempts to scrape all degrees specified in [DEGREES_PATH]. Might fail when
/// reading or parsing the latter. Degrees for which scraping is not possible
/// do not belong to the output.
pub fn degrees() -> Result<Vec<Degree>, Box<dyn Error>> {
    let file = File::open(DEGREES_PATH)?;
    let json = from_reader(file)?;
    Ok(to_degrees(json))
}
