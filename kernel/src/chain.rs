use crate::kernel::{ReturnData};
use crate::kernel::{
    Parcel, Note
};
use crate::{network::*, utility};

use actix::prelude::*;
use futures_lite::future;

// use sp_keyring::AccountKeyring
use subxt::{
    ext::sp_core::{crypto::Pair, ed25519::Pair as ed25519Pair},
    tx::{
        Era,
        PairSigner,
        PlainTip,
        PolkadotExtrinsicParamsBuilder as Params,
        PolkadotExtrinsicParams
    },
    OnlineClient,
    config::{ SubstrateConfig, WithExtrinsicParams }
};

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod samaritan_node { }

type PolkadotConfig = WithExtrinsicParams<SubstrateConfig, PolkadotExtrinsicParams<SubstrateConfig>>;

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

    // HANGS!!!
    // pub async fn get_did_and_keys(&self, str: &str) -> Result<Parcel, subxt::Error> {
    //     // we have our 12 words now
    //     let key_box = Pair::generate_with_phrase(None);
    //     let pair: ed25519Pair = key_box.0;

    //     // get did
    //     let did = String::from("did:sam:root:") + &utility::get_random_str(32);

    //     // upload to IPFS, but run it on another thread to minimize delay
    //     let ipfs_data = Network::upload_to_ipfs(str.to_owned()).await;

    //     let cid = match ipfs_data.unwrap().0 {
    //         Parcel::String(str) => str.clone(),
    //         _ => { String::new() }
    //     };

    //     // send message to network actor to upload to IPFS
    //     self.network_addr.do_send(Note(101, Parcel::Empty));

    //     // send transaction onchain
    //     let signer = PairSigner::<PolkadotConfig, ed25519Pair>::new(pair);
    //     let api = OnlineClient::<PolkadotConfig>::new().await?;

    //     // Create a transaction to submit:
    //     let tx = samaritan_node::tx()
    //         .kernel()
    //         .record_data_entry("samaritan_root_document".as_bytes().to_vec(), Vec::from(cid.to_owned()));

    //     // Configure the transaction tip and era:
    //     let tx_params = Params::new()
    //         .tip(PlainTip::new(20_000_000_000))
    //         .era(Era::Immortal, api.genesis_hash());

    //     // submit the transaction:
    //     let hash = api.tx().sign_and_submit(&tx, &signer, tx_params).await?;
    //     println!("Samaritans root document tx submitted: {}", hash);

    //     // return did and keys
    //     Ok(Parcel::Tuple1(did, key_box.1))

    // }

    // parody
    pub async fn get_did_and_keys(&self, str: &str) -> Result<Parcel, subxt::Error> {
        Ok(utility::get_did_and_keys_mimick(str))
    }

}

impl Actor for ChainClient {
    type Context = Context<Self>;
}

impl Handler<Note> for ChainClient {
    type Result = Result<ReturnData, std::io::Error>;

    /// handle incoming "Note" and dispatch to various appropriate methods
    fn handle(&mut self, msg: Note, _: &mut Context<Self>) -> Self::Result {
        match &msg.0 {
            101 => {
                future::block_on(async {
                    match msg.1 {
                        Parcel::String(str) => {
                            Ok::<ReturnData, std::io::Error>(ReturnData((self.get_did_and_keys(&str).await).unwrap_or(Parcel::Empty)))
                        }
                        _ => Ok::<ReturnData, std::io::Error>(ReturnData(Parcel::Empty))
                    }
                })
            },
            _ => Ok(ReturnData(Parcel::Empty))
        }
    }
}
