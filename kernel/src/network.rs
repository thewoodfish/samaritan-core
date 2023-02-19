// use std::collections::HashMap;
use crate::{
    chain::{self, ChainClient},
    kernel::{self, Parcel, ReturnData, StringHashMap},
    utility::{self, compute_hash},
};
use actix::prelude::*;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs,
    io::{BufReader, BufWriter, Result, Write, Read},
};
use std::{
    fmt::Debug,
    fs::{File, OpenOptions},
};
use std::{path::Path, time::SystemTime};
// use ipfs_api_backend_actix::ApiError;

// use ipfs_api::{IpfsApi, IpfsClient};
// use serde_json::Value;
// use std::io::Cursor;
use futures_lite::future;

#[derive(Debug)]
pub struct Cache<T> {
    created: u64,
    data: T,
}

#[derive(Debug)]
pub struct Network {
    pub ipfs_url: String,
    pub app_uri_cache: Vec<Cache<StringHashMap>>,
    pub url_cache_height: u64,
}

type Note = kernel::Note;

impl Network {
    pub fn new(ipfs_url: String) -> Network {
        Network {
            ipfs_url,
            app_uri_cache: Vec::<Cache<StringHashMap>>::new(),
            url_cache_height: 0,
        }
    }

    // HANGING!!!
    // pub async fn upload_to_ipfs(str: String) -> Result<ReturnData, ApiError> {
    //     let client = IpfsClient::default();
    //     let data = Cursor::new(str);

    //     match client.add(data).await {
    //         Ok(res) => {
    //             println!("{:?}", res);
    //             Ok(ReturnData(Parcel::String(res.hash)))
    //         },
    //         Err(_) => {
    //             Err(ApiError {
    //                 message: "could not upload to IPFS".to_string(),
    //                 code: 102
    //             })
    //         }
    //     }
    // }

    // this just mimics the whole IPFS file upload and returns a false CID
    // pub async fn upload_to_ipfs(str: String) -> Result<ReturnData, ApiError> {
    //     Ok(utility::upload_to_ipfs_mimick(str))
    // }

    pub async fn sync_hash_table(&mut self) {
        // read file
        // let buf = utility::read_file("./ipfs/hash_table.json").unwrap();
        // let inter: Value = serde_json::from_reader(buf).unwrap();
        // let hash_table = inter.as_str().unwrap().to_owned();

        // let client = IpfsClient::default();
        // let data = Cursor::new(hash_table);

        // HANGS!!!
        // match client.add(data).await {
        //     Ok(res) => {
        //         // update HashTable uri
        //         self.ipfs_url = res.hash;
        //     },
        //     Err(e) => eprintln!("error adding file: {}", e)
        // }
    }

    fn create_app_locality(&mut self, did: String) {
        let chain_storage = utility::read_json_from_file("./chain/AppHashTableUri.json");
        let apps_hashtable_uri = chain_storage.get("uri").unwrap();

        // get the app specific hashtable url
        let apps_hashtable = utility::read_json_from_file(apps_hashtable_uri);
        let app_specific_ht_url = apps_hashtable.get(&did).unwrap();

        self.set_cache_url(did, app_specific_ht_url.clone());

        // cache some key-value pairs in the store
        // self.cache_content(app_specific_ht_url);
    }

    // fn cache_content<P: AsRef<Path> + Debug>(&mut self, app_ht_uri: P) {
    //     // read file contents
    //     let table = utility::read_json_from_file(app_ht_uri);

    //     // TODO: Add randomness to cache
    //     let cache_height = self.did_url_cache_height;

    //     // cache only the DID uri's
    //     let _ = table
    //         .iter()
    //         .filter(|(k, v)| (*k).contains("did:sam") && cache_height < 50)
    //         .map(|e| {
    //             // set up cache
    //             let now = SystemTime::now()
    //                 .duration_since(SystemTime::UNIX_EPOCH)
    //                 .expect("Time went backwards")
    //                 .as_secs();

    //             let mut block = Cache {
    //                 created: now,
    //                 data: StringHashMap::new(),
    //             };

    //             block.data = StringHashMap::new();
    //             block.data.insert(e.0.clone(), e.1.clone());

    //             self.did_uri_cache.push(block);
    //             self.did_url_cache_height += 1;
    //         });
    // }

    fn set_cache_url(&mut self, app_did: String, app_hashtable_uri: String) -> bool {
        // avoid duplicates
        let exists = self
            .app_uri_cache
            .iter()
            .any(|c| c.data.contains_key(&app_did));

        if self.url_cache_height < 50 && !exists {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();

            let mut block = Cache {
                created: now,
                data: StringHashMap::new(),
            };

            block.data.insert(app_did, app_hashtable_uri);

            self.app_uri_cache.push(block);
            self.url_cache_height += 1;

            true
        } else {
            false
        }
    }

    fn insert_record(&mut self, param: String) {
        let params: Vec<&str> = param.split("#").collect();
        if params[0] == "" {
            self.insert_new_node(
                params[1].into(),
                serde_json::from_str(params[2]).unwrap(),
                params[3].into(),
            );
        } else {
            let (hash_key, file_addr) = self.insert_did_node(
                params[0].into(),
                params[1].into(),
                params[2].into(),
                params[3].into(),
            );

            // notify the chain
            future::block_on(async {
                chain::ChainClient::record_data_entry(hash_key, file_addr).await;
            });
        }
    }

    fn insert_did_node(
        &mut self,
        sam_did: String,
        key: String,
        val: String,
        app_did: String,
    ) -> (String, String) {
        // check cache first
        let mut app_ht_uri = String::with_capacity(64);
        let mut app_hashtable: HashMap<String, Value>;
        for c in &self.app_uri_cache {
            if c.data.contains_key(&app_did) {
                app_ht_uri = c.data.get(&app_did).unwrap().clone();
                break;
            }
        }

        // if not found in cache
        if app_ht_uri.is_empty() {
            // get manually
            app_ht_uri = utility::get_app_htable_uri(&app_did);

            // insert into cache
            self.set_cache_url(app_did.clone(), app_ht_uri.clone());
        }

        // retrieve contents from the file location
        app_hashtable = utility::read_json_from_file_raw(app_ht_uri.clone());

        // the file uri name is hash(`did of app` + `did of samaritan`)
        let h_key = app_did.clone() + sam_did.as_str();
        let hash_key = format!("{}", compute_hash(&h_key.as_bytes()[..]));
        let file_url = format!("./ipfs/files/{}.json", hash_key);

        // update the hash table of the did, to link to the common file
        let mut sam_htable: StringHashMap = utility::get_sam_hashtable(&sam_did);
        if !sam_htable.contains_key(&app_did) {
            sam_htable.insert(app_did.clone(), file_url.clone());

            // save
            utility::save_sam_htable(&sam_htable, &sam_did);
        }

        // check for the did entry in the apps hash table
        if !app_hashtable.contains_key(&sam_did) {
            // change the URL string to a JSON value
            let json_file_url = format!(
                r#""./ipfs/files/{}.json""#,
                utility::compute_hash(&hash_key.as_bytes()[..])
            );
            let json_str = serde_json::from_str(json_file_url.as_str()).unwrap();
            // create entry
            app_hashtable.insert(sam_did.clone(), json_str);

            // save entry
            let mut writer = utility::write_file(app_ht_uri.clone()).unwrap();
            writer
                .write(&serde_json::to_string(&app_hashtable).unwrap().as_bytes())
                .ok();
            writer.flush().ok();

            // create new empty file
            fs::write(file_url.clone(), b"{}").expect("Failed to create file");
        }

        // read file and write to it
        let mut kv_store = utility::read_json_from_file_raw(file_url.clone());
        kv_store.insert(key, serde_json::from_str(&val).unwrap());

        // save file
        let mut writer = utility::write_file(file_url.clone()).unwrap();
        writer
            .write(&serde_json::to_string(&kv_store).unwrap().as_bytes())
            .ok();
        writer.flush().ok();

        (hash_key, file_url)
    }

    fn insert_new_node(&mut self, key: String, val: Value, app_did: String) {
        let mut uri = String::with_capacity(50);

        // get hashtable address from cache
        for i in &self.app_uri_cache {
            if let Some(ht_uri) = (*i).data.get(&app_did) {
                uri = ht_uri.clone();
                break;
            }
        }

        // if not found in cache
        if uri.is_empty() {
            // get manually
            uri = utility::get_app_htable_uri(&app_did);

            // insert into cache
            self.set_cache_url(app_did.clone(), uri.clone());
        }

        // read from address
        let mut app_htable = utility::read_json_from_file_raw(uri.clone());
        app_htable.insert(key, val);

        // save back to file
        let mut writer = utility::write_file(uri).unwrap();
        writer
            .write(&serde_json::to_string(&app_htable).unwrap().as_bytes())
            .ok();
        writer.flush().ok();
    }

    async fn get_record(&mut self, param: String) -> Option<Value> {
        let params: Vec<&str> = param.split("#").collect();
        if params[0] == "" {
            self.get_app_record(params[1].into(), params[2].into())
        } else {
            self.get_did_record(params[0].into(), params[1].into(), params[2].into())
        }
    }

    fn get_did_record(&mut self, sam_did: String, key: String, app_did: String) -> Option<Value> {
        // frist open the root file of the app
        let mut app_ht_uri = String::with_capacity(64);
        let app_hashtable: HashMap<String, Value>;

        // prepare hashkey
        let h_key = app_did.clone() + sam_did.as_str();
        let hash_key = format!("{}", compute_hash(&h_key.as_bytes()[..]));

        for c in &self.app_uri_cache {
            if c.data.contains_key(&app_did) {
                app_ht_uri = c.data.get(&app_did).unwrap().clone();
                break;
            }
        }

        // if not found in cache
        if app_ht_uri.is_empty() {
            // get manually
            app_ht_uri = utility::get_app_htable_uri(&app_did);

            // insert into cache
            self.set_cache_url(app_did.clone(), app_ht_uri.clone());
        }

        // one more level of indirection
        // read only the address of the file
        app_hashtable = utility::read_json_from_file_raw(app_ht_uri.clone());

        // check for existence
        if app_hashtable.contains_key(&sam_did) {
            // check chain if access is allowed
            let chain_data = utility::read_json_from_file_raw("./chain/DataRecord.json");
            if chain_data.contains_key(&hash_key) {
                let record = chain_data.get(&hash_key).unwrap().clone();
                let can_access = record["can_access"].as_bool().unwrap();
                if can_access {
                    // get storage file of did
                    let storage_addr = record["uri"].as_str().unwrap();

                    // read file
                    let kv_store = utility::read_json_from_file_raw(storage_addr.clone());
                    if kv_store.contains_key(&key) {
                        let val = kv_store.get(&key).unwrap().clone();
                        Some(val)
                    } else {
                        None
                    }
                } else {
                    Some(json!({
                        "status": "revoked"
                    }))
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_app_record(&mut self, key: String, app_did: String) -> Option<Value> {
        // frist open the root file of the app
        let mut app_ht_uri = String::with_capacity(64);
        let app_hashtable: HashMap<String, Value>;

        for c in &self.app_uri_cache {
            if c.data.contains_key(&app_did) {
                app_ht_uri = c.data.get(&app_did).unwrap().clone();
                break;
            }
        }

        // if not found in cache
        if app_ht_uri.is_empty() {
            // get manually
            app_ht_uri = utility::get_app_htable_uri(&app_did);

            // insert into cache
            self.set_cache_url(app_did.clone(), app_ht_uri.clone());
        }

        // read table and get the specific value asked for
        app_hashtable = utility::read_json_from_file_raw(app_ht_uri.clone());

        // check for existence
        if app_hashtable.contains_key(&key) {
            // get the value
            let val = app_hashtable.get(&key).unwrap().clone();
            Some(val)
        } else {
            None
        }
    }

    async fn delete_record(&mut self, param: String) -> Option<Value> {
        let params: Vec<&str> = param.split("#").collect();
        if params[0] == "" {
            self.del_app_record(params[1].into(), params[2].into())
        } else {
            self.del_did_record(params[0].into(), params[1].into(), params[2].into())
        }
    }

    fn del_did_record(&mut self, sam_did: String, key: String, app_did: String) -> Option<Value> {
        // frist open the root file of the app
        let mut app_ht_uri = String::with_capacity(64);
        let app_hashtable: HashMap<String, Value>;

        // prepare hashkey
        let h_key = app_did.clone() + sam_did.as_str();
        let hash_key = format!("{}", compute_hash(&h_key.as_bytes()[..]));

        for c in &self.app_uri_cache {
            if c.data.contains_key(&app_did) {
                app_ht_uri = c.data.get(&app_did).unwrap().clone();
                break;
            }
        }

        // if not found in cache
        if app_ht_uri.is_empty() {
            // get manually
            app_ht_uri = utility::get_app_htable_uri(&app_did);

            // insert into cache
            self.set_cache_url(app_did.clone(), app_ht_uri.clone());
        }

        // one more level of indirection
        // read only the address of the file
        app_hashtable = utility::read_json_from_file_raw(app_ht_uri.clone());

        // check for existence
        if app_hashtable.contains_key(&sam_did) {
            // check chain if access is allowed
            let chain_data = utility::read_json_from_file_raw("./chain/DataRecord.json");
            if chain_data.contains_key(&hash_key) {
                let record = chain_data.get(&hash_key).unwrap().clone();
                let can_access = record["can_access"].as_bool().unwrap();
                if can_access {
                    // get storage file of did
                    let storage_addr = record["uri"].as_str().unwrap();

                    // read file
                    let mut kv_store = utility::read_json_from_file_raw(storage_addr.clone());
                    if kv_store.contains_key(&key) {
                        let val = kv_store.remove(&key).unwrap();

                        // save file
                        let mut writer = utility::write_file(storage_addr).unwrap();
                        writer
                            .write(&serde_json::to_string(&kv_store).unwrap().as_bytes())
                            .ok();
                        writer.flush().ok();

                        Some(val)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn del_app_record(&mut self, key: String, app_did: String) -> Option<Value> {
        // frist open the root file of the app
        let mut app_ht_uri = String::with_capacity(64);
        let mut app_hashtable: HashMap<String, Value>;

        for c in &self.app_uri_cache {
            if c.data.contains_key(&app_did) {
                app_ht_uri = c.data.get(&app_did).unwrap().clone();
                break;
            }
        }

        // if not found in cache
        if app_ht_uri.is_empty() {
            // get manually
            app_ht_uri = utility::get_app_htable_uri(&app_did);

            // insert into cache
            self.set_cache_url(app_did.clone(), app_ht_uri.clone());
        }

        // read table
        app_hashtable = utility::read_json_from_file_raw(app_ht_uri.clone());

        // check for existence
        if app_hashtable.contains_key(&key) {
            // get the value when its deleted
            let val = app_hashtable.remove(&key).unwrap();

            // save back to file
            let mut writer = utility::write_file(app_ht_uri).unwrap();
            writer
                .write(&serde_json::to_string(&app_hashtable).unwrap().as_bytes())
                .ok();
            writer.flush().ok();

            Some(val)
        } else {
            None
        }
    }

    async fn read_did_doc(did: String) -> Option<String> {
        // get did document
        let did_file_uri = utility::get_sam_ht_uri(&did);
        let mut reader = utility::read_file(did_file_uri).unwrap();
        let mut temp_str = String::new();
        let _ = reader.read_to_string(&mut temp_str);

        Some(serde_json::to_string(&temp_str).unwrap())
    }
}

impl Actor for Network {
    type Context = Context<Self>;
}

impl Handler<Note> for Network {
    type Result = Result<ReturnData>;

    /// handle incoming "Note" and dispatch to various appropriate methods
    fn handle(&mut self, msg: Note, _: &mut Context<Self>) -> Self::Result {
        match &msg.0 {
            100 => {
                // read the corresponding DID document
                let data = future::block_on(async {
                    match msg.1 {
                        Parcel::String(param) => match Network::read_did_doc(param).await {
                            Some(val) => Ok::<ReturnData, std::io::Error>(ReturnData(
                                Parcel::String(val),
                            )),
                            None => Ok::<ReturnData, std::io::Error>(ReturnData(Parcel::Empty)),
                        },
                        _ => Ok::<ReturnData, std::io::Error>(ReturnData(Parcel::Empty)),
                    }
                });

                return data;
            }

            101 => {
                future::block_on(async {
                    // update IPFS version
                    self.sync_hash_table().await;
                });
            }
            102 => {
                // update cache
                match msg.1 {
                    Parcel::String(did) => {
                        self.create_app_locality(did);
                    }
                    _ => {}
                }
            }
            103 => {
                // insert into database
                future::block_on(async {
                    match msg.1 {
                        Parcel::String(params) => {
                            self.insert_record(params);
                        }
                        _ => {}
                    }
                });

                return Ok(ReturnData(Parcel::String("blessed assurance!".to_owned())));
            }
            104 => {
                // retreive from database
                let data = future::block_on(async {
                    match msg.1 {
                        Parcel::String(params) => match self.get_record(params).await {
                            Some(val) => Ok::<ReturnData, std::io::Error>(ReturnData(
                                Parcel::String(serde_json::to_string(&val).unwrap()),
                            )),
                            None => Ok::<ReturnData, std::io::Error>(ReturnData(Parcel::Empty)),
                        },
                        _ => Ok::<ReturnData, std::io::Error>(ReturnData(Parcel::Empty)),
                    }
                });

                return data;
            }
            105 => {
                // retreive from database
                let data = future::block_on(async {
                    match msg.1 {
                        Parcel::String(params) => match self.delete_record(params).await {
                            Some(val) => Ok::<ReturnData, std::io::Error>(ReturnData(
                                Parcel::String(serde_json::to_string(&val).unwrap()),
                            )),
                            None => Ok::<ReturnData, std::io::Error>(ReturnData(Parcel::Empty)),
                        },
                        _ => Ok::<ReturnData, std::io::Error>(ReturnData(Parcel::Empty)),
                    }
                });

                return data;
            }
            _ => {}
        }
        Ok(ReturnData(Parcel::Empty))
    }
}
