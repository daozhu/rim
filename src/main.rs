#[macro_use]
extern crate diesel;

use actix_web::{HttpServer, App, web, HttpRequest, HttpResponse, Error};
use actix_web_actors::ws;
use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;
use actix::{Actor, StreamHandler};
use actix_files;

pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

mod schema;
mod models;
mod routes;

struct Ws;

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Ws {
    fn handle(&mut self,
              msg: Result<ws::Message, ws::ProtocolError>,
              ctx: &mut Self::Context,
    ) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

async fn ws_handle(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(Ws{}, &req, stream);
    println!("{:?}", resp);
    resp
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("can not find database");
    let database_pool = Pool::builder()
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .unwrap();
    HttpServer::new(move || {
        App::new()
            .data(database_pool.clone())
            .service(routes::add_product)
            .service(routes::get_all_product)
            .service(routes::del_product)
            .service(
                actix_files::Files::new("/static", "./static")
                    .show_files_listing()
            )
            .service(
                web::resource("/ws/").to(ws_handle)
            )
            .service(
                web::resource("/").to(routes::home)
            )
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
