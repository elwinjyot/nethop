mod hop_lang;
mod http;
mod network;
mod ui;
mod file_handler;

use std::{env, error::Error, io::{self, Write}};

use crate::{
    hop_lang::{clean_script, fetch_connection_header, fetch_requests},
    network::connect,
    network::execute_batch_requests,
    file_handler::{read_queries_from_file, read_queries_from_workspace}
};

fn main() -> Result<(), Box<dyn Error>> {
    println!("🐇 NetHop v0.1-Beta");

    let args: Vec<String> = env::args().collect();
    let query_raw = match args.get(1) {
        Some(file_path) => read_queries_from_file(file_path)?,
        None => read_queries_from_workspace()?
    };

    let cleaned_queries = clean_script(&query_raw)?;
    let mut start_query = String::new();
    print!("Queries prepared, start execution? [Y/n]: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut start_query)?;

    if start_query.trim().to_lowercase() == "y" {
        let mut conn = fetch_connection_header(&cleaned_queries)?;
        connect(&mut conn)?;
        let all_requests = fetch_requests(&cleaned_queries)?;
        execute_batch_requests(all_requests, &mut conn)?;
    } else {
        println!("❌ Cancelled");
    }
    
    Ok(())
}
