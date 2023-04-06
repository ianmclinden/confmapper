use actix_web::{middleware, web::Data, App, HttpServer};
use phone_list::PhoneNumber;
use std::{env, io};
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
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    let args = Args::from_args();

    let mut numbers: Vec<PhoneNumber> = Vec::new();
    if args.phone_list.is_some() {
        numbers = serde_json::from_str(&args.phone_list.unwrap()).expect("Invalid phone list");
    }
    let phone_list = Data::new(numbers);

    if args.id_length < 6 || args.id_length > 12 {
        panic!("Invalid id length: '{}' (0-6)", args.id_length);
    }
    let id_length = Data::new(args.id_length);

    env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    let db = Data::new(sled::open(args.database)?);

    HttpServer::new(move || {
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
    })
    .bind(("0.0.0.0", args.port))?
    .run()
    .await
}
