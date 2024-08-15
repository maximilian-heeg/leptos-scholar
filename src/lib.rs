use anyhow::Result;
use reqwest::StatusCode;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

/// Represents the scraped author information
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorInfo {
    /// Author name
    name: String,
    /// Total number of citations
    total: usize,
    /// h-index of the author
    h_index: usize,
    /// i10-index of the author
    i10_index: usize,
    /// Yearly citation counts
    #[serde(rename = "years")]
    yearly_citations: BTreeMap<usize, usize>,
}

/// Custom error types for the scraper
#[derive(Error, Debug, Serialize, Deserialize)]
enum ScraperError {
    #[error("Website not found. Check the ID.")]
    InvalidId,
    #[error("Failed to find the citation table on the website")]
    TableNotFound,
    #[error("Failed to find the name on the website")]
    NameNotFound,
    #[error("Failed to parse value: {0}")]
    ParseError(String),
    #[error("Insufficient data: expected 3 values, found {0}")]
    InsufficientData(usize),
    #[error("Failed to parse year: {0}")]
    YearParseError(String),
    #[error("Failed to parse citation count: {0}")]
    CitationParseError(String),
}

/// Fetches the HTML content of the author's Google Scholar page
///
/// # Arguments
///
/// * `authorid` - The Google Scholar ID of the author
///
/// # Returns
///
/// * `Result<Html>` - The parsed HTML document
async fn fetch_page(authorid: &str) -> Result<Html> {
    let response = reqwest::get(&format!(
        "https://scholar.google.com/citations?user={authorid}",
    ))
    .await?;

    match response.status() {
        StatusCode::OK => {
            let html_content = response.text().await?;
            let document = Html::parse_document(&html_content);
            Ok(document)
        }
        _ => Err(ScraperError::InvalidId.into()),
    }
}

/// Extracts the author's main citation information (Name, total, h-index, i10-index)
///
/// # Arguments
///
/// * `document` - The parsed HTML document of the author's page
///
/// # Returns
///
/// * `Result<(String, usize, usize, usize)>` - A tuple containing (name, total citations, h-index, i10-index)
fn extract_author_info(document: &Html) -> Result<(String, usize, usize, usize)> {
    let table_selector = Selector::parse("table#gsc_rsb_st").unwrap();
    let row_selector = Selector::parse("tr > td:nth-child(2)").unwrap();
    let name_selector = Selector::parse("div#gsc_prf_in").unwrap();

    let table = document
        .select(&table_selector)
        .next()
        .ok_or(ScraperError::TableNotFound)?;

    let values: Result<Vec<usize>, _> = table
        .select(&row_selector)
        .map(|element| {
            element
                .inner_html()
                .parse()
                .map_err(|_| ScraperError::ParseError(element.inner_html()))
        })
        .collect();

    let values = values?;

    if values.len() < 3 {
        return Err(ScraperError::InsufficientData(values.len()).into());
    }

    let name = document
        .select(&name_selector)
        .next()
        .ok_or(ScraperError::NameNotFound)?
        .inner_html();

    Ok((name, values[0], values[1], values[2]))
}

/// Extracts the yearly citation counts
///
/// # Arguments
///
/// * `document` - The parsed HTML document of the author's page
///
/// # Returns
///
/// * `Result<BTreeMap<usize, usize>>` - A map of years to citation counts
fn extract_citations(document: &Html) -> Result<BTreeMap<usize, usize>> {
    let div_selector = Selector::parse("div.gsc_md_hist_w > div.gsc_md_hist_b").unwrap();
    let year_selector = Selector::parse("span.gsc_g_t").unwrap();
    let citation_selector = Selector::parse("a.gsc_g_a > span.gsc_g_al").unwrap();

    let div = document
        .select(&div_selector)
        .next()
        .ok_or(ScraperError::TableNotFound)?;

    let years = div.select(&year_selector);
    let citations = div.select(&citation_selector);

    years
        .zip(citations)
        .map(|(y, c)| {
            let year = y
                .inner_html()
                .parse()
                .map_err(|_| ScraperError::YearParseError(y.inner_html()))?;
            let citations = c
                .inner_html()
                .parse()
                .map_err(|_| ScraperError::CitationParseError(c.inner_html()))?;
            Ok((year, citations))
        })
        .collect()
}

/// Main function to run the scraper
///
/// This function fetches the author's page,
/// extracts citation information, and returns the results as YAML.
pub async fn fetch_info(author_id: String) -> Result<String> {
    let document = fetch_page(&author_id).await?;

    let (name, total, h_index, i10_index) = extract_author_info(&document)?;
    let yearly_citations = extract_citations(&document)?;

    let author_info = AuthorInfo {
        name,
        total,
        h_index,
        i10_index,
        yearly_citations,
    };

    let res = serde_yaml::to_string(&author_info)?;
    Ok(res)
}
