use crate::GitConfig;
use actix_files::NamedFile;
use actix_web::cookie::time;
use actix_web::cookie::time::format_description;
use actix_web::http::header;
use actix_web::http::header::HeaderValue;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::collections::HashMap;

pub async fn objects_pack(
    request: HttpRequest,
    service: web::Data<impl GitConfig>,
) -> impl Responder {
    let uri = request.uri();
    let path = uri.path().to_string();
    let req_file = service.rewrite(path).await;
    let mut map = HashMap::new();
    let time = time::OffsetDateTime::now_utc();
    let expires = time::OffsetDateTime::now_utc() + time::Duration::days(1);
    map.insert(
        "Date".to_string(),
        time.format(&format_description::parse("%a, %d %b %Y %H:%M:%S GMT").unwrap())
            .unwrap(),
    );
    map.insert(
        "Expires".to_string(),
        expires
            .format(&format_description::parse("%a, %d %b %Y %H:%M:%S GMT").unwrap())
            .unwrap(),
    );
    map.insert(
        "Cache-Control".to_string(),
        "public, max-age=86400".to_string(),
    );
    #[allow(unused_assignments)]
    let mut xtype = "application/x-git-loose-object".to_string();
    if uri.to_string().ends_with(".pack") {
        xtype = "application/x-git-packed-objects".to_string();
    } else if uri.to_string().ends_with(".idx") {
        xtype = "application/x-git-packed-objects-toc".to_string();
    } else {
        xtype = "application/x-git-loose-object".to_string();
    }
    if !req_file.exists() {
        return HttpResponse::NotFound().body("File not found");
    }
    match NamedFile::open(req_file) {
        Ok(mut named_file) => {
            named_file = named_file.use_last_modified(true);
            let mut response = named_file.into_response(&request);
            for (k, v) in map.iter() {
                response.headers_mut().insert(
                    k.to_string().parse().unwrap(),
                    HeaderValue::from_str(v).unwrap(),
                );
            }

            response
                .headers_mut()
                .insert(header::CONTENT_TYPE, HeaderValue::from_str(&xtype).unwrap());
            response
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to open file"),
    }
}
