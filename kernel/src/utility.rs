use std::{collections::HashMap};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write, Result};
use std::path::Path;
use std::fs::OpenOptions;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

use crate::kernel::*;


pub fn read_json_from_file<P: AsRef<Path>>(path: P) -> HashMap<String, String> {
    let reader = read_file(path).unwrap();
    // Read the JSON contents of the file as an instance of `User`.
    serde_json::from_reader(reader).unwrap()

    // format is 
    // did => [cid, hash]
}

pub fn update_hash_table(did: String, cid: String) {
    let path = "./record/hash_table.json";
    let mut table = read_json_from_file(path);

    // append new
    table.entry(did).or_insert(cid);

    let mut writer = write_file(path).unwrap();
    writer.write(&serde_json::to_string(&table).unwrap().as_bytes()).ok();
    writer.flush().ok();
}

pub fn read_file<P: AsRef<Path>>(path: P) -> Result<BufReader<File>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path).unwrap();

    Ok(BufReader::new(file))
}

pub fn write_file<P: AsRef<Path>>(path: P) -> Result<BufWriter<File>> {
    // write back to file
    let file = OpenOptions::new()
        .write(true)
        .open(path)?;

    Ok(BufWriter::new(file))
}

pub fn get_did_suffix(n: u32) -> String {
    let r = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(n as usize)
        .collect::<Vec<_>>();

    let mut sfx: String = String::from_utf8_lossy(&r).into();

    // make sure it hasn't been previously assigned
    let path = "./record/hash_table.json";
    let table = read_json_from_file(path);

    if table.contains_key(&sfx) {
        sfx = get_did_suffix(32);
    }

    sfx
}

// this just mimics the whole IPFS file upload and returns a false CID
pub fn upload_to_ipfs_mimick(str: String)  -> ReturnData {
    // get pseudo-CID
    let cid = get_did_suffix(48);
    let path = "./files/".to_owned() + &cid + ".txt";

    let file = OpenOptions::new() 
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();

    let mut writer = BufWriter::new(file);
    writer.write(&str.as_bytes()).ok();
    writer.flush().ok();

    ReturnData(Parcel::String(cid))
}