// SPDX-FileCopyrightText: 2024 Stefano Volpe <foxy@teapot.ovh>
//
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod teachings;
pub mod year;

use eyre::{Result, eyre};
use itertools::Itertools;
use lazy_static::lazy_static;
use log::{info, warn};
use regex::Regex;
use reqwest::blocking::get;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json::from_reader;
use std::fs::File;
use teachings::get_desc_teaching_page;
use year::current_academic_year;

lazy_static! {
    static ref TABLE: Selector = Selector::parse("td.title").unwrap();
    static ref MISSING_TRANSLATIONS: [(String, String); 5] = [
        ("BASI DI DATI".to_string(), "DATABASES".to_string()),
        (
            "INTRODUZIONE ALL'APPRENDIMENTO AUTOMATICO".to_string(),
            "Introduction to machine learning".to_string()
        ),
        ("FONDAMENTI DI".to_string(), "".to_string()),
        (
            "Learning outcomes".to_string(),
            "=== Learning outcomes".to_string()
        ),
        (
            "Teaching contents".to_string(),
            "=== Teaching contents".to_string()
        )
    ];
}

#[derive(Deserialize, Debug, Clone)]
struct Predegree {
    id: String,
    name: String,
    code: String,
}

pub struct Degree {
    pub name: String,
    pub slug: String,
    pub url: String,
}

const DEGREES_PATH: &str = "config/degrees.json";

fn to_lowercase_maybe(s: String, b: bool) -> String {
    if b { s.to_lowercase() } else { s }
}

fn parse_degree(predegree: &Predegree, academic_year: u32) -> Option<Degree> {
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
            url: format!(
                "https://corsi.unibo.it/{degree_type}/{unibo_slug}/insegnamenti/piano/{academic_year}/{code}/000/{academic_year}"
            ),
        })
    }
}

fn to_degrees(predegrees: Vec<Predegree>) -> Vec<Degree> {
    let academic_year = current_academic_year();
    predegrees
        .iter()
        .filter_map(|predegree| parse_degree(predegree, academic_year))
        .collect()
}

pub fn analyze_degree(degree: &Degree) -> Result<String> {
    let Degree {
        slug: _slug,
        name,
        url,
    } = degree;
    info!("{name} [{url}]");
    let res = get(url).map_err(|e| eyre!("\tNetwork error: {e}"))?;
    let res2 = res
        .error_for_status()
        .map_err(|e| eyre!("\tServer error: {e}"))?;
    let text = res2.text().map_err(|e| eyre!("\tDecoding error: {e}"))?;
    let document = Html::parse_document(&text);
    let title_list = document.select(&TABLE);
    let buf = format!("= {name}\n\n")
        + title_list
            .filter_map(|item| {
                let a_el = item
                    .children()
                    .filter_map(|f| f.value().as_element())
                    .find(|r| r.name() == "a")
                    .and_then(|a_el| a_el.attr("href"));
                let temp_name = item.text().join("");
                let name = temp_name.trim();
                info!("\tVisiting {name}");
                match a_el {
                    Some(link) => {
                        let teaching_desc = get_desc_teaching_page(link);
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
                                warn!("\t\tCannot get description: {e:?}");
                                None
                            }
                        }
                    }
                    None => {
                        warn!("\t\tMissing link: {name}");
                        None
                    }
                }
            })
            .join("")
            .as_str();
    Ok(buf)
}

pub fn degrees() -> Result<Vec<Degree>> {
    match File::open(DEGREES_PATH) {
        Ok(file) => match from_reader(file) {
            Ok(json) => Ok(to_degrees(json)),
            Err(error) => Err(eyre!("Parsing {DEGREES_PATH}: {error:?}")),
        },
        Err(error) => Err(eyre!("Reading {DEGREES_PATH:?}: {error:?}")),
    }
}
