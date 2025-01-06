use anyhow::Result;
use flate2::read::GzDecoder;
use quick_xml::de::from_str;
use serde::Deserialize;
use spinoff::{spinners, Color, Spinner};
use std::fs;
use std::io::prelude::*;

/* This file contains a series of datastructures for working with nations.xml
 * The end functionality revolves around invoking the Archive::from() method against
 * a nations.xml.gz file. The resulting data structure can then be read in other contexts.
 * This is not meant to be a complete implementation of the nations.xml archive structure,
 * and some data is lost during ingestion. Please keep this in mind when repurposing this library.
 */

/*
 * The basic structure of nations.xml is
 * <NATIONS> - a root node containing several Nations
 *  <NATION> - Describes a single nation
 *   <NAME> - Nation name,
 *   <UNSTATUS> - Non-member, Member, Delegate
 *   <ENDORSEMENTS> - List of endorsing nations
 *   <REGION> - Current region
 *   <INFLUENCENUM> - Actual hard influence value
 *   <LASTLOGIN> - Timestamp of last login
 * This contains all information YeetSeeker needs, except residency
 * That is fetched with API calls after the sheet is compiled
 * The two can then be woven together after the fact
 */

// Read GZ file and output resultant XML
fn decompress_archive(path: &String) -> Result<String> {
    let archive_content = fs::File::open(path)?;
    let mut decoder = GzDecoder::new(archive_content);
    let mut output = String::new();
    decoder.read_to_string(&mut output)?;
    Ok(output)
}

#[derive(Debug, Deserialize)]
pub struct Nation {
    #[serde(rename = "NAME")]
    pub name: String,
    #[serde(rename = "UNSTATUS")]
    pub wa_status: String,
    #[serde(rename = "ENDORSEMENTS")]
    pub endorsements_list: String,
    #[serde(rename = "REGION")]
    pub region: String,
    #[serde(rename = "INFLUENCENUM")]
    pub influence: f64,
    #[serde(rename = "LASTLOGIN")]
    pub last_login: u64,
}

#[derive(Debug, Deserialize)]
pub struct Archive {
    // Weird and awful - why do we skip the entire NATIONS node?
    // Makes no sense. But whatever, it works!!! WOO!!!
    #[serde(rename = "NATION")]
    pub nations: Vec<Nation>,
}

impl Archive {
    pub fn from(archive: &String) -> Result<Self> {
        // Open archive and fetch XML contents
        let mut spinner = Spinner::new(
            spinners::Aesthetic,
            "Decompressing XML archive...",
            Color::Blue,
        );
        let decompressed_archive = decompress_archive(archive)?;
        spinner.success("Decompression complete!");

        let mut spinner = Spinner::new(spinners::Aesthetic, "Parsing XML archive...", Color::Blue);
        let archive: Archive = from_str(&decompressed_archive)?;
        spinner.success("Parsing complete!");

        Ok(archive)
    }
}
