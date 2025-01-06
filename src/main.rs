mod api;
mod archiveparse;
mod sheetbuilder;

use clap::Parser;
use sheetbuilder::SheetBuilder;
use std::io;
use std::io::Write;

#[derive(Parser)]
struct Args {
    /// User's main nation
    main_nation: Option<String>,
    /// Region to compile sheet for
    region: Option<String>,
    /// Clobber nations.xml.gz
    #[arg(short, long, default_value_t = false)]
    force_update: bool,
}

fn main() {
    let args = Args::parse();

    let mut main_nation = String::new();
    let mut region = String::new();
    if args.main_nation.is_some() {
        main_nation = args.main_nation.unwrap()
    } else {
        print!("Main nation: ");
        io::stdout().flush().unwrap();
        let _ = io::stdin().read_line(&mut main_nation).unwrap();
    }

    if args.region.is_some() {
        region = args.region.unwrap();
    } else {
        print!("Region: ");
        io::stdout().flush().unwrap();
        let _ = io::stdin().read_line(&mut region).unwrap();
    }

    if let Ok(builder) = SheetBuilder::new(&main_nation) {
        // Ask user to overwrite file, or default true if not present
        let update_archive = args.force_update || builder.check_archive();

        // If we need to update the archive, do so
        if update_archive {
            builder.update_archive().expect("Could not update archive");
        }

        // Compile sheet from archive and write to file
        let sheet = builder
            .build_sheet(&region)
            .expect("Could not build sheet from archive");
        // TODO: Export sheet to chosen title
        let outfile = sheet.get_default_name();
        println!("Exporting sheet to {outfile}");
        match sheet.export(&outfile) {
            Ok(_) => println!("Done! File is saved to {outfile}"),
            Err(reason) => eprintln!("Could not save file: {reason}"),
        }
    } else {
        eprintln!("Invalid user-agent provided. Shame on you.");
    };
}
