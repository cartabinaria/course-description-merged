// SPDX-FileCopyrightText: 2024 Stefano Volpe <foxy@teapot.ovh>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use itertools::Itertools;
use reqwest::Url;
use reqwest::blocking::get;
use scraper::{ElementRef, Html, Selector};
use std::{collections::HashMap, error::Error, sync::LazyLock};
use substring::Substring;

/// Selector for the teaching title
static TITLE: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("div#u-content-intro>h1").unwrap());
/// Selector for the teaching language
static LANG: LazyLock<Selector> = LazyLock::new(|| Selector::parse("li.language-en").unwrap());
/// Selector for the teaching description
static DESC: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("div.description-text").unwrap());
/// Catalogue searches can contain several modules of an integrated course.
static CATALOGUE: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("body.unibo-model-lista-insegnamenti").unwrap());
static COURSE_LINK: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(".unordered-results .teachingname > a").unwrap());
/// A dictionary specifying, for each teaching, a keyword marking the end of its
/// description.
static DESC_END_MARKER: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    [
        ("Numerical Computing".to_string(), "Teaching".to_string()),
        ("History of Informatics".to_string(), "Office".to_string()),
        ("*".to_string(), "Readings".to_string()),
    ]
    .into()
});
// static PROF: LazyLock<Selector> = LazyLock::new(|| {
//     Selector::parse("div.line:nth-child(1) > ul:nth-child(1) > li:nth-child(1) > a:nth-child(2)")
//         .unwrap()
// });

/// Scrapes a webpage to look for the English counterpart.
fn get_eng_url(url: &str) -> Result<String, Box<dyn Error>> {
    if url.is_empty() {
        Ok("".to_string())
    } else {
        let res = get(url)?.text()?;
        let mut document = Html::parse_document(&res);
        if document.select(&CATALOGUE).next().is_some() {
            let teaching_url = teaching_page_link(&document, url)?;
            let res = get(&teaching_url)?.text()?;
            document = Html::parse_document(&res);
            if document.select(&CATALOGUE).next().is_some() {
                return Err("Teaching link points to another catalogue page".into());
            }
        }
        teaching_page_link(&document, url)
    }
}

/// Follows a catalogue's requested module, or a teaching's English counterpart.
fn teaching_page_link(document: &Html, url: &str) -> Result<String, Box<dyn Error>> {
    if document.select(&CATALOGUE).next().is_some() {
        let url = Url::parse(url)?;
        let code = url
            .query_pairs()
            .find(|(key, _)| key == "codiceMateria")
            .map(|(_, code)| code)
            .ok_or("Catalogue URL has no teaching code")?;
        let mut matches = document.select(&COURSE_LINK).filter(|link| {
            link.text()
                .join("")
                .split_once(" - ")
                .is_some_and(|(candidate, _)| candidate.trim() == code)
        });
        let teaching = matches
            .next()
            .ok_or("Teaching code not found in catalogue")?;
        if matches.next().is_some() {
            return Err("Multiple catalogue entries match the teaching code".into());
        }
        return teaching
            .value()
            .attr("href")
            .map(str::to_string)
            .ok_or("Catalogue teaching has no link".into());
    }
    document
        .select(&LANG)
        .next()
        .and_then(|language| {
            language
                .children()
                .filter_map(ElementRef::wrap)
                .find(|child| child.value().name() == "a")
        })
        .and_then(|link| link.value().attr("href"))
        .map(str::to_string)
        .ok_or("Error: Cannot get english url".into())
}

/// Scrapes a single teaching page.
pub fn get_desc_teaching_page(
    slug: &String,
    year: &u32,
    url: &str,
) -> Result<String, Box<dyn Error>> {
    let teaching_url = get_eng_url(url)?;
    let eng_page = get(&teaching_url)?.text()?;
    let document = Html::parse_document(&eng_page);
    parse_teaching_page(slug, year, &teaching_url, &document)
}

fn parse_teaching_page(
    slug: &String,
    year: &u32,
    teaching_url: &str,
    document: &Html,
) -> Result<String, Box<dyn Error>> {
    // let teacher = document
    //     .select(&PROF)
    //     .next()
    //     .ok_or(eyre!("Cannot parse professor name"))?
    //     .text()
    //     .join("");
    let teaching_title = document
        .select(&TITLE)
        .next()
        .ok_or("Cannot parse teaching title")?
        .text()
        .join("");
    let full_description = document
        .select(&DESC)
        .next()
        .ok_or("Cannot parse teaching description")?
        .text()
        .join("");
    let i = full_description
        .find("Learning outcomes")
        .unwrap_or(full_description.len());
    let backup = || {
        DESC_END_MARKER
            .get("*")
            .and_then(|marker| full_description.find(marker))
    };
    let f = DESC_END_MARKER
        .iter()
        .find(|(pattern, _)| teaching_title.contains(pattern.as_str()))
        .map(|(_, marker)| {
            full_description
                .find(marker)
                .unwrap_or(full_description.len())
        })
        .or_else(backup);
    let filtered_description = full_description
        .substring(
            i,
            f.ok_or("No description end marker defined for this page content")? - 2,
        )
        .split('\n')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .join("\n\n");
    Ok(format!(
        "\n== {}[{}]\n\nlink:degree-{}-{}.pdf[PDF], xref:degree-{}-{}.adoc[ADOC].\n\n{}", // Professor: {}
        teaching_url,
        teaching_title.as_str(),
        slug,
        *year,
        slug,
        *year,
        // teacher,
        filtered_description.trim()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ISSUE_URL: &str = "https://www.unibo.it/it/studiare/insegnamenti-competenze-trasversali-moocs/insegnamenti?codiceMateria=08574&annoAccademico=2023&codiceCorso=8014&single=True&search=True";
    const COURSE_IT: &str = "https://www.unibo.it/it/studiare/insegnamenti-competenze-trasversali-moocs/insegnamenti/insegnamento/2023/320660";
    const COURSE_EN: &str = "https://www.unibo.it/en/study/course-units-transferable-skills-moocs/course-unit-catalogue/course-unit/2023/320660";
    const ENGLISH_LINK: &str = "https://www.unibo.it/it/studiare/insegnamenti-competenze-trasversali-moocs/insegnamenti/insegnamento/@@multilingual-selector/c49b978e935747049f9cebe759020dc7/en?post_path=/2023/320660";
    const CATALOGUE_IT: &str = include_str!("fixtures/catalogue-it.html");
    const CATALOGUE_EN: &str = include_str!("fixtures/catalogue-en.html");
    const TEACHING_IT: &str = include_str!("fixtures/operating-systems-it.html");
    const TEACHING_EN: &str = include_str!("fixtures/operating-systems-en.html");

    #[test]
    fn catalogue_selects_requested_module() {
        for (html, expected) in [(CATALOGUE_IT, COURSE_IT), (CATALOGUE_EN, COURSE_EN)] {
            let document = Html::parse_document(html);
            assert_eq!(teaching_page_link(&document, ISSUE_URL).unwrap(), expected);
        }
    }

    #[test]
    fn catalogue_does_not_select_first_module() {
        let document = Html::parse_document(CATALOGUE_IT);
        let url = ISSUE_URL.replace("08574", "58348");
        assert_eq!(
            teaching_page_link(&document, &url).unwrap(),
            COURSE_IT.replace("320660", "320661")
        );
    }

    #[test]
    fn catalogue_rejects_missing_or_unknown_code() {
        let document = Html::parse_document(CATALOGUE_IT);
        for url in [
            ISSUE_URL.split('?').next().unwrap().to_string(),
            ISSUE_URL.replace("08574", "00000"),
        ] {
            assert!(teaching_page_link(&document, &url).is_err());
        }
    }

    #[test]
    fn catalogue_rejects_ambiguous_matches() {
        // Deliberately mutate a real fixture to model two matching modules.
        let html = CATALOGUE_IT.replace("58348 -", "08574 -");
        let document = Html::parse_document(&html);
        assert!(teaching_page_link(&document, ISSUE_URL).is_err());
    }

    #[test]
    fn catalogue_rejects_empty_results() {
        // Synthetic mutation: the catalogue layout without matching entries.
        let html = CATALOGUE_IT.replace("class=\"teachingname\"", "class=\"no-result\"");
        let document = Html::parse_document(&html);
        assert_eq!(
            teaching_page_link(&document, ISSUE_URL)
                .unwrap_err()
                .to_string(),
            "Teaching code not found in catalogue"
        );
    }

    #[test]
    fn standard_page_keeps_english_link() {
        let document = Html::parse_document(TEACHING_IT);
        assert_eq!(
            teaching_page_link(&document, COURSE_IT).unwrap(),
            ENGLISH_LINK
        );
    }

    #[test]
    fn standard_page_requires_english_link() {
        // Synthetic mutation: no English-language selector.
        let html = TEACHING_IT.replace("language-en", "language-it");
        let document = Html::parse_document(&html);
        assert_eq!(
            teaching_page_link(&document, COURSE_IT)
                .unwrap_err()
                .to_string(),
            "Error: Cannot get english url"
        );
    }

    #[test]
    fn language_link_decodes_query_entities() {
        // Synthetic mutation: a standard language link with an extra query field.
        let html = TEACHING_IT.replace("?post_path=", "?example=1&amp;post_path=");
        let document = Html::parse_document(&html);
        assert_eq!(
            teaching_page_link(&document, COURSE_IT).unwrap(),
            ENGLISH_LINK.replace("?post_path=", "?example=1&post_path=")
        );
    }

    #[test]
    fn issue_catalogue_resolves_to_english_description() {
        let course = teaching_page_link(&Html::parse_document(CATALOGUE_IT), ISSUE_URL).unwrap();
        assert_eq!(course, COURSE_IT);
        let english = teaching_page_link(&Html::parse_document(TEACHING_IT), &course).unwrap();
        assert_eq!(english, ENGLISH_LINK);
        let document = Html::parse_document(TEACHING_EN);
        let description = parse_teaching_page(
            &"informatica-management".to_string(),
            &2023,
            &english,
            &document,
        )
        .unwrap();
        assert!(
            description.starts_with(&format!("\n== {ENGLISH_LINK}[08574 - Operating Systems]\n"))
        );
        assert!(description.contains("link:degree-informatica-management-2023.pdf[PDF], xref:degree-informatica-management-2023.adoc[ADOC]."));
        assert!(description.contains("Learning outcomes\n\nAt the end of the course, the student knows the fundamental concepts"));
        assert!(
            description.contains("Course contents\n\n1: Introduction to operating systems (OS)")
        );
        assert!(
            description.ends_with(
                "See the course web page, from the lecturer's web page, for more details."
            )
        );
        assert!(!description.contains("Readings/Bibliography"));
        assert!(!description.contains("Lab-based Course"));
    }

    #[test]
    fn catalogue_is_not_a_description() {
        let document = Html::parse_document(CATALOGUE_EN);
        let error = parse_teaching_page(
            &"informatica-management".to_string(),
            &2023,
            ISSUE_URL,
            &document,
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "Cannot parse teaching description");
    }
}
