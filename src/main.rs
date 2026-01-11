mod hop_lang;
mod http;
mod network;
mod ui;

use std::error::Error;
use std::fs;

use crate::{
    hop_lang::{clean_script, fetch_connection_header, fetch_requests},
    network::connect,
    network::execute_batch_requests,
};

fn main() -> Result<(), Box<dyn Error>> {
    // TODO: Organise the file handling
    // to a separate module
    let query_raw = fs::read_to_string("example.hop")?;

    let cleaned_queries = clean_script(&query_raw)?;
    let mut conn = fetch_connection_header(&cleaned_queries)?;
    connect(&mut conn)?;

    let all_requests = fetch_requests(&cleaned_queries)?;

    execute_batch_requests(all_requests, &mut conn)?;
    Ok(())
}
