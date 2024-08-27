use dotenv::dotenv;
use std::env;

pub fn get_database_url() -> String {
    match env::var("DATABASE_URL") {
        Ok(value) => value,
        Err(err) => {
            eprint!("Unable to find \"DATABASE_URL\", is your .env file setup correctly? : {err}");
            std::process::exit(1);
        }
    }
}

pub fn read_environment_variables() {
    if let Err(err) = dotenv() {
        eprintln!("Error while parsing environment variables : {err}");
        std::process::exit(1);
    }
}
