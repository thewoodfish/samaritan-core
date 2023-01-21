// use std::collections::HashMap;
use crate::{
    kernel::{self, Parcel, ReturnData, StringHashMap},
    utility, chain::{self, ChainClient},
};
use actix::prelude::*;
use serde_json::Value;
use std::{fs::{File, OpenOptions}, fmt::Debug};
use std::io::{BufReader, BufWriter, Result, Write};
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
    pub did_uri_cache: Vec<Cache<StringHashMap>>,
    pub did_url_cache_height: u64,
}

type Note = kernel::Note;

impl Network {
    pub fn new(ipfs_url: String) -> Network {
        Network {
            ipfs_url,
            app_uri_cache: Vec::<Cache<StringHashMap>>::new(),
            url_cache_height: 0,
            did_uri_cache: Vec::<Cache<StringHashMap>>::new(),
            did_url_cache_height: 0,
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
        let table = utility::read_json_from_file("./chain/AppHashTableUri.json");
        let mut cid = String::new();

        if let Some(uri) = table.get("uri") {
            cid = uri.clone();
        }
        self.set_cache_url(did, cid.clone());

        // cache some key-value pairs in the store
        self.cache_content(cid);
    }

    fn cache_content<P: AsRef<Path> + Debug>(&mut self, uri: P) {
        // read file contents
        println!("----------{:?}", uri);
        
        let table = utility::read_json_from_file(uri);

        // TODO: Add randomness to cache
        let cache_height = self.did_url_cache_height;

        // cache only the DID uri's
        let _ = table
            .iter()
            .filter(|(k, v)| (*k).contains("did:sam") && cache_height < 50)
            .map(|e| {
                // set up cache
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs();

                let mut block = Cache {
                    created: now,
                    data: StringHashMap::new(),
                };

                block.data = StringHashMap::new();
                block.data.insert(e.0.clone(), e.1.clone());

                self.did_uri_cache.push(block);
                self.did_url_cache_height += 1;
            });
    }

    fn set_cache_url(&mut self, did: String, uri: String) -> bool {
        // avoid duplicates
        let exists = self.app_uri_cache.iter().any(|c| c.data.contains_key(&did));

        if self.url_cache_height < 50 && !exists {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();

            let mut block = Cache {
                created: now,
                data: StringHashMap::new(),
            };

            block.data.insert(did, uri);

            self.app_uri_cache.push(block);
            self.url_cache_height += 1;

            true
        } else {
            false
        }
    }

    fn insert_record(&self, param: String) {
        let params: Vec<&str> = param.split("#").collect();
        if params[0] == "" {
            println!("{:?}", params[2]);
            self.insert_new_node(params[1].into(), serde_json::from_str(params[2]).unwrap(), params[3].into());
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
        &self,
        did: String,
        key: String,
        val: String,
        app_did: String,
    ) -> (String, String) {
        // check cache first
        let mut uri: Option<String> = None;
        for c in &self.did_uri_cache {
            if c.data.contains_key(&did) {
                uri = Some(c.data.get(&did).unwrap().clone());
                break;
            }
        }

        // if not found in cache
        if uri == None {
            // get manually
            // get hashtable address from cache
            for i in &self.app_uri_cache {
                if let Some(ht_uri) = (*i).data.get(&key) {
                    uri = Some(ht_uri.clone());
                    break;
                }
            }
        }

        let uri = uri.unwrap();

        // retrieve contents from the file location
        let mut table = utility::read_json_from_file(uri.clone());

        // the file uri name is hash(`did of app` + `did of user`)
        let hash_key = app_did.clone() + did.as_str();
        let file_url = format!(
            "./ipfs/files/{}.json",
            utility::compute_hash(&hash_key.as_bytes()[..])
        );
        let mut file: File;

        // update the hash table of the did too, to link to the common file
        let mut did_table: StringHashMap = utility::get_did_hashtable(&did);
        did_table.entry(app_did).or_insert(file_url.clone());

        // check for the did entry
        if !table.contains_key(&did) {
            // create entry
            table.insert(did.clone(), file_url.clone());

            // save entry
            let mut writer = utility::write_file(uri.clone()).unwrap();
            writer
                .write(&serde_json::to_string(&table).unwrap().as_bytes())
                .ok();
            writer.flush().ok();

            // create the new file
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(file_url.clone())
                .unwrap();

            let mut writer = BufWriter::new(file);
            writer.write(b"{}").ok();
            writer.flush().ok();
        }

        // read file and write to it
        let mut table = utility::read_json_from_file(file_url.clone());
        table
            .entry(key)
            .or_insert(serde_json::from_str(&val).unwrap());

        // save file
        let mut writer = utility::write_file(file_url.clone()).unwrap();
        writer
            .write(&serde_json::to_string(&table).unwrap().as_bytes())
            .ok();
        writer.flush().ok();

        (hash_key, file_url)
    }

    fn insert_new_node(&self, key: String, val: Value, app_did: String) {
        let mut uri = String::with_capacity(50);

        println!("{:?}", self.app_uri_cache);
        println!("{:?}", app_did);

        // get hashtable address from cache
        for i in &self.app_uri_cache {
            if let Some(ht_uri) = (*i).data.get(&app_did) {
                uri = ht_uri.clone();
                break;
            }
        }

        println!("{:?}", uri);

        // read from address
        let tab = utility::read_json_from_file(uri.clone());

        // get the storage file
        let app_storage_url = tab.get(&app_did).unwrap();
        let mut table = utility::read_json_from_file_raw(app_storage_url.clone());
        table.insert(key, val);

        // save back to file
        let mut writer = utility::write_file(app_storage_url).unwrap();
        writer
            .write(&serde_json::to_string(&table).unwrap().as_bytes())
            .ok();
        writer.flush().ok();
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
            }
            _ => {}
        }

        Ok(ReturnData(Parcel::Empty))
    }
}
