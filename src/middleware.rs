use std::future::{ready, Ready};

use actix_web::{
    body::EitherBody,
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    http::{header},
    Error, HttpRequest, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use tracing::debug;

use crate::config::{Configuration, Repo};

pub struct ConfigServer {
    configuration: Configuration,
}

impl ConfigServer {
    pub fn new(config: Configuration) -> Self {
        ConfigServer {
            configuration: config,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ConfigServer
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = ConfigServerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ConfigServerMiddleware {
            service: service,
            configuration: self.configuration.clone(),
        }))
    }
}
pub struct ConfigServerMiddleware<S> {
    service: S,
    configuration: Configuration,
}

impl<S, B> Service<ServiceRequest> for ConfigServerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        // Change this to see the change in outcome in the browser.
        // Usually this boolean would be acquired from a password check or other auth verification.
        let is_logged_in = false;

        // Don't forward to `/login` if we are already on `/login`.
        if !is_logged_in && request.path() != "/login" {
            let (request, _pl) = request.into_parts();

            if !request.headers().contains_key(header::AUTHORIZATION) {
                let response = HttpResponse::Unauthorized()
                    .append_header((
                        header::WWW_AUTHENTICATE,
                        "Basic realm=\"ConfigServer\", charset=\"UTF-8\"",
                    ))
                    .finish()
                    .map_into_right_body();
                return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
            } else {
                let _ = is_authorized(&request);
                let _ = match_request(&request);
                let response = HttpResponse::Ok().body("Hey there!").map_into_right_body();
                return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
            }

            // let response = HttpResponse::Found()
            //     .insert_header((http::header::LOCATION, "/login"))
            //     .finish()
            //     // constructed responses map to "right" body
            //     .map_into_right_body();
        }

        let res = self.service.call(request);

        Box::pin(async move {
            // forwarded responses map to "left" body
            res.await.map(ServiceResponse::map_into_left_body)
        })
    }
}

// fn select_repository(configuration: &Configuration, path:String)-> &Repo{

// }

fn match_repository(repositories: Vec<Repo>){

}

fn match_request(request: &HttpRequest)->(&str, &str){
    let p= request.path();
    let q = request.query_string();
    debug!("{:?}   {:?}", p,q);
    let uri = request.uri();
    let path = uri.path();
    let x = uri.path_and_query().unwrap();
    debug!("{:?}", x);
    ("","")
}



fn is_authorized(request: &HttpRequest) -> bool {
    let auth = request.headers().get(header::AUTHORIZATION).unwrap();
    debug!("{:?}", auth);
    let authstr=auth.to_str().unwrap();
    let b64 = authstr.strip_prefix("Basic ").unwrap();
    let bytes = base64::decode(b64).unwrap();
    let creds = String::from_utf8(bytes).unwrap();
    let splitted = creds.split(":");
    false
}
