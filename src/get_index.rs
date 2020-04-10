use tokio::fs;
use warp::{ Filter, };
use warp::reject::custom as reject;
use crate::{
    Errors,
    filter,
};
use regex::Regex;
use horrorshow::helper::doctype;

pub fn filter (
    path: String,
    root: String,
) -> filter!(impl warp::Reply) {
    warp::get()
        .and(warp::path(path).map(move || root.clone()))
        .and_then(get_index)
}

async fn get_index (
    root: String
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut dir = fs::read_dir(root)
        .await
        .map_err(|_| {reject(Errors::Internal)})?;

    let mut ids = Vec::new();

    while let Some(entry) = dir.next_entry()
        .await
        .map_err(|_| {reject(Errors::Internal)})? {

        lazy_static! {
            static ref REG: Regex = Regex::new(r"([0-f]{16})-thumbnail").unwrap();
        }

        let fname = entry.file_name().to_string_lossy().into_owned();

        if let Some(caps) = REG.captures(&fname) {
            if let Some(id) = caps.get(1) {
                ids.push(id.as_str().to_string());
            }
        }
    }

    use horrorshow::html;

    let res = format!("{}", html! {
        : doctype::HTML;
        html {
            head {
                title : "Uploaded images";
            }
            body {
                h1(id="heading", class="title") : "Uploaded images";
                div {
                    @ for id in &ids {
                        a(href=format!("/img/{}.png", id), target="_blank") {
                            img(src=format!("/img/{}-thumbnail.png", id));
                        }
                    }
                }
                br; br;
            }
        }
    });

    Ok(warp::reply::html(res))
}
