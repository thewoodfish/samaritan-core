use crate::kernel::{self, ReturnData, Parcel};
use actix::prelude::*;
use ipfs_api_backend_actix::ApiError;

use ipfs_api::{IpfsApi, IpfsClient};
use std::io::Cursor;


#[derive(Debug)]
pub struct Network {
    hash_key: String
}

type Note = kernel::Note;

impl Network {
    pub fn new(hash_key: String) -> Network {
        Network { hash_key }
    }

    pub async fn upload_to_ipfs(str: String) -> Result<ReturnData, ApiError> {
        let client = IpfsClient::default();
        let data = Cursor::new(str);

        match client.add(data).await {
            Ok(res) => {
                Ok(ReturnData::String(res.hash))
            },
            Err(e) => {
                Err(ApiError {
                    message: "could not upload to IPFS".to_string(),
                    code: 102
                })
            }
        }
    }

    pub async fn update_hash_table() {
        // Open the file in read-only mode with buffer.
        let path = "../record/hash_table.json";
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
    }
}

impl Actor for Network {
    type Context = Context<Self>;
}

impl Handler<Note> for Network {
    type Result = Result<ReturnData, std::io::Error>;

    /// handle incoming "Note" and dispatch to various appropriate methods
    fn handle(&mut self, msg: Note, ctx: &mut Context<Self>) -> Self::Result {
        match &msg.0 {
            101 => {
                
            }
            _ => {}
        }
        
        Ok(ReturnData::Nothing)
    }
}
