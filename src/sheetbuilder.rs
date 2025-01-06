use crate::api::CensusReader;
use crate::archiveparse::Archive;
use anyhow::Result;
use dialoguer::Confirm;
use rust_xlsxwriter::{ExcelDateTime, Format, Workbook};
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn normalize(string: &String) -> String {
    let new_string = string.trim().to_lowercase().replace(" ", "_");
    new_string
}

pub struct SheetBuilder {
    http_client: ureq::Agent,
}

impl SheetBuilder {
    pub fn new(main_nation: &String) -> Result<Self> {
        // Build user-agent to identify user and tool to admin
        let user_agent = format!(
            "YeetSeeker/{}; Developer=Volstrostia; User={}",
            env!("CARGO_PKG_VERSION"),
            normalize(main_nation)
        );

        // Supply user-agent to HTTP client
        let http_client = ureq::builder().user_agent(&user_agent).build();

        // https://www.nationstates.net/cgi-bin/api.cgi?nation=volstrostia
        // Use our agent to check if the nation exists, and bubble up an error if not
        // This requires the user to use a valid nation to use the tool
        let request_url = format!(
            "https://www.nationstates.net/cgi-bin/api.cgi?nation={}",
            &main_nation
        );

        // This value will bubble up an error if the nation is not found
        let _existance_check = http_client.get(&request_url).call()?;

        // If we're here, we passed the API check, and are ready to use the client
        Ok(Self { http_client })
    }

    // Check if we need to perform an update
    pub fn check_archive(&self) -> bool {
        // No file, obviously we need an update
        if !Path::new("nations.xml.gz").exists() {
            true
        } else {
            println!("It appears you already have a nations.xml.gz file");

            let do_update = Confirm::new()
                .with_prompt("Would you like to replace it with the latest one?")
                .interact()
                .unwrap();

            do_update
        }
    }

    // Update the archive
    pub fn update_archive(&self) -> Result<()> {
        println!("Downloading nations archive from NationStates mainframe");
        // Open a file to save to
        let mut output_file = std::fs::File::create("nations.xml.gz")?;

        // Request nations.xml.gz
        let nations_response = self
            .http_client
            .get("https://www.nationstates.net/pages/nations.xml.gz")
            .call()?;

        println!("Saving nations.xml.gz");
        std::io::copy(&mut nations_response.into_reader(), &mut output_file)?;

        Ok(())
    }

    // Build the actual sheet from nations.xml.gz
    pub fn build_sheet(&self, region: &String) -> Result<Sheet> {
        // Parse sheet into SheetRows
        // Compile SheetRows into Sheet
        // Sheet includes additional metadata
        // Sheet can be exported to xlsx format

        println!("Fetching residency from API");
        // At this point, we have nations.xml.gz set up - reuse our HTTP agent and open a thread
        // to poll the API in the background while we parse the XML dump
        let region = normalize(region);
        let api_region = region.clone();
        let api_agent = CensusReader::with_agent(&self.http_client);
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            // 80 is the code for Residency
            let results = api_agent.get_rankings(&api_region, &80);
            // Send results down the pipeline for use later
            tx.send(results).unwrap();
        });

        // Now that the thread is running, kick off the parsing
        let archive = Archive::from(&"nations.xml.gz".to_string())?;

        // Fetch the rankings value from the other thread - waiting for it to complete if necessary
        println!("Waiting for residency check to complete");
        let residency_rankings = rx.recv()??;
        println!("Residency fetch has completed");

        // Remove nations from the archive that are not in the target region
        let mut nations = archive.nations;
        nations.retain(|nation| normalize(&nation.region) == region);

        let mut sheet_rows = Vec::new();
        let mut total_influence = 0.0;

        for nation in nations {
            let normalized_name = normalize(&nation.name);

            let residency_ranking = residency_rankings
                .iter()
                .find(|&x| normalize(&x.nation) == normalized_name);
            let residency = match residency_ranking {
                Some(value) => value.score,
                None => -1.0, // Target was not in region at the time
            };

            let endorsement_count: u64;
            if nation.endorsements_list.is_empty() {
                endorsement_count = 0;
            } else {
                // Count commas used to seperate endorsements
                // foo has zero, so one endorsement
                // foo,bar has one, and two endos
                // and so on and so forth
                endorsement_count = (1 + nation
                    .endorsements_list
                    .chars()
                    .filter(|c| *c == ',')
                    .count()) as u64;
            };

            let wa_member = match nation.wa_status.as_str() {
                "Non-member" => false,
                "WA Member" => true,
                "WA Delegate" => true,
                _ => {
                    panic!("Unrecognized value for WA membership! File a bug report!")
                }
            };

            total_influence += nation.influence;

            sheet_rows.push(SheetRow {
                nation: normalized_name,
                influence: nation.influence,
                endorsement_count,
                wa_member,
                residency,
                last_login: nation.last_login,
            });
        }

        // Sort rows by name - easy key for stinky human brains
        sheet_rows.sort_by(|x, y| x.nation.cmp(&y.nation));

        Ok(Sheet {
            region,
            total_influence,
            sheet_rows,
        })
    }
}

pub struct SheetRow {
    nation: String,
    influence: f64,
    endorsement_count: u64,
    wa_member: bool,
    residency: f64,
    last_login: u64,
}

pub struct Sheet {
    region: String,
    total_influence: f64,
    sheet_rows: Vec<SheetRow>,
}

impl Sheet {
    pub fn get_default_name(&self) -> String {
        let hms = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        let title = format!("region_{}_{}.xlsx", &self.region, hms);
        title
    }

    pub fn export(&self, filename: &String) -> Result<()> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        let decimal_format = Format::new().set_num_format("0.0000");
        let bold_format = Format::new().set_bold();
        let date_format = Format::new().set_num_format("yyyy-mm-dd hh:mm:ss");

        worksheet.write_with_format(0, 0, "URL", &bold_format)?;
        worksheet.write_with_format(0, 1, "Name", &bold_format)?;
        worksheet.write_with_format(0, 2, "Influence", &bold_format)?;
        worksheet.write_with_format(0, 3, "Endorsements", &bold_format)?;
        worksheet.write_with_format(0, 4, "WA Status", &bold_format)?;
        worksheet.write_with_format(0, 5, "Residency", &bold_format)?;
        worksheet.write_with_format(0, 6, "Last Login", &bold_format)?;

        worksheet.write_with_format(0, 8, "Total Influence:", &bold_format)?;
        worksheet.write(0, 9, self.total_influence)?;
        worksheet.write_with_format(1, 8, "Generated at:", &bold_format)?;
        let generated_time = ExcelDateTime::from_timestamp(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time travelers may not use this program")
                .as_secs() as i64,
        )?;
        worksheet.write_with_format(1, 9, generated_time, &date_format)?;
        worksheet.write_with_format(2, 8, "Generated with:", &bold_format)?;
        worksheet.write(2, 9, format!("YeetSeeker v{}", env!("CARGO_PKG_VERSION")))?;

        let mut row = 1;
        for nation in &self.sheet_rows {
            worksheet.write(
                row,
                0,
                rust_xlsxwriter::Url::new(format!(
                    "https://www.nationstates.net/nation={}",
                    &nation.nation
                )),
            )?;
            worksheet.write(row, 1, &nation.nation)?;
            worksheet.write_with_format(row, 2, nation.influence, &decimal_format)?;
            worksheet.write(row, 3, nation.endorsement_count)?;
            worksheet.write(
                row,
                4,
                match &nation.wa_member {
                    true => "Member",
                    false => "Non-member",
                },
            )?;
            worksheet.write_with_format(row, 5, nation.residency, &decimal_format)?;
            let last_login = ExcelDateTime::from_timestamp(nation.last_login as i64)?;
            worksheet.write_with_format(row, 6, last_login, &date_format)?;

            row += 1;
        }

        workbook.save(&filename)?;

        Ok(())
    }
}
