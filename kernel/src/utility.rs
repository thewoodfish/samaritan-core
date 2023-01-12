use std::{error::Error, collections::HashMap};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;


pub fn read_json_from_file<P: AsRef<Path>>(path: P) -> HashMap<String, String> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    serde_json::from_reader(reader).unwrap()

    // format is 
    // did => [cid, hash]
}

pub fn update_hash_table(did: String, cid: String) {
    let mut table = read_json_from_file("../record/hash_table.json");

    // append new
    table.entry(did).or_insert(cid);
}