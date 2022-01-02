use std::env;

use actix_web::{
    body, error, get, middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use env_logger::Env;
use handlebars::Handlebars;
use log::error;
use serde::Serialize;
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;
extern crate dotenv;

use dotenv::dotenv;

#[derive(Serialize)]
struct Post {
    id: String,
    title: String,
    body: String,
}

struct PostRecord {
    id: sqlx::types::Uuid,
    title: String,
    body: String,
}

#[get("/")]
async fn index(
    hb: web::Data<Handlebars<'_>>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let posts: Vec<_> = sqlx::query_as!(PostRecord, "select id, title, body from posts")
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?
        .into_iter()
        .map(|record| Post {
            id: record.id.to_string(),
            title: record.title,
            body: record.body,
        })
        .collect();

    let body = hb
        .render("index", &json!({ "posts": posts }))
        .map_err(|e| error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().body(body))
}

#[get("/posts/{id}")]
async fn show(
    hb: web::Data<Handlebars<'_>>,
    pool: web::Data<PgPool>,
    info: web::Path<(String,)>,
) -> Result<HttpResponse, Error> {
    let id = info.into_inner().0;
    let uuid = sqlx::types::Uuid::parse_str(&id).map_err(|e| error::ErrorNotFound(e))?;
    let record = sqlx::query_as!(PostRecord, "select id, title, body from posts where id = $1", uuid)
        .fetch_one(pool.as_ref())
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    let post =
        Post {
            id: record.id.to_string(),
            title: record.title,
            body: record.body,
        };

    let body = hb
        .render("show", &json!({ "post": post }))
        .map_err(|e| error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().body(body))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect database");

    let pool_ref = web::Data::new(pool);

    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".html", "./src/templates")
        .unwrap();

    let handlebars_ref = web::Data::new(handlebars);

    HttpServer::new(move || {
        App::new()
            .app_data(handlebars_ref.clone())
            .app_data(pool_ref.clone())
            .wrap(Logger::default())
            .service(index)
            .service(show)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
