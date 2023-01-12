use crate::kernel::{self, ReturnData};
use crate::kernel::{
    Parcel, Note
};
use crate::{network::*, utility};

use actix::prelude::*;

// use sp_keyring::AccountKeyring;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use subxt::{
    config::SubstrateConfig,
    ext::sp_core::{crypto::Pair, ed25519::Pair as ed25519Pair},
    storage::StorageKey,
    tx::PairSigner,
    OnlineClient, PolkadotConfig,
};

#[derive(Debug)]
pub struct ChainClient {
    network_addr: Addr<Network>
}

impl ChainClient {
    pub fn new(network_addr: Addr<Network>) -> ChainClient {
        ChainClient {
            network_addr
        }
    }

    pub async fn get_id_and_keys(&self, str: &str) -> Result<(), subxt::Error> {
        // we have our 12 words now
        let key_box = Pair::generate_with_phrase(None);
        let pair: ed25519Pair = key_box.0;

        // get did
        let did = kernel::Kernel::generate_user_did(&pair.public().0[..]);
        let did_doc_sk = StorageKey(str.into());
        let mut hasher = DefaultHasher::new();
        did_doc_sk.hash(&mut hasher);

        // hash the did document
        let root_doc_hash = hasher.finish();

        // upload to IPFS, but run it on another thread to minimize delay
        let ipfs_data = Network::upload_to_ipfs(str.to_owned()).await;
        let cid = match ipfs_data.unwrap() {
            ReturnData::String(str) => str,
            _ => { String::new() }
        };

        // save to local file and upload to ipfs in background
        utility::update_hash_table(did, cid);

        // send message to network actor to upload to IPFS
        self.network_addr.do_send(Note(101, Parcel::Empty));


        // let signer = PairSigner::<SubstrateConfig, ed25519Pair>::new(pair);

        // // Create a client to use:
        // let api = OnlineClient::<PolkadotConfig>::new().await?;

        //
        Ok(())
    }
}

impl Actor for ChainClient {
    type Context = Context<Self>;
}

impl Handler<Note> for ChainClient {
    type Result = Result<ReturnData, std::io::Error>;

    /// handle incoming "Note" and dispatch to various appropriate methods
    fn handle(&mut self, msg: Note, ctx: &mut Context<Self>) -> Self::Result {
        match &msg.0 {
            101 => {
                async {
                    match msg.1 {
                        Parcel::String(str) => {
                            let _ = self.get_id_and_keys(&str).await;
                        }
                        _ => {}
                    }
                };
            }
            _ => {}
        }

        Ok(ReturnData::Nothing)
    }
}
