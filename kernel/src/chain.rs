use actix::prelude::*;
use crate::kernel;

#[derive(Debug)]
pub struct ChainClient {

}

type Note = kernel::Note;

impl ChainClient {
    pub fn new() -> ChainClient {
        ChainClient {}
    }
}

impl Actor for ChainClient {
    type Context = Context<Self>;
}

impl Handler<Note> for ChainClient {
    type Result = Result<bool, std::io::Error>;

    /// handle incoming "Note" and dispatch to various appropriate methods
    fn handle(&mut self, msg: Note, ctx: &mut Context<Self>) -> Self::Result {
        match msg.0 {
            101 => {

            },
            _ => {}
        }
        Ok(true)
    }
}
