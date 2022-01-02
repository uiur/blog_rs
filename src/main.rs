use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Responder, get, HttpRequest, body};
use env_logger::Env;
use handlebars::Handlebars;
use log::error;
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Serialize)]
struct Post {
    id: String,
    title: String,
}

fn generate_posts() -> Vec<Post> {
    (0..10)
        .into_iter()
        .map(|i| Post {
            id: Uuid::new_v4().to_string(),
            title: format!("post_{}", i),
        })
        .collect()
}


#[get("/")]
async fn index(hb: web::Data<Handlebars<'_>>) -> impl Responder {
    let posts = generate_posts();
    let body = hb.render("index", &json!({ "posts": posts }));

    match body {
        Ok(s) => HttpResponse::Ok().body(s),
        Err(e) => {
            error!("{}", e);
            HttpResponse::InternalServerError().into()
        }
    }
}

#[get("/posts/{id}")]
async fn show(hb: web::Data<Handlebars<'_>>, info: web::Path<(String,)>) -> impl Responder {
    let id = info.into_inner().0;
    let post = Post { id: id, title: "foobar".to_string() };
    let body = hb.render("show", &json!({ "post": post }));
    match body {
        Ok(s) => HttpResponse::Ok().body(s),
        Err(e) => {
            error!("{}", e);
            HttpResponse::InternalServerError().into()
        }
    }

}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".html", "./src/templates")
        .unwrap();

    let handlebars_ref = web::Data::new(handlebars);

    HttpServer::new(move || {
        App::new()
            .app_data(handlebars_ref.clone())
            .wrap(Logger::default())
            .service(index)
            .service(show)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
