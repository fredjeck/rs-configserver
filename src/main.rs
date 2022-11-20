use actix_web::{
    get, http::header, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};

#[get("/")]
async fn hello(_req: HttpRequest) -> impl Responder {
    if (!_req.headers().contains_key(header::AUTHORIZATION)) {
        return HttpResponse::Unauthorized()
            .append_header((
                header::WWW_AUTHENTICATE,
                "Basic realm=\"ConfigServer\", charset=\"UTF-8\"",
            ))
            .finish();
    }else{
        let auth = _req.headers().get(header::AUTHORIZATION).unwrap();
        print!("{:?}", auth);
        return HttpResponse::Ok().body("Hey there!")
    }
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
