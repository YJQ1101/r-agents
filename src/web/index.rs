use actix_web::{web, HttpResponse, Responder};

fn handle_index_file(path: &str) -> HttpResponse {
  HttpResponse::Ok().body("good")
}

async fn index() -> impl Responder {
    handle_index_file("index.html")
}

pub fn index_config(config:&mut web::ServiceConfig){
    config.service(web::scope("/rllamar")
        .service(web::resource("/index").route(web::get().to(index)))
        // .service(icon)
        // .service(assets)
        .service(web::resource("/index.html").route(web::get().to(index)))
        .service(web::resource("/404").route(web::get().to(index))))
    ;
}
