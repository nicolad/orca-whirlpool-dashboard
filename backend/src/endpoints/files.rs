use actix_web::{
    dev::ServiceRequest,
    get,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse, Responder,
};
use chrono::NaiveDateTime;
use clerk_rs::{apis::users_api::User, validators::actix::clerk_authorize};
use crate::utils::convert_to_mp4::convert_to_mp4;
use serde::Serialize;
use std::fs;

use crate::app_state::AppState;

/// Holds data for each discovered `final.mp3`.
#[derive(Debug)]
struct FinalFile {
    timestamp: NaiveDateTime,
    file_path: String,
    dir_name: String,
}

/// Response struct so we can serialize the timestamp back to a string.
#[derive(Serialize)]
struct FinalFileResponse {
    timestamp: String,
    file_path: String,
    dir_name: String,
}

/// GET /speech/files
/// Returns all final.mp3 files for the current user, sorted by the timestamp
/// embedded in the folder name (e.g. "2025-04-03-14:03").
#[get("/files")]
async fn list_speech_files(state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    // 1) Authorize via Clerk
    let srv_req = ServiceRequest::from_request(req);
    let jwt = match clerk_authorize(&srv_req, &state.client, true).await {
        Ok(value) => value.1,
        Err(e) => return e,
    };

    // 2) Build path: user_files/<user_id>
    let user_id = &jwt.sub;
    let user_dir_path = format!("user_files/{}", user_id);

    // 3) Read top-level directory for this user
    let read_dir = match fs::read_dir(&user_dir_path) {
        Ok(d) => d,
        Err(_) => {
            // If directory doesn't exist or can't be read, return an empty JSON array
            let empty: Vec<FinalFileResponse> = Vec::new();
            return HttpResponse::Ok().json(empty);
        }
    };

    let mut final_files: Vec<FinalFile> = Vec::new();

    for entry in read_dir.flatten() {
        // Each entry is expected to be a timestamp-based folder, e.g. "2025-04-03-14:03"
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue, // skip unreadable entries
        };
        if meta.is_dir() {
            let dir_name = match entry.file_name().into_string() {
                Ok(s) => s,
                Err(_) => continue, // skip non-UTF8
            };

            // 4) Construct path to final.mp3
            let final_mp3_path = format!("{}/{}/final.mp3", user_dir_path, dir_name);
            if fs::metadata(&final_mp3_path).is_err() {
                // If final.mp3 doesn't exist, skip
                continue;
            }

            // 5) Parse the directory name as a NaiveDateTime
            let maybe_timestamp = NaiveDateTime::parse_from_str(&dir_name, "%Y-%m-%d-%H:%M");
            if let Ok(dt) = maybe_timestamp {
                final_files.push(FinalFile {
                    timestamp: dt,
                    file_path: final_mp3_path,
                    dir_name,
                });
            }
        }
    }

    // 6) Sort by timestamp ascending (oldest first).
    //    For newest first, use `final_files.sort_by_key(|f| std::cmp::Reverse(f.timestamp));`
    final_files.sort_by_key(|f| f.timestamp);

    // 7) Build response with human-readable date/time
    let result: Vec<FinalFileResponse> = final_files
        .iter()
        .map(|file| FinalFileResponse {
            timestamp: file.timestamp.format("%Y-%m-%d-%H:%M").to_string(),
            file_path: file.file_path.clone(),
            dir_name: file.dir_name.clone(),
        })
        .collect();

    HttpResponse::Ok().json(result)
}

/// GET /files/{dir_name}/mp4
/// Converts `final.mp3` to `final.mp4` inside the specified directory and
/// returns the MP4 bytes. If the MP4 already exists it is reused.
#[get("/files/{dir_name}/mp4")]
async fn mp4_for_file(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    let srv_req = ServiceRequest::from_request(req);
    let jwt = match clerk_authorize(&srv_req, &state.client, true).await {
        Ok(value) => value.1,
        Err(e) => return e,
    };

    let user_id = &jwt.sub;
    let dir_name = path.into_inner();
    // Build paths under user_files
    let folder_path = format!("user_files/{}/{}", user_id, dir_name);
    let final_mp3_path = format!("{}/final.mp3", folder_path);
    let final_mp4_path = format!("{}/final.mp4", folder_path);

    if fs::metadata(&final_mp3_path).is_err() {
        return HttpResponse::NotFound()
            .json(serde_json::json!({ "error": "final.mp3 not found" }));
    }

    if fs::metadata(&final_mp4_path).is_err() {
        if let Err(e) = convert_to_mp4(&final_mp3_path, &final_mp4_path) {
            return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e }));
        }
    }

    let video_bytes = match fs::read(&final_mp4_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": format!("Failed to read mp4: {e}") }));
        }
    };

    HttpResponse::Ok()
        .content_type("video/mp4")
        .body(video_bytes)
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(list_speech_files).service(mp4_for_file);
}
