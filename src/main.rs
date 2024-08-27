use database::{
    apply_migrations, connect_to_client, create_migration_history, get_migration_history,
    read_migration_files,
};
use environment::{get_database_url, read_environment_variables};

mod database;
mod environment;

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
