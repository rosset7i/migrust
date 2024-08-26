use std::{
    env,
    fs::{self, create_dir},
};

use dotenv::dotenv;
use postgres::{Client, NoTls};

fn main() {
    read_environment_variables();

    let database_url = get_database_url();

    let files = if let Some(value) = read_migration_files() {
        value
    } else {
        println!("No migrations to apply! Exiting...");
        std::process::exit(0);
    };

    let mut client = connect_to_client(&database_url);

    let migration_ids = if let Ok(rows) = get_migration_history(&mut client) {
        map_rows(rows)
    } else {
        create_migration_history(&mut client);
        vec![]
    };

    let not_applied_migrations = filter_not_applied_migrations(migration_ids, files);

    apply_migrations(&mut client, &not_applied_migrations);

    if not_applied_migrations.is_empty() {
        println!("No migrations to apply! Exiting...");
    } else {
        println!(
            "Successfully applied migrations: {:?}",
            not_applied_migrations
        );
    }
}

fn apply_migrations(client: &mut Client, not_applied_migrations: &[String]) {
    not_applied_migrations.iter().for_each(|migration| {
        let script = match fs::read_to_string(format!("migrations/{migration}")) {
            Ok(value) => value,
            Err(err) => {
                eprint!("Failed to apply migration {migration} : {err}");
                std::process::exit(1);
            }
        };

        let mut transaction = match client.transaction() {
            Ok(value) => value,
            Err(err) => {
                eprint!("Failed to apply migration {migration} : {err}");
                std::process::exit(1);
            }
        };

        if let Err(err) = transaction.execute(&script, &[]) {
            eprint!("Failed to apply migration {migration} : {err}");
            std::process::exit(1);
        }

        if let Err(err) = transaction.execute(
            "INSERT INTO public.migration_history VALUES ($1)",
            &[migration],
        ) {
            eprint!("Failed to apply migration {migration} : {err}");
            std::process::exit(1);
        }

        if let Err(err) = transaction.commit() {
            eprint!("Failed to apply migration {migration} : {err}");
            std::process::exit(1);
        }
    });
}

fn get_migration_history(client: &mut Client) -> Result<Vec<postgres::Row>, postgres::Error> {
    client.query(
        "SELECT mh.migration_id FROM public.migration_history mh",
        &[],
    )
}

fn create_migration_history(client: &mut Client) {
    let result = client.execute("CREATE TABLE IF NOT EXISTS public.migration_history (migration_id varchar(255) primary key)", &[]);

    if let Err(err) = result {
        eprintln!("Error while creating table public.migration_history : {err}");
        std::process::exit(1);
    }
}

fn map_rows(rows: Vec<postgres::Row>) -> Vec<String> {
    rows.iter().filter_map(|x| x.get("migration_id")).collect()
}

fn filter_not_applied_migrations(migration_ids: Vec<String>, files: Vec<String>) -> Vec<String> {
    let mut not_applied_migrations: Vec<String> = files
        .into_iter()
        .filter(|x| !migration_ids.contains(x))
        .collect();

    not_applied_migrations.sort();

    not_applied_migrations
}

fn connect_to_client(database_url: &str) -> Client {
    match Client::connect(database_url, NoTls) {
        Ok(client) => client,
        Err(err) => {
            eprint!(
                "Unable to connect to the database, is your connection string correct? : {err}"
            );
            std::process::exit(1);
        }
    }
}

fn get_database_url() -> String {
    match env::var("DATABASE_URL") {
        Ok(value) => value,
        Err(err) => {
            eprint!("Unable to find \"DATABASE_URL\", is your .env file setup correctly? : {err}");
            std::process::exit(1);
        }
    }
}

fn read_environment_variables() {
    if let Err(err) = dotenv() {
        eprintln!("Error while parsing environment variables : {err}");
        std::process::exit(1);
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

                let valid_migration = file_name.chars().rev().last().is_some_and(char::is_numeric);

                valid_migration.then_some(file_name)
            })
            .collect();

        (!file_names.is_empty()).then_some(file_names)
    } else {
        println!("No \"migrations\" directory was found, creating one...");

        if let Err(err) = create_dir("migrations") {
            eprintln!("Error creating \"migrations\" directory : {err}");
        }
        None
    }
}
