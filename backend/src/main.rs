use env_logger::Env;
use exit_trust_root_lib::config::{Config, ConfigAndCache};
use actix_web::{web, App, HttpServer};
use crossbeam::queue::SegQueue;
use exit_trust_root_lib::endpoints::{return_signed_exit_contract_data, start_client_registration, submit_registration_code};
use exit_trust_root_lib::register_client_batch_loop::register_client_batch_loop;
use exit_trust_root_lib::signature_update_loop;
use log::info;
use openssl::ssl::{SslAcceptor, SslMethod};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::join;


const CONFIG_PATH: &str = "/etc/exit_trust_root.toml";
/// The backend RPC port for the info server fucntions implemented in this repo
pub const SERVER_PORT: u16 = 4050;

fn load_config(path: &str) -> Config {
    let contents = std::fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Failed to read config file: {}", path));
    toml::from_str(&contents).unwrap_or_else(|_| panic!("Failed to parse config file: {}", path))
}

#[actix_web::main]
async fn main() {
    // On Linux static builds we need to probe ssl certs path to be able to
    // do TLS stuff.
    openssl_probe::probe();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = load_config(CONFIG_PATH);
    start_exit_trust_root_server(args).await;
}

async fn start_exit_trust_root_server(config: Config) {
    let domain = config.url.clone();
    let https = config.https;
    let exit_contract_data_cache: ConfigAndCache = ConfigAndCache {
        config: Arc::new(config.clone()),
        cache: Arc::new(RwLock::new(HashMap::new())),
        registration_queue: Arc::new(SegQueue::new()),
        texts_sent: Arc::new(RwLock::new(HashMap::new())),
    };
    let sig_loop = signature_update_loop(exit_contract_data_cache.clone());
    let reg_loop = register_client_batch_loop(
        config.rpc,
        config.private_key,
        exit_contract_data_cache.registration_queue.clone(),
    );
    let web_data = web::Data::new(exit_contract_data_cache.clone());

    let server = HttpServer::new(move || {
        App::new()
            .service(return_signed_exit_contract_data)
            .service(start_client_registration)
            .service(submit_registration_code)
            .app_data(web_data.clone())
    });
    info!("Starting exit trust root server on {:?}", domain);

    let server = if https {
        // build TLS config from files
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        // set the certificate chain file location
        builder
            .set_certificate_chain_file(format!("/etc/letsencrypt/live/{}/fullchain.pem", domain))
            .unwrap();
        builder
            .set_private_key_file(
                format!("/etc/letsencrypt/live/{}/privkey.pem", domain),
                openssl::ssl::SslFiletype::PEM,
            )
            .unwrap();

        info!("Binding to SSL");
        server
            .bind_openssl(format!("{}:{}", domain, SERVER_PORT), builder)
            .unwrap()
    } else {
        info!("Binding to {}:{}", domain, SERVER_PORT);
        server.bind(format!("{}:{}", domain, SERVER_PORT)).unwrap()
    };

    // run these in the background, then block on the server becuase it handles
    // ctrl-c and other signals to shutdown.
    actix_web::rt::spawn(sig_loop);
    actix_web::rt::spawn(reg_loop);
    server.run().await.unwrap()
}
