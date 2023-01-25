use crate::kernel::*;
use fnv::FnvHasher;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::fs::{rename, File};
use std::hash::Hasher;
use std::io::{BufReader, BufWriter, Read, Result, Write};
use std::path::Path;
use std::thread::spawn;

pub fn read_json_from_file<P: AsRef<Path>>(path: P) -> StringHashMap {
    let reader = read_file(path).unwrap();
    serde_json::from_reader(reader).unwrap()
}

pub fn read_json_from_file_raw<P: AsRef<Path>>(path: P) -> HashMap<String, Value> {
    let reader = read_file(path).unwrap();
    serde_json::from_reader(reader).unwrap()
}

pub fn read_file<P: AsRef<Path>>(path: P) -> Result<BufReader<File>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path).unwrap();

    Ok(BufReader::new(file))
}

pub fn write_file<P: AsRef<Path>>(path: P) -> Result<BufWriter<File>> {
    // write back to file
    let file = OpenOptions::new().write(true).truncate(true).open(path)?;
    Ok(BufWriter::new(file))
}

pub fn get_random_str(n: u32, scope: &str) -> String {
    let r = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(n as usize)
        .collect::<Vec<_>>();

    let mut sfx: String = String::from_utf8_lossy(&r).into();
    let mut path = String::with_capacity(50);

    // make sure it hasn't been previously assigned
    if scope == "app" {
        path = get_hash_table_addr("./chain/AppHashTableUri.json");
    } else {
        path = get_hash_table_addr("./chain/HashTableUri.json");
    }

    let table = read_json_from_file(path);
    if table.contains_key(&sfx) {
        sfx = get_random_str(32, "user");
    }

    sfx
}

fn get_hash_table_addr<T: AsRef<Path>>(path: T) -> String {
    // get the address of the root hash table from the chain
    let root_addr = read_json_from_file(path);

    let addr = root_addr.get("uri").unwrap();

    addr.clone()
}

// this just mimics the whole IPFS file upload and returns a false CID
pub fn upload_to_ipfs_mimick(str: String) -> ReturnData {
    // get pseudo-CID
    let cid = format!("{}", compute_hash(&str.as_bytes()));

    let path = format!("./ipfs/{}.json", cid);

    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path.clone())
        .unwrap();

    let mut writer = BufWriter::new(file);
    writer.write(&str.as_bytes()).ok();
    writer.flush().ok();

    ReturnData(Parcel::Tuple1(cid, path))
}

// this just simulates the storage on a samaritan node
pub fn get_did_and_keys_mimick(str: &str) -> Parcel {
    let did = String::from("did:sam:root:") + &get_random_str(32, "user");

    // upload to IPFS(files)
    let cid_n_path = match upload_to_ipfs_mimick(str.to_string()) {
        ReturnData(parcel) => match parcel {
            Parcel::Tuple1(cid, path) => (cid, path),
            _ => (String::new(), String::new()),
        },
        _ => (String::new(), String::new()),
    };

    // get the address of the hash table from the chain
    let addr = get_hash_table_addr("./chain/HashtableUri.json");

    // retrieve the hash table from IPFS and update it
    let mut table = read_json_from_file(addr.clone());

    // append new
    table.insert(did.clone(), cid_n_path.1);

    let mut writer = write_file(addr.clone()).unwrap();
    writer
        .write(&serde_json::to_string(&table).unwrap().as_bytes())
        .ok();
    writer.flush().ok();

    let seed = generate_random_words(12);

    // update async
    spawn(move || {
        update_hash_table_uri(&table, addr, "./chain/HashTableUri.json");
    });
    update_keyring(did.clone(), seed.clone());

    Parcel::Tuple1(did, seed)
}

pub fn update_hash_table_uri<P: AsRef<Path> + Clone>(
    table: &StringHashMap,
    addr: String,
    storage: P,
) {
    // get the hash of the new hash table
    let table_str = serde_json::to_string(table).unwrap();

    // compute hash
    let new_addr = format!("./ipfs/{}.json", compute_hash(&table_str.as_bytes()));

    // rename file
    rename(addr, new_addr.clone()).ok();

    // update the chain, set new oot addr
    set_hash_table_uri(new_addr, storage);
}

fn set_hash_table_uri<P: AsRef<Path> + Clone>(uri: String, path: P) {
    let mut table = read_json_from_file(path.clone());

    table.insert("uri".to_owned(), uri);

    // write changes
    let mut writer = write_file(path).unwrap();
    writer
        .write(&serde_json::to_string(&table).unwrap().as_bytes())
        .ok();
    writer.flush().ok();
}

// create new app entry
pub fn create_api_keys_mimick() -> Parcel {
    // generate did for app
    let did = String::from("did:sam:app:") + &get_random_str(48, "app");

    // first get URI of hash table
    let uri = get_hash_table_addr("./chain/AppHashTableUri.json");

    // read the file
    let mut map = read_json_from_file(uri.clone());

    // create a new file that'll serve as table for the application
    let path = format!("./ipfs/{}.json", did.split(':').collect::<Vec<&str>>()[3]);

    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path.clone())
        .unwrap();

    let mut writer = BufWriter::new(file);
    writer.write(b"{}").ok();
    writer.flush().ok();

    // update root hash table
    map.insert(did.clone(), path.clone());

    let mut writer = write_file(uri.clone()).unwrap();
    writer
        .write(&serde_json::to_string(&map).unwrap().as_bytes())
        .ok();
    writer.flush().ok();

    let seed = generate_random_words(12);

    // update async
    spawn(move || {
        update_hash_table_uri(&map, uri, "./chain/AppHashTableUri.json");
    });

    update_keyring(did.clone(), seed.clone());

    Parcel::Tuple1(did, seed)
}

pub fn compute_hash(value: &[u8]) -> u64 {
    let mut hasher = FnvHasher::default();

    hasher.write(value);
    hasher.finish()
}

fn generate_random_words(n: u32) -> String {
    rand_word::new(n as usize)
}

fn update_keyring(did: String, seed: String) {
    let path = "./chain/Keyring.json";
    let mut table = read_json_from_file(path.clone());

    table.insert(format!("{}", compute_hash(&seed.as_bytes())), did);

    // write changes
    let mut writer = write_file(path).unwrap();
    writer
        .write(&serde_json::to_string(&table).unwrap().as_bytes())
        .ok();
    writer.flush().ok();
}

// auhenticate from keyring
pub fn authenticate(keys: String) -> (String, String) {
    // hash and compare
    let path = "./chain/Keyring.json";
    let table = read_json_from_file(path.clone());
    let seed = format!("{}", compute_hash(&keys.as_bytes()));
    let mut exists = "false";
    let mut did = String::new();

    if let Some(id) = table.get(&seed) {
        exists = "true";
        did = id.clone();
    }

    (exists.into(), did)
}

// check whether DID represents app
pub fn is_app(str: &str) -> bool {
    if str.split(":").collect::<Vec<&str>>()[2] == "app" {
        true
    } else {
        false
    }
}

// pub fn get_did_suffix(did: String) -> String {
//     did.splitn(3, ":").collect::<Vec<&str>>()[2].into()
// }

pub fn get_sam_hashtable(did: &String) -> StringHashMap {
    // get samaritan did document uri
    let did_file_uri = get_sam_ht_uri(did);
    let mut reader = read_file(did_file_uri).unwrap();
    let mut temp_str = String::new();
    let _ = reader.read_to_string(&mut temp_str);

    let table: Value = serde_json::from_str(&temp_str).unwrap();
    let hash_table = table["hash_table"].clone();

    convert_to_hashmap(hash_table)
}

pub fn get_app_htable_uri(did: &String) -> String {
    // get the address of the hash table from the chain
    let apps_ht_uri = get_hash_table_addr("./chain/AppHashtableUri.json");
    let apps_root_table: StringHashMap = read_json_from_file(apps_ht_uri);

    // get file table uri
    apps_root_table.get(did).unwrap().into()
}

pub fn get_sam_ht_uri(did: &String) -> String {
    let addr = get_hash_table_addr("./chain/HashtableUri.json");
    let did_root_table: StringHashMap = read_json_from_file(addr);

    // get file table uri
    did_root_table.get(did).unwrap().into()
}

pub fn save_sam_htable(table: &StringHashMap, did: &String) {
    // save the modified hashtable in the did document
    let path = get_sam_ht_uri(did);
    let mut reader = read_file(path.clone()).unwrap();
    let mut temp_str = String::new();

    let _ = reader.read_to_string(&mut temp_str);

    let mut did_doc: Value = serde_json::from_str(&temp_str).unwrap();
    did_doc["hash_table"] = json!(table);

    // read file
    let mut writer = write_file(path).unwrap();
    writer
        .write(&serde_json::to_string(&did_doc).unwrap().as_bytes())
        .ok();
    writer.flush().ok();
}

fn convert_to_hashmap(json: Value) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Value::Object(obj) = json {
        for (key, value) in obj {
            if let Value::String(string_value) = value {
                map.insert(key, string_value);
            }
        }
    }
    map
}
