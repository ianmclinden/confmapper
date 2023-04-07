use std::{env, io, path::Path};

use actix_web::{middleware, web::Data, App, HttpServer};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use phone_list::PhoneNumbers;

use structopt::StructOpt;

mod conference;
mod phone_list;

#[derive(Debug, StructOpt)]
#[structopt(about)]
struct Args {
    /// Conference mapper database path
    #[structopt(
        short,
        long,
        default_value = "mapper.db",
        env = "CONFERENCE_MAPPER_DATABASE"
    )]
    database: String,

    /// API listen port
    #[structopt(short, long, default_value = "9000", env = "CONFERENCE_MAPPER_PORT")]
    port: u16,

    /// JSON dial-in phone number list
    #[structopt(short = "l", long, env = "CONFERENCE_MAPPER_PHONELIST")]
    phone_list: Option<String>,

    /// Number of digits to use for conference id (6-12)
    #[structopt(short, long, env = "CONFERENCE_MAPPER_ID_LENGTH", default_value = "6")]
    id_length: u32,

    /// TLS Certificate file
    #[structopt(
        short = "c",
        long = "certs",
        env = "CONFERENCE_MAPPER_TLS_CERT",
        default_value = "keys/cert.crt"
    )]
    tls_cert_file: String,

    /// TLS Key file
    #[structopt(
        short = "k",
        long = "key",
        env = "CONFERENCE_MAPPER_TLS_KEY",
        default_value = "keys/cert.key"
    )]
    tls_key_file: String,
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    let args = Args::from_args();

    let mut numbers = PhoneNumbers::new();
    if args.phone_list.is_some() {
        numbers = serde_json::from_str(&args.phone_list.unwrap()).expect("Invalid phone list");
    }
    let phone_list = Data::new(numbers);

    if args.id_length < 6 || args.id_length > 12 {
        panic!("Invalid id length: '{}' (0-6)", args.id_length);
    }
    let id_length = Data::new(args.id_length);

    let mut sslbuilder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    let mut uses_openssl = false;
    if Path::new(&args.tls_cert_file).exists() && Path::new(&args.tls_key_file).exists() {
        sslbuilder
            .set_private_key_file(args.tls_key_file, SslFiletype::PEM)
            .expect("Could not read TLS key file");
        sslbuilder
            .set_certificate_chain_file(args.tls_cert_file)
            .expect("Could not read TLS cert file");
        uses_openssl = true;
    }

    env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    let db = Data::new(sled::open(args.database)?);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            // Shared data
            .app_data(Data::clone(&db))
            .app_data(Data::clone(&phone_list))
            .app_data(Data::clone(&id_length))
            // HTTP request handlers
            .service(conference::get)
            .service(conference::set)
            .service(phone_list::get)
    });
    if uses_openssl {
        println!("Listening on https://0.0.0.0:{}", args.port);
        server
            .bind_openssl(("0.0.0.0", args.port), sslbuilder)?
            .run()
            .await
    } else {
        println!("Listening on http://0.0.0.0:{}", args.port);
        server.bind(("0.0.0.0", args.port))?.run().await
    }
}
