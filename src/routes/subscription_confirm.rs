use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Params {
    token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(_params))]
pub async fn confirm(_params: web::Query<Params>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
