// SPDX-FileCopyrightText: 2024 Stefano Volpe <foxy@teapot.ovh>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use eyre::{Result, eyre};
use itertools::Itertools;
use reqwest::blocking::get;
use scraper::{Html, Selector};
use std::collections::HashMap;
use substring::Substring;

lazy_static::lazy_static! {
    static ref TITLE: Selector = Selector::parse("div#u-content-intro>h1").unwrap();
    static ref LANG: Selector = Selector::parse("li.language-en").unwrap();
    static ref DESC: Selector = Selector::parse("div.description-text").unwrap();
    static ref DESC_END_MARKER: HashMap<String,String> = [
        ("Numerical Computing".to_string(), "Teaching".to_string()),
        ("History of Informatics".to_string(), "Office".to_string()),
        ("*".to_string(), "Readings".to_string())
    ]
    .into();
    static ref PROF: Selector = Selector::parse("div.line:nth-child(1) > ul:nth-child(1) > li:nth-child(1) > a:nth-child(2)").unwrap();
}

fn get_eng_url(url: &str) -> Result<String> {
    if url.is_empty() {
        Ok("".to_string())
    } else {
        let res = get(url)?.text()?;
        let document = Html::parse_document(&res);
        let link_ite = document.select(&LANG).map(|x| x.inner_html()).next();
        link_ite.ok_or(eyre!("Cannot get english url"))
    }
}

pub fn get_desc_teaching_page(slug: &String, year: &u32, url: &str) -> Result<String> {
    let eng_url_temp = get_eng_url(url)?;
    let start = eng_url_temp.find("http").unwrap_or(0);
    let tmp = eng_url_temp.substring(start, eng_url_temp.len());
    let end = tmp.find('\"').unwrap_or(0);
    let teaching_url = tmp.substring(0, end);
    let eng_page = get(teaching_url)?.text()?;
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
        .ok_or(eyre!("Cannot parse teaching title"))?
        .text()
        .join("");
    let full_description = document
        .select(&DESC)
        .next()
        .ok_or(eyre!("Cannot parse teaching description"))?
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
            f.ok_or(eyre!(
                "No description end marker defined for this page content"
            ))? - 2,
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
