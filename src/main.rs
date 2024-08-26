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
        Some(value) => value,
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

    let migration_history: Vec<String> = if let Ok(value) =
        client.query("SELECT * FROM public.migration_history", &[])
    {
        value.iter().map(|x| x.get("migration_id")).collect()
    } else {
        if let Err(e) =
                client.execute("CREATE TABLE IF NOT EXISTS public.migration_history (migration_id varchar(255) primary key)", &[])
            {
                eprintln!("Error while creating table public.migration_history : {e}");
                std::process::exit(1);
            } else {
                vec![]
            }
    };

    let mut not_applied_migrations: Vec<&String> = files
        .iter()
        .filter(|x| !migration_history.contains(x))
        .collect();

    not_applied_migrations.sort();

    for migration in &not_applied_migrations {
        let script = match fs::read_to_string(format!("migrations/{migration}")) {
            Ok(value) => value,
            Err(e) => {
                eprint!("Failed to apply migration {migration} : {e}");
                std::process::exit(1);
            }
        };

        let mut transaction = match client.transaction() {
            Ok(value) => value,
            Err(e) => {
                eprint!("Failed to apply migration {migration} : {e}");
                std::process::exit(1);
            }
        };

        if let Err(e) = transaction.execute(&script, &[]) {
            eprint!("Failed to apply migration {migration} : {e}");
            std::process::exit(1);
        }

        if let Err(e) = transaction.execute(
            "INSERT INTO public.migration_history VALUES ($1)",
            &[migration],
        ) {
            eprint!("Failed to apply migration {migration} : {e}");
            std::process::exit(1);
        }

        if let Err(e) = transaction.commit() {
            eprint!("Failed to apply migration {migration} : {e}");
            std::process::exit(1);
        }
    }

    if not_applied_migrations.is_empty() {
        println!("No migrations to apply! Exiting...");
    } else {
        println!(
            "Successfully applied migrations: {:?}",
            not_applied_migrations
        );
    }
}

fn read_migration_files() -> Option<Vec<String>> {
    if let Ok(value) = fs::read_dir("./migrations") {
        //TODO: Fix this unwrap
        let file_names: Vec<String> = value
            .filter_map(|x| {
                let file_name = x
                    .unwrap()
                    .path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();

                let valid_migration = file_name
                    .chars()
                    .rev()
                    .last()
                    .is_some_and(|x| x.is_numeric());

                if valid_migration {
                    Some(file_name)
                } else {
                    None
                }
            })
            .collect();

        if file_names.is_empty() {
            None
        } else {
            Some(file_names)
        }
    } else {
        println!("No \"migrations\" directory was found, creating one...");

        if let Err(e) = create_dir("./migrations") {
            eprintln!("Error creating \"migrations\" directory : {e}")
        }
        None
    }
}
