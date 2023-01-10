use crate::kernel;
use actix::prelude::*;

use sp_keyring::AccountKeyring;
use subxt::{tx::PairSigner, OnlineClient, PolkadotConfig};

use random_word::*;
use sp_keyring::sr25519::sr25519::*;

// #[subxt::subxt(runtime_metadata_path = "../artifacts/polkadot_metadata.scale")]
// pub mod polkadot {}

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     tracing_subscriber::fmt::init();

//     let signer = PairSigner::new(AccountKeyring::Alice.pair());
//     let dest = AccountKeyring::Bob.to_account_id().into();

//     // Create a client to use:
//     let api = OnlineClient::<PolkadotConfig>::new().await?;

//     // Create a transaction to submit:
//     let tx = polkadot::tx()
//         .balances()
//         .transfer(dest, 123_456_789_012_345);

//     // Submit the transaction with default params:
//     let hash = api.tx().sign_and_submit_default(&tx, &signer).await?;

//     println!("Balance transfer extrinsic submitted: {}", hash);

//     Ok(())
// }

#[derive(Debug)]
pub struct ChainClient {}

type Note = kernel::Note;

impl ChainClient {
    pub fn new() -> ChainClient {
        ChainClient {}
    }

    pub fn get_id_and_keys(str: &str) {
        // generate random words
        let mut mnemonic = String::with_capacity(90);
        let mut i = 0;

        while i < 12 {
            let word = random_word::gen();
            if i != 0 {
                mnemonic.push(' ');
            }
            mnemonic.push_str(word);
            i += 1;
        }

        // extract password from JSON string
        let password = str
            .splitn(2, "password\":")
            .filter(|s| !(*s).contains("serviceEndpoint"))
            .collect::<String>()
            .trim_matches('"');

        // we have our 12 words now
        // let full_pair = Pair::from_entropy(mnemonic, )
    }
}

impl Actor for ChainClient {
    type Context = Context<Self>;
}

impl Handler<Note> for ChainClient {
    type Result = Result<bool, std::io::Error>;

    /// handle incoming "Note" and dispatch to various appropriate methods
    fn handle(&mut self, msg: Note, ctx: &mut Context<Self>) -> Self::Result {
        match &msg.0 {
            101 => {
                ChainClient::get_id_and_keys(&msg.1);
            }
            _ => {}
        }
        Ok(true)
    }
}
