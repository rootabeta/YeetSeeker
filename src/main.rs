use std::io;
use std::io::Write;
use yeetseeker::SheetBuilder;

fn main() {
    let mut main_nation = String::new();
    let mut region = String::new();

    print!("Main nation: ");
    io::stdout().flush().unwrap();
    let _ = io::stdin().read_line(&mut main_nation).unwrap();

    print!("Region: ");
    io::stdout().flush().unwrap();
    let _ = io::stdin().read_line(&mut region).unwrap();

    if let Ok(builder) = SheetBuilder::new(&main_nation) { 
        // Ask user to overwrite file, or default true if not present
        let update_archive = builder.check_archive();

        // If we need to update the archive, do so
        if update_archive { 
            builder.update_archive().expect("Could not update archive");
        }

        match builder.build_sheet(&region) { 
            Ok(sheet_path) => println!("Created sheet at {sheet_path}"),
            Err(reason) => eprintln!("Failed to create sheet:\n{reason}")
        }
    } else { 
        eprintln!("Invalid user-agent provided. Shame on you.");
    };
}
