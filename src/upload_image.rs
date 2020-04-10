use futures::{
    TryFutureExt,
    TryStreamExt,
    stream::FuturesUnordered,
};
use bytes::BufMut;
use warp::{ Filter, http::StatusCode, };
use warp::reject::custom as reject;
use serde::{Deserialize, Serialize};
use tokio::fs;
use crate::{
    Errors,
    filter,
    links,
    with_params,
    AppParameters,
};
use base64;
use std::{
    mem,
    convert::TryInto,
    ptr::NonNull,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    slice::from_raw_parts,
};
use core::ffi::c_void;


#[derive(Deserialize, Debug)]
struct Base64JsonImage {
    filename: Option<String>,
    data: String,
}

#[derive(Debug)]
struct Image {
    //filename: Option<String>,
    data: Vec<u8>,
}

pub fn filter (
    app: AppParameters,
) -> filter!(impl warp::Reply) {
    warp::post()
        .and(warp::path("upload_image"))
        .and(with_params(app))
        .and(
            formdata()
            .or(base64json())
            .unify()
        )
        .and_then(upload_image)
}

fn base64json() -> filter!((Vec<Image>,)) {
    warp::body::json::<Vec<Base64JsonImage>>()
        .and_then(|images: Vec<Base64JsonImage>| async {
            let a: Result<Vec<Image>, warp::Rejection> =
                images.into_iter().map(move |image| {
                    Ok(Image {
                    //filename: image.filename,
                    data: base64::decode(image.data)
                        .map_err(|_| {reject(Errors::Base64Decoding)})?
                })}).collect();
            if let Ok(images) = &a {
                if images.is_empty() {
                    Err(reject(Errors::Base64Decoding))
                } else {a}
            } else { a }
        })
}

fn formdata() -> filter!((Vec<Image>,)) {
    warp::multipart::form()
        .max_length(1024 * 1024 * 10) // in bytes
        .and_then(|form: warp::multipart::FormData| { async {
            let part: Result<Vec<Image>, warp::Rejection> =
                form.and_then(|part| {
                    /*
                    let name = (&part).name().to_string();
                    println!("name: {:?}", &name);

                    let fname = part.filename().map(|x| x.to_string());
                    println!("fname: {:?}", fname);
                    */

                    let value = part.stream().try_fold(Vec::new(), |mut vec, data| {
                        vec.put(data);
                        async move { Ok(vec) }
                    });

                    value.map_ok(move |vec| Image {
                        //filename: fname,
                        data: vec,
                    })
                })
                .try_collect()
                .await
                .map_err(|_| {
                    reject(Errors::Multipart)
                });
            part
        }})
}

#[derive(Debug, Serialize)]
pub struct UploadImageReply {
    pub code: u16,
    pub ids: Vec<String>,
}

async fn upload_image (
    app: AppParameters,
    images: Vec<Image>,
) -> Result<impl warp::Reply, warp::Rejection> {

    let res = images.into_iter().map(|image| async {
        let (original, thumbnail, hash) = unsafe {
            let mut w: i32 = 0;
            let mut h: i32 = 0;
            let mut n: i32 = 0;

            let original = NonNull::new({
                let (ptr, len, cap) = image.data.into_raw_parts();
                let original = links::stbi_load_from_memory(
                    ptr, len.try_into().unwrap(), &mut w, &mut h, &mut n, 0
                );

                Vec::from_raw_parts(ptr, len, cap);

                original
            }).ok_or(reject(Errors::ImageDecoding))?;

            let mut hasher = DefaultHasher::new();
            Hash::hash_slice(from_raw_parts(original.as_ptr(), (w * h * n) as usize), &mut hasher);

            let thumbnail_w: i32 = 100;
            let thumbnail_h: i32 = 100;

            let thumbnail = NonNull::new(
                libc::malloc(mem::size_of::<u8>() * (thumbnail_w * thumbnail_h * n) as usize) as *mut u8
            ).ok_or(reject(Errors::Internal))?;

            let (s0, t0, s1, t1) = if w > h {
                let shift = (w - h) as f32 / (w as f32) / 2.0;
                (shift, 0.0, 1.0 - shift, 1.0)
            } else {
                let shift = (h - w) as f32 / (h as f32) / 2.0;
                (0.0, shift, 1.0, 1.0 - shift)
            };

            if links::stbir_resize_region(
                original.as_ptr() as *const c_void, w, h, 0,
                thumbnail.as_ptr() as *mut c_void, thumbnail_w, thumbnail_h, 0,
                links::stbir_datatype_STBIR_TYPE_UINT8,
                n, -1, 0,
                links::stbir_edge_STBIR_EDGE_CLAMP, links::stbir_edge_STBIR_EDGE_CLAMP,
                links::stbir_filter_STBIR_FILTER_DEFAULT,  links::stbir_filter_STBIR_FILTER_DEFAULT,
                links::stbir_colorspace_STBIR_COLORSPACE_LINEAR, 0 as *mut c_void,
                s0, t0, s1, t1
            ) == 0 {
                libc::free(thumbnail.as_ptr() as *mut c_void);
                links::stbi_image_free(original.as_ptr() as *mut c_void);
                return Err(reject(Errors::Internal));
            }

            let mut original_size: i32 = 0;
            let mut thumbnail_size: i32 = 0;

            let original_png = NonNull::new(
                links::stbi_write_png_to_mem(original.as_ptr(), 0, w, h, n, &mut original_size)
            ).ok_or(reject(Errors::Internal))?;

            let thumbnail_png = NonNull::new(
                links::stbi_write_png_to_mem(thumbnail.as_ptr(), 0, thumbnail_w, thumbnail_h, n, &mut thumbnail_size)
            ).ok_or(reject(Errors::Internal))?;


            Box::from_raw(original.as_ptr());
            Box::from_raw(thumbnail.as_ptr());

            // Potentially UB here since original_png and thumbnail_png are from malloc() (from libc?),
            // not from Vec::into_raw_parts(), but it works
            (
                Vec::from_raw_parts(original_png.as_ptr(), original_size as usize, original_size as usize),
                Vec::from_raw_parts(thumbnail_png.as_ptr(), thumbnail_size as usize, thumbnail_size as usize),
                hasher.finish()
            )
        };

        fs::write(format!("{}/{:x}.png", app.storage_path, hash), original)
            .await
            .map_err(|_| reject(Errors::Database))?;

        fs::write(format!("{}/{:x}-thumbnail.png", app.storage_path, hash), thumbnail)
            .await
            .map_err(|_| reject(Errors::Database))?;

        Ok(format!("{:x}", hash))
    })
        .collect::<FuturesUnordered<_>>()
        .try_collect::<Vec<String>>()
        .await?;

    Ok(warp::reply::with_status(
        warp::reply::json(&UploadImageReply{
            code: StatusCode::CREATED.as_u16(),
            ids: res,
        }
    ), StatusCode::CREATED))
}
