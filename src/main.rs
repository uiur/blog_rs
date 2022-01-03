use std::env;

use actix_web::{
    delete, error, get, http::header::HttpDate, middleware::Logger, post, web, App, Error,
    HttpResponse, HttpServer,
};
use env_logger::Env;
use handlebars::Handlebars;

use serde::Deserialize;
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};

extern crate dotenv;

use dotenv::dotenv;

mod model;
use crate::model::post::*;

#[get("/")]
async fn index(
    hb: web::Data<Handlebars<'_>>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let posts = Post::find_all(pool.get_ref())
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

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
    let post = Post::find(pool.get_ref(), &id)
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    let body = hb
        .render("show", &json!({ "post": post }))
        .map_err(|e| error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().body(body))
}

#[derive(Deserialize)]
struct PostFormData {
    title: String,
    body: String,
}

#[post("/posts")]
async fn create(
    pool: web::Data<PgPool>,
    form: web::Form<PostFormData>,
) -> Result<HttpResponse, Error> {
    let post = Post::create(pool.get_ref(), &form.title, &form.body)
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Found()
        .append_header(("Location", format!("/posts/{}", post.id)))
        .finish())
}

#[derive(Deserialize)]
struct FormData {
    _method: String,
}

async fn destroy(
    pool: web::Data<PgPool>,
    info: web::Path<(String,)>,
    form: web::Form<FormData>,
) -> Result<HttpResponse, Error> {
    let id = info.into_inner().0;
    let post = Post::find(pool.get_ref(), &id)
        .await
        .map_err(|e| error::ErrorUnprocessableEntity(e))?;

    post.destroy(pool.get_ref())
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish())
}

#[post("/posts/{id}")]
async fn handle_post_resource(
    pool: web::Data<PgPool>,
    info: web::Path<(String,)>,
    form: web::Form<FormData>,
) -> Result<HttpResponse, Error> {
    if form._method == "DELETE" {
        return destroy(pool, info, form).await;
    }

    if form._method == "PATCH" {
        todo!()
    }

    Ok(HttpResponse::NotFound().finish())
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
            .service(create)
            .service(handle_post_resource)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
