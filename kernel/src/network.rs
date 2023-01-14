use crate::{kernel::{self, ReturnData, Parcel}, utility};
use actix::prelude::*;
use ipfs_api_backend_actix::ApiError;

// use ipfs_api::{IpfsApi, IpfsClient};
// use serde_json::Value;
// use std::io::Cursor;
use futures_lite::future;

#[derive(Debug)]
pub struct Network {
    pub ipfs_url: String,
}

type Note = kernel::Note;

impl Network {
    pub fn new(ipfs_url: String) -> Network {
        Network { ipfs_url }
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
    pub async fn upload_to_ipfs(str: String) -> Result<ReturnData, ApiError> {
        Ok(utility::upload_to_ipfs_mimick(str))
    }

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
}

impl Actor for Network {
    type Context = Context<Self>;
}

impl Handler<Note> for Network {
    type Result = Result<ReturnData, std::io::Error>;

    /// handle incoming "Note" and dispatch to various appropriate methods
    fn handle(&mut self, msg: Note, _: &mut Context<Self>) -> Self::Result {
        match &msg.0 {
            101 => {
                future::block_on(async {
                    // update IPFS version
                    self.sync_hash_table().await;
                });
            }
            _ => {}
        }
        
        Ok(ReturnData(Parcel::Empty))
    }
}
