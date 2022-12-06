use std::{future::{ready, Ready}, path::{PathBuf, Path}, fs};

use actix_web::{
    body::EitherBody,
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    Error, HttpRequest, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use tracing::{info};

use crate::configuration::Configuration;

enum AuthenticationState {
    Unauthorized,
    Authorized { login: String, password: String },
}

enum Query {
    Invalid,
    Success {
        repository: String,
        path: String,
        branch: String,
    },
}

pub struct ConfigServer {
    configuration: Configuration,
    repository_path: PathBuf
}

impl ConfigServer {
    pub fn new(config: Configuration, root: PathBuf) -> Self {
        ConfigServer {
            configuration: config,
            repository_path: root
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
            repository_path: self.repository_path.clone()

        }))
    }
}
pub struct ConfigServerMiddleware<S> {
    service: S,
    configuration: Configuration,
    repository_path: PathBuf
}

impl<S, B> ConfigServerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    fn not_found(
        &self,
        request: HttpRequest,
    ) -> LocalBoxFuture<'static, Result<ServiceResponse<EitherBody<B>>, Error>> {
        let response = HttpResponse::NotFound().finish();
        return Box::pin(async {
            Ok(ServiceResponse::new(
                request,
                response.map_into_right_body(),
            ))
        });
    }

    fn unauthorized(
        &self,
        request: HttpRequest,
    ) -> LocalBoxFuture<'static, Result<ServiceResponse<EitherBody<B>>, Error>> {
        let response = HttpResponse::Unauthorized()
            .append_header((
                header::WWW_AUTHENTICATE,
                "Basic realm=\"ConfigServer\", charset=\"UTF-8\"",
            ))
            .finish();
        return Box::pin(async {
            Ok(ServiceResponse::new(
                request,
                response.map_into_right_body(),
            ))
        });
    }
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

    fn call(&self, rq: ServiceRequest) -> Self::Future {
        if rq.path().starts_with("/encrypt") {
            let res = self.service.call(rq);

            return Box::pin(async move {
                // forwarded responses map to "left" body
                res.await.map(ServiceResponse::map_into_left_body)
            });
        }
        

        let (request, _pl) = rq.into_parts();

        let (login, password) = match is_request_authorized(&request) {
            AuthenticationState::Unauthorized => return self.unauthorized(request),
            AuthenticationState::Authorized { login, password } => (login, password),
        };

        let (repo, path, _branch) = match parse_query(request.path()) {
            Query::Invalid => return self.not_found(request),
            Query::Success {
                repository,
                path,
                branch,
            } => (repository, path, branch),
        };

        let repo_config = match self.configuration.repository(&repo)
        {
            Some(c) => c,
            None => return self.not_found(request),
        };       

        if !repo_config.is_granted_for(&login, &password) {
            return self.unauthorized(request);
        }

        let content = load_content(self.repository_path.to_str().unwrap(), &repo, &path);
        

        let response = HttpResponse::Ok().body(content).map_into_right_body();
        return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
    }
}

/// Ensures the provided request bears a basic Authorization scheme
fn is_request_authorized(request: &HttpRequest) -> AuthenticationState {
    if !request.headers().contains_key(header::AUTHORIZATION) {
        return AuthenticationState::Unauthorized;
    }

    let auth_header = request.headers().get(header::AUTHORIZATION).unwrap();
    let mut auth_str = auth_header.to_str().unwrap();
    if !auth_str.starts_with("Basic "){
        // We only support Basic auth
        return AuthenticationState::Unauthorized
    }

    auth_str = auth_str.strip_prefix("Basic ").unwrap();
    let bytes = base64::decode(auth_str).unwrap();
    let credentials = String::from_utf8(bytes).unwrap();
    let login_pwd: Vec<&str> = credentials.split(":").collect();

    AuthenticationState::Authorized {
        login: login_pwd[0].to_owned(),
        password: login_pwd[1].to_owned(),
    }
}

/// Morphs the inbound path to a repositozy, path and optional branch
fn parse_query(request_path: &str) -> Query {
    let path_elements: Vec<&str> = request_path.split('/').collect();
    if path_elements.len() < 2 {
        return Query::Invalid;
    }

    return Query::Success {
        repository: path_elements[1].to_owned(),
        path: path_elements[2].to_owned(),
        branch: " ".to_string(),
    };
}

fn load_content(repository_path: &str, repository: &str, file_path : &str) ->String{
    let p = Path::new(repository_path).join(repository).join(file_path);
    info!("{:?}", p);

    let content = fs::read_to_string(p).unwrap();

    content
}
