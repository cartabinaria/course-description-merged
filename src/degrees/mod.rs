// SPDX-FileCopyrightText: 2024 Stefano Volpe <foxy@teapot.ovh>
//
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod teachings;
pub mod year;

use itertools::Itertools;
use regex::Regex;
use reqwest::blocking::get;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json::from_reader;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::sync::LazyLock;
use teachings::get_desc_teaching_page;
use year::current_academic_year;

static TABLE: LazyLock<Selector> = LazyLock::new(|| Selector::parse("td.title").unwrap());
static FIRST_LINK: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(".no-bullet > li:first-child > a").unwrap());
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

#[derive(Deserialize, Debug, Clone)]
struct Predegree {
    id: String,
    name: String,
    code: String,
}

pub struct Degree {
    pub name: String,
    pub slug: String,
    pub year_urls: HashMap<u32, String>, // k: year, v: Url
}

const DEGREES_PATH: &str = "config/degrees.json";

fn to_lowercase_maybe(s: String, b: bool) -> String {
    if b { s.to_lowercase() } else { s }
}

// Given a university degree (with name and type), return the url of the
// Course Structures of the last 3 completed years.
fn get_course_structure_urls(degree_type: &str, degree_name: String) -> HashMap<u32, String> {
    let end_year = current_academic_year() - 2; // too new, not useful 
    let start_year = end_year - 1; // get 3 years in total
    (start_year..=end_year)
        .filter_map(|year| {
            let url = format!(
                "https://corsi.unibo.it/{degree_type}/{degree_name}/insegnamenti?year={year}"
            );
            eprintln!("Visiting: {url}");

            let res = get(&url).ok()?.error_for_status().ok()?;
            let text = res.text().ok()?;
            let document = Html::parse_document(&text);
            let link = document.select(&FIRST_LINK).next()?;
            let href = link.value().attr("href")?.to_string();
            eprintln!("Got link: {href}");
            Some((year, href))
        })
        .collect()
}

fn parse_degree(predegree: &Predegree) -> Option<Degree> {
    let Predegree { name, id, code } = predegree;
    if name.is_empty() || id.is_empty() || code.is_empty() {
        None
    } else {
        let unibo_slug = to_lowercase_maybe(
            Regex::new(r"( (e|per il|in) )|Magistrale|Master")
                .unwrap()
                .replace_all(name, "")
                .to_string(),
            !code.eq("9254/000"),
        )
        // AI's slug is kebab-case
        .replace(' ', if code.eq("9063/000") { "-" } else { "" });
        let degree_type = if name.contains("Magistrale") || name.contains("Master") {
            "magistrale"
        } else {
            "laurea"
        };
        Some(Degree {
            name: name.to_string(),
            slug: id.to_string(),
            year_urls: get_course_structure_urls(degree_type, unibo_slug),
        })
    }
}

fn to_degrees(predegrees: Vec<Predegree>) -> Vec<Degree> {
    predegrees.iter().filter_map(parse_degree).collect()
}

pub fn analyze_degree(degree: &Degree) -> Result<HashMap<u32, String>, Box<dyn Error>> {
    let Degree {
        slug,
        name,
        year_urls,
    } = degree;

    let res = year_urls
        .iter()
        .map(|(year, url)| {
            eprintln!("Analysing {year} link: {url}");
            let res = get(url)
                .map_err(|e| format!("\tNetwork error: {e}"))
                .unwrap();
            let res2 = res
                .error_for_status()
                .map_err(|e| format!("\tServer error: {e}"))
                .unwrap();
            let text = res2
                .text()
                .map_err(|e| format!("\tDecoding error: {e}"))
                .unwrap();
            let document = Html::parse_document(&text);
            let title_list = document.select(&TABLE);
            let buf = format!("= {name} ({year})\n\n")
                + title_list
                    .filter_map(|item| {
                        let a_el = item
                            .children()
                            .filter_map(|f| f.value().as_element())
                            .find(|r| r.name() == "a")
                            .and_then(|a_el| a_el.attr("href"));
                        let temp_name = item.text().join("");
                        let name = temp_name.trim();
                        eprintln!("\tVisiting {name}");
                        match a_el {
                            Some(link) => {
                                let teaching_desc = get_desc_teaching_page(slug, year, link);
                                match teaching_desc {
                                    Ok(desc) => {
                                        let entry_doc = "\n".to_string() + desc.as_str();
                                        Some(MISSING_TRANSLATIONS.iter().fold(
                                            entry_doc,
                                            |entry_doc, (source, replacement)| {
                                                entry_doc.replace(source, replacement)
                                            },
                                        ))
                                    }
                                    Err(e) => {
                                        eprintln!("\t\tWARN: Cannot get description: {e:?}");
                                        None
                                    }
                                }
                            }
                            None => {
                                eprintln!("\t\tWARN: Missing link: {name}");
                                None
                            }
                        }
                    })
                    .join("")
                    .as_str();
            (*year, buf)
        })
        .collect();

    Ok(res)
}

pub fn degrees() -> Result<Vec<Degree>, Box<dyn Error>> {
    let file = File::open(DEGREES_PATH)?;
    let json = from_reader(file)?;
    Ok(to_degrees(json))
}
