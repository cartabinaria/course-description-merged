// SPDX-FileCopyrightText: 2024 Stefano Volpe <foxy@teapot.ovh>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use super::retry::get_status_checked_with_retry;
use itertools::Itertools;
use log::error;
use scraper::{Html, Selector};
use std::{collections::HashMap, error::Error, io::Error as IoError, sync::LazyLock};
use substring::Substring;

/// Selector for the teaching title
static TITLE: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("div#u-content-intro>h1").unwrap());
/// Selector for the teaching language
static LANG: LazyLock<Selector> = LazyLock::new(|| Selector::parse("li.language-en").unwrap());
/// Selector for the teaching description
static DESC: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("div.description-text").unwrap());
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
fn get_eng_url(url: &str, log_ctx: &str) -> Result<String, Box<dyn Error>> {
    if url.is_empty() {
        Ok("".to_string())
    } else {
        let res = get_status_checked_with_retry(url).map_err(|e| {
            let message = if let Some(status) = e.status() {
                format!(
                    "[{log_ctx}] HTTP {status} while requesting teaching language page {url}: {e}"
                )
            } else {
                format!(
                    "[{log_ctx}] Request error while requesting teaching language page {url}: {e}"
                )
            };
            error!("{message}");
            IoError::other(message)
        })?;
        let res = res.text()?;
        let document = Html::parse_document(&res);
        let link_ite = document.select(&LANG).map(|x| x.inner_html()).next();
        link_ite.ok_or("Error: Cannot get english url".into())
    }
}

/// Scrapes a single teaching page.
pub fn get_desc_teaching_page(
    slug: &String,
    year: &u32,
    url: &str,
) -> Result<String, Box<dyn Error>> {
    let log_ctx = format!("degree_slug={slug} year={year} teaching_listing_url={url}");
    let eng_url_temp = get_eng_url(url, &log_ctx)?;
    let start = eng_url_temp.find("http").unwrap_or(0);
    let tmp = eng_url_temp.substring(start, eng_url_temp.len());
    let end = tmp.find('\"').unwrap_or(0);
    let teaching_url = tmp.substring(0, end);
    let eng_page = get_status_checked_with_retry(teaching_url).map_err(|e| {
        let message = if let Some(status) = e.status() {
            format!(
                "[{log_ctx} teaching_page_url={teaching_url}] HTTP {status} while requesting teaching page {teaching_url}: {e}"
            )
        } else {
            format!(
                "[{log_ctx} teaching_page_url={teaching_url}] Request error while requesting teaching page {teaching_url}: {e}"
            )
        };
        error!("{message}");
        IoError::other(message)
    })?;
    let eng_page = eng_page.text()?;
    let document = Html::parse_document(&eng_page);
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
