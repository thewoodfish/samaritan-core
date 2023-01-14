use std::fmt::format;
use std::{collections::HashMap};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write, Result};
use std::path::Path;
use std::fs::OpenOptions;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

use crate::kernel::*;


pub fn read_json_from_file<P: AsRef<Path>>(path: P) -> HashMap<String, String> {
    let reader = read_file(path).unwrap();
    serde_json::from_reader(reader).unwrap()

    // format is 
    // did => [cid, hash]
}

pub fn read_json_from_chain<P: AsRef<Path>>(path: P) -> HashMap<u64, (bool, String)> {
    let reader = read_file(path).unwrap();
    serde_json::from_reader(reader).unwrap()

    // format is 
    // did => [key, (bool, cid)]
}

pub fn update_hash_table(did: String, cid: String) {
    let path = "./ipfs/hash_table.json";
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

pub fn get_random_str(n: u32) -> String {
    let r = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(n as usize)
        .collect::<Vec<_>>();

    let mut sfx: String = String::from_utf8_lossy(&r).into();

    // make sure it hasn't been previously assigned
    let path = "./ipfs/hash_table.json";
    let table = read_json_from_file(path);

    if table.contains_key(&sfx) {
        sfx = get_random_str(32);
    }

    sfx
}

// this just mimics the whole IPFS file upload and returns a false CID
pub fn upload_to_ipfs_mimick(str: String)  -> ReturnData {
    // get pseudo-CID
    let cid = get_random_str(48);
    let path = "./files/".to_owned() + format!("{}", compute_hash(&cid.as_bytes())).as_str() + ".txt";

    let file = OpenOptions::new() 
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();

    let mut writer = BufWriter::new(file);
    writer.write(&str.as_bytes()).ok();
    writer.flush().ok();

    // update hash table
    update_hash_table_uri(format!("{}", compute_hash(&str.as_bytes())));

    ReturnData(Parcel::String(cid))
}

// this just simulates the storage on a samaritan node
pub fn get_did_and_keys_mimick(str: &str) -> Parcel {
    let did = String::from("did:sam:root:") + &get_random_str(32);

    // upload to IPFS(files)
    let cid = match upload_to_ipfs_mimick(str.to_string()) {
        ReturnData(parcel) => {
            match parcel {
                Parcel::String(str) => str.to_owned(),
                _ => String::new()
            }
        },
        _ => String::new()
    };

    // record transaction
    record_data_entry(did.clone(), "samaritan_root_document".to_owned(), cid);

    Parcel::Tuple1(did, generate_random_words(12))
}

// simulates chain function -> update_hashtable_uri
pub fn update_hash_table_uri(uri: String) {
    let path = "./chain/HashtableUri.json";
    let mut table = read_json_from_file(path);

    // append new
    table.insert("uri".to_string(), uri);

    let mut writer = write_file(path).unwrap();
    writer.write(&serde_json::to_string(&table).unwrap().as_bytes()).ok();
    writer.flush().ok();
}

// simulates chain function -> record_data_entry
pub fn record_data_entry(key1: String, key2: String, cid: String) {
    let key = key1 + &key2;
    let path = "./chain/FragRecord.json";
    let mut table: HashMap<u64, (bool, String)> = read_json_from_chain(path);

    // append new
    table.insert(compute_hash(key.as_bytes()), (true, cid));

    let mut writer = write_file(path).unwrap();
    writer.write(&serde_json::to_string(&table).unwrap().as_bytes()).ok();
    writer.flush().ok();
}

// simulates chain function -> delete_data_entry
pub fn delete_data_entry(key1: String, key2: String, cid: String) {
    let key = key1 + &key2;
    let path = "./chain/FragRecord.json";
    let mut table: HashMap<u64, (bool, String)> = read_json_from_chain(path);

    // append new
    table.insert(compute_hash(&key.as_bytes()[..]), (false, cid));

    let mut writer = write_file(path).unwrap();
    writer.write(&serde_json::to_string(&table).unwrap().as_bytes()).ok();
    writer.flush().ok();
}

fn compute_hash(value: &[u8]) -> u64 {
    let s = RandomState::new();
    let mut hasher = s.build_hasher();

    hasher.write(value);
    hasher.finish()
}

fn generate_random_words(n: u32) -> String {
    rand_word::new(n as usize)
}