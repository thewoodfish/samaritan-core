use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use actix::prelude::*;
use actix_web_actors::ws;
use futures_lite::future;
use serde_json::{json, Value};
use std::str;

use crate::{
    chain,
    network,
};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

// message to be passed between actors
pub struct Note(pub u32, pub Parcel);

// most common hashmap used
pub type StringHashMap = HashMap<String, String>;

/// parcel containing data
#[derive(Debug)]
pub enum Parcel {
    Empty,
    String(String),
    Tuple1(String, String),
}

/// types returned from actor messages
#[derive(Debug)]
pub struct ReturnData(pub Parcel);

impl Message for Note {
    type Result = Result<ReturnData, std::io::Error>;
}

#[derive(Debug)]
pub struct Kernel {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    pub hb: Instant,
    /// address of chain client actor
    pub ccl_addr: Addr<chain::ChainClient>,
    // address of network actor
    pub net_addr: Addr<network::Network>,
}

impl Kernel {
    /// helper method that sends ping to client every 5 seconds (HEARTBEAT_INTERVAL).
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for Kernel {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        Running::Stop
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Kernel {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        log::debug!("WEBSOCKET MESSAGE: {msg:?}");
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                log::debug!("{}", text);
                let m = text.trim();

                // dispatch functions to respective actors based on numbers
                let v: Vec<&str> = m.splitn(2, '#').collect();
                let mut error = json!({
                    "res": "fatal: an error has occured",
                    "error": true
                })
                .to_string();

                future::block_on(async {
                    match v[0] {
                        "~0" => {
                            error = json!({
                                "status": "Not found"
                            })
                            .to_string();

                            let ret = self
                                .net_addr
                                .send(Note(100, Parcel::String(v[1].to_owned())))
                                .await;

                            if let Ok(data_result) = ret {
                                if let Ok(data) = data_result {
                                    let parcel = data.0;
                                    println!("{:?}", parcel);
                                    if let Parcel::String(str) = parcel {
                                        ctx.text(str)
                                    } else {
                                        ctx.text(error)
                                    }
                                } else {
                                    ctx.text(error)
                                }
                            } else {
                                ctx.text(error)
                            }
                        }
                        
                        "~1" => {
                            // create new samaritan
                            match self
                                .ccl_addr
                                .send(Note(101, Parcel::String(v[1].to_owned())))
                                .await
                            {
                                Ok(ret) => match ret {
                                    Ok(data) => match data.0 {
                                        Parcel::Tuple1(did, keys) => {
                                            let res = json!({
                                                "did": did,
                                                "keys": keys,
                                                "error": false
                                            });
                                             
                                            ctx.text(res.to_string())
                                        }
                                        _ => ctx.text(error),
                                    },
                                    Err(_) => ctx.text(error),
                                },
                                Err(_) => ctx.text(error),
                            };
                        },

                        "~2" => {
                            // new API key
                            match self.ccl_addr.send(Note(102, Parcel::Empty)).await {
                                Ok(ret) => match ret {
                                    Ok(data) => match data.0 {
                                        Parcel::Tuple1(did, keys) => {
                                            let res = json!({
                                                "did": did,
                                                "keys": keys,
                                                "error": false
                                            });

                                            ctx.text(res.to_string())
                                        }
                                        _ => ctx.text(error),
                                    },
                                    Err(_) => ctx.text(error),
                                },
                                Err(_) => ctx.text(error),
                            };
                        }

                        "~3" => {
                            // auth did
                            match self
                                .ccl_addr
                                .send(Note(103, Parcel::String(v[1].to_owned())))
                                .await
                            {
                                Ok(ret) => match ret {
                                    Ok(data) => match data.0 {
                                        Parcel::Tuple1(exists, did) => {
                                            let res = json!({
                                                "exists": exists,
                                                "did": did,
                                                "error": false
                                            });

                                            ctx.text(res.to_string())
                                        }
                                        _ => ctx.text(error),
                                    },
                                    Err(_) => ctx.text(error),
                                },
                                Err(_) => ctx.text(error),
                            }
                        }

                        "~4" => {   // insert into database
                            match self
                                .net_addr
                                .send(Note(103, Parcel::String(v[1].to_owned())))
                                .await
                            {
                                Ok(ret) => match ret {
                                    Ok(data) => match data.0 {
                                        Parcel::String(_) => {
                                            let res = json!({
                                                "success": true,
                                                "error": false
                                            });

                                            ctx.text(res.to_string())
                                        }
                                        _ => ctx.text(error),
                                    },
                                    Err(_) => ctx.text(error),
                                },
                                Err(_) => ctx.text(error),
                            }
                        }

                        "~5" => {
                            error = json!({
                                "status": "Not found"
                            })
                            .to_string();

                            let ret = self
                                .net_addr
                                .send(Note(104, Parcel::String(v[1].to_owned())))
                                .await;

                            if let Ok(data_result) = ret {
                                if let Ok(data) = data_result {
                                    let parcel = data.0;
                                    if let Parcel::String(val_str) = parcel {
                                        let mut res: Value = serde_json::from_str(&val_str).unwrap();
                                        res["status"] = "found".into();
                                        ctx.text(res.to_string())
                                    } else {
                                        ctx.text(error)
                                    }
                                } else {
                                    ctx.text(error)
                                }
                            } else {
                                ctx.text(error)
                            }
                        }

                        "~6" => {
                            error = json!({
                                "success": false
                            })
                            .to_string();

                            let ret = self
                                .net_addr
                                .send(Note(105, Parcel::String(v[1].to_owned())))
                                .await;

                            if let Ok(data_result) = ret {
                                if let Ok(data) = data_result {
                                    let parcel = data.0;
                                    if let Parcel::String(val_str) = parcel {
                                        let res: Value = serde_json::from_str(&val_str).unwrap();
                                        ctx.text(res.to_string())
                                    } else {
                                        ctx.text(error)
                                    }
                                } else {
                                    ctx.text(error)
                                }
                            } else {
                                ctx.text(error)
                            }
                        }

                        "~7" => {
                            error = json!({
                                "success": false
                            })
                            .to_string();

                            let ret = self
                                .ccl_addr
                                .send(Note(104, Parcel::String(v[1].to_owned())))
                                .await;

                            if let Ok(data_result) = ret {
                                if let Ok(data) = data_result {
                                    let parcel = data.0;
                                    if let Parcel::String(app_did) = parcel {
                                        let res: Value = json!({
                                            "success": true,
                                            "did": app_did
                                        });
                                        ctx.text(res.to_string())
                                    } else {
                                        ctx.text(error)
                                    }
                                } else {
                                    ctx.text(error)
                                }
                            } else {
                                ctx.text(error)
                            }
                        }

                        "~8" => {
                            error = json!({
                                "success": false
                            })
                            .to_string();

                            let ret = self
                                .net_addr
                                .send(Note(106, Parcel::String(v[1].to_owned())))
                                .await;

                            if let Ok(data_result) = ret {
                                if let Ok(data) = data_result {
                                    let parcel = data.0;
                                    if let Parcel::String(data_req) = parcel {
                                        let res: Value = json!({
                                            "success": true,
                                            "data": data_req
                                        });
                                        ctx.text(res.to_string())
                                    } else {
                                        ctx.text(error)
                                    }
                                } else {
                                    ctx.text(error)
                                }
                            } else {
                                ctx.text(error)
                            }
                        }
                        _ => {}
                    }
                });
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
