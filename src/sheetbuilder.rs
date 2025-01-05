use crate::archiveparse::Archive;
use anyhow::Result;
use dialoguer::Confirm;
use std::path::Path;

pub fn normalize(string: &String) -> String { 
    let new_string = string.trim().to_lowercase().replace(" ","_");
    new_string
}

pub struct SheetBuilder { 
    http_client: ureq::Agent
}

impl SheetBuilder { 
    pub fn new(main_nation: &String) -> Result<Self> { 
        // Build user-agent to identify user and tool to admin
        let user_agent = format!("YeetSeeker/{}; Developer=Volstrostia; User={}",
            env!("CARGO_PKG_VERSION"),
            normalize(main_nation)
        );

        // Supply user-agent to HTTP client
        let http_client = ureq::builder()
            .user_agent(&user_agent)
            .build();

        // https://www.nationstates.net/cgi-bin/api.cgi?nation=volstrostia
        // Use our agent to check if the nation exists, and bubble up an error if not
        // This requires the user to use a valid nation to use the tool
        let request_url = format!("https://www.nationstates.net/cgi-bin/api.cgi?nation={}", &main_nation);

        // This value will bubble up an error if the nation is not found
        let _existance_check = http_client.get(&request_url).call()?;

        // If we're here, we passed the API check, and are ready to use the client
        Ok(Self { 
            http_client
        })
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
        let nations_response = self.http_client.get("https://www.nationstates.net/pages/nations.xml.gz").call()?;

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

        let archive = Archive::from(&"nations.xml.gz".to_string())?;
        // Nation { name: "Durmengrad at Updates End", wa_status: "Non-member", endorsements_list: "", region: "Warzone Trinidad", influence: 14.0, last_login: 1734904321 }] }
        // TODO: Build API agent (and by build I mean steal it from Influencea) to fetch residency
        // TODO: Boil down sheet data to only nations in the target region
        // TODO: Compile residency struct and sheet data into Sheet Rows
        // TODO: Build Sheet implementation to export SheetRows to xlsx
    }
}

pub struct SheetRow { 
    nation: String,
    influence: f64,
    endorsement_count: u64,
    wa_member: bool,
    residency: f64,
    last_login: u64
}

pub struct Sheet {
    region: String,
    total_influence: f64,
    sheet_rows: Vec<SheetRow>
}

impl Sheet { 
    /*
    pub fn new(region: &String, rows: Vec<SheetRow>) -> Self { 
        Self {

        }
    }*/
}
