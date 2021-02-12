use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use env_logger;
mod routing;
use dotenv;
mod stats;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
mod auth;

use stats::StatPool;

#[derive(Serialize)]
struct ErrorResp<'a> {
    message: &'a str,
}

pub async fn resp_not_found() -> HttpResponse {
    HttpResponse::NotFound().json(ErrorResp {
        message: "Page not found",
    })
}

async fn greet(_req: HttpRequest) -> impl Responder {
    println!("HELLO");
    HttpResponse::Ok().json(ErrorResp {
        message: "HELLO WORLD",
    })
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    std::env::set_var("RUST_BACKTRACE", "1");
    //dotenv::dotenv().ok();
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&std::env::var("DATABASE_URL_MAIN").expect("DATABASE_URL_MAIN not set"))
        .await
        .unwrap();

    let sp = PgPoolOptions::new()
        .max_connections(10)
        .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL not set"))
        .await
        .unwrap();
    let st = StatPool { pool: sp };

    sqlx::query("UPDATE TOKENS SET uses = 0;")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("SELECT * FROM stats;")
        .execute(&st.pool)
        .await
        .unwrap();
    env_logger::init();
    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(st.clone())
            .route("/", web::get().to(greet))
            .wrap(auth::RequiresAuth)
            .configure(routing::init_routes)
            .configure(stats::init_routes)
            .default_service(web::route().to(resp_not_found))
            .wrap(middleware::Logger::default())
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
