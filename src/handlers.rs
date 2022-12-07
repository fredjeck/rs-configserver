use actix_web::{web::{Bytes, self}, HttpResponse};

use crate::{configuration::Configuration, crypto::encrypt_base64_string};

/// Parses the body content and encrypts it using the provided configuration
pub async fn encrypt_body_content(bytes: Bytes, data: web::Data<Configuration>) -> HttpResponse {
    let body = match String::from_utf8(bytes.to_vec()) {
        Ok(text) => text,
        Err(_) => return HttpResponse::BadRequest().finish()
    };
    HttpResponse::Ok().body(encrypt_base64_string(&data.encryption_key, &body))
}