#![feature(vec_into_raw_parts)]
#[macro_use] extern crate lazy_static;

use warp::{
    Filter,
    http::StatusCode,
};
use serde::{Serialize};
use std::{env, net, convert::Infallible};

mod links;
mod upload_image;
mod get_index;

#[derive(Clone)]
pub struct AppParameters {
    socket: net::SocketAddr,
    storage_path: String,
    web_root: String,
}

impl AppParameters {
    pub fn get() -> AppParameters {

        let socket: net::SocketAddr = env::var("APP_SOCKET")
            .unwrap_or("0.0.0.0:3000".into())
            .parse()
            .expect("Bad socket address");

        let storage_path = env::var("APP_STORAGE")
            .unwrap_or("/storage".into());

        let web_root = env::var("WEB_ROOT")
            .unwrap_or("/webroot".into());

        AppParameters {
            socket, storage_path, web_root
        }
    }
}

#[tokio::main]
async fn main() {
    let app = AppParameters::get();

    warp::serve(
        upload_image::filter(app.clone())
            .or(warp::fs::dir(app.web_root))
            .or(warp::path("img").and(warp::fs::dir(app.storage_path.clone())))
            .or(get_index::filter("images".into(), app.storage_path))
            .recover(handle_rejection)
    )
        .run(app.socket)
        .await;
}

fn with_params(app: AppParameters) -> impl Filter<Extract = (AppParameters,), Error = Infallible> + Clone {
    warp::any().map(move || app.clone())
}

#[macro_export]
macro_rules! filter {
    ($E:ty) => {impl Filter<Extract = $E, Error = warp::Rejection> + Clone}
}

#[derive(Debug)]
enum Errors {
    Multipart,
    Base64Decoding,
    Internal,
    Database,
    ImageDecoding,
}

impl warp::reject::Reject for Errors {}

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    let (code, message) =
        if err.is_not_found() {
            (StatusCode::NOT_FOUND, "Resourse not found")
        }
        else if let Some(Errors::Multipart) = err.find() {
            (StatusCode::UNPROCESSABLE_ENTITY, "Error decoding multipart/form-data")
        }
        else if let Some(Errors::Base64Decoding) = err.find() {
            (StatusCode::UNPROCESSABLE_ENTITY, "Error decoding base64 image data")
        }
        else if let Some(Errors::Internal) = err.find() {
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        }
        else if let Some(Errors::Database) = err.find() {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        }
        else if let Some(Errors::ImageDecoding) = err.find() {
            (StatusCode::UNPROCESSABLE_ENTITY, "Error decoding image")
        }
        else {
            eprintln!("Unhandled rejection: {:?}", err);
            (StatusCode::BAD_REQUEST, "Bad request")
            /*
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            */
        };

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}

