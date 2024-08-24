use std::{
    env,
    fs::{self, create_dir},
};

use dotenv::dotenv;
use postgres::{Client, NoTls};

fn main() {
    if let Err(e) = dotenv() {
        eprintln!("Error while parsing environment variables : {e}");
        std::process::exit(1);
    }

    let database_url = match env::var("DATABASE_URL") {
        Ok(value) => value,
        Err(e) => {
            eprint!("Unable to find \"DATABASE_URL\", is your .env file setup correctly? : {e}");
            std::process::exit(1);
        }
    };

    let files = match read_migration_files() {
        Some(value) => {
            println!("{:?}", value);
            value
        }
        None => {
            println!("No migrations to apply! Exiting...");
            std::process::exit(0);
        }
    };

    let mut client = match Client::connect(&database_url, NoTls) {
        Ok(client) => client,
        Err(e) => {
            eprint!("Unable to connect to the database, is your connection string correct? : {e}");
            std::process::exit(1);
        }
    };

    let _ = client.prepare("");
    // let mut transaction = client.transaction().unwrap().execute("TODO", &[]).unwrap();
}

fn read_migration_files() -> Option<Vec<String>> {
    if let Ok(value) = fs::read_dir("./migrations") {
        //TODO: Fix this unwrap
        let file_names: Vec<String> = value
            .map(|x| x.unwrap().path().display().to_string())
            .collect();

        if !file_names.is_empty() {
            Some(file_names)
        } else {
            None
        }
    } else {
        println!("No \"migrations\" directory was found, creating one...");

        if let Err(e) = create_dir("./migrations") {
            eprintln!("Error creating \"migrations\" directory : {e}")
        }
        None
    }
}
