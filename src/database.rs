use postgres::{Client, Error, NoTls, Row};
use std::fs::{create_dir_all, read_dir, read_to_string};

pub fn connect_to_client(database_url: &str) -> Client {
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

pub fn get_migration_history(client: &mut Client) -> Result<Vec<Row>, Error> {
    client.query(
        "SELECT mh.migration_id FROM public.migration_history mh",
        &[],
    )
}

pub fn create_migration_history(client: &mut Client) {
    let result = client.execute("CREATE TABLE IF NOT EXISTS public.migration_history (migration_id varchar(255) primary key)", &[]);

    if let Err(err) = result {
        eprintln!("Error while creating table public.migration_history : {err}");
        std::process::exit(1);
    }
}

pub fn read_migration_files() -> Option<Vec<String>> {
    if let Ok(value) = read_dir("./migrations") {
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

        if let Err(err) = create_dir_all("migrations") {
            eprintln!("Error creating \"migrations\" directory : {err}");
        }
        None
    }
}

pub fn apply_migrations(client: &mut Client, not_applied_migrations: &[String]) {
    not_applied_migrations.iter().for_each(|migration| {
        let script = match read_to_string(format!("migrations/{migration}")) {
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
