use std::time::Instant;

use actix::*;
use actix_web::{
    middleware::Logger, web, App, Error, HttpRequest, HttpResponse,
    HttpServer, /* Responder , */
};
use actix_web_actors::ws;

mod chain;
mod kernel;
mod network;
mod utility;

/// Entry point for our websocket route
async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    ccl: web::Data<Addr<chain::ChainClient>>,
    net: web::Data<Addr<network::Network>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        kernel::Kernel {
            hb: Instant::now(),
            ccl_addr: ccl.get_ref().clone(),
            net_addr: net.get_ref().clone(),
        },
        &req,
        stream,
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("starting HTTP server at http://localhost:1509");

    // start the network actor
    let netwrk = network::Network::new("empty".to_string()).start();

    // start chain client actor
    let client = chain::ChainClient::new(netwrk.clone()).start();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .app_data(web::Data::new(netwrk.clone()))
            .route("/", web::get().to(chat_route))
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(("127.0.0.1", 1509))?
    .run()
    .await
}
