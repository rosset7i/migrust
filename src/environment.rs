use dotenv::dotenv;
use std::env;

pub fn get_database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|err| {
        panic!("Unable to find \"DATABASE_URL\", is your .env file setup correctly? : {err}")
    })
}

pub fn read_environment_variables() {
    dotenv().unwrap_or_else(|err| panic!("Error while parsing environment variables : {err}"));
}
