use mongodb::{options::ClientOptions, Client, Database};
use crate::config::get_env;

pub async fn init() -> Database {
    let client_options = ClientOptions::parse(&get_env("MONGO_URI")).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    client.database(&get_env("DATABASE_NAME"))
}