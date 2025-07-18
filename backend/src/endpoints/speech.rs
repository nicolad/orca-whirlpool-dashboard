use actix_web::{
    dev::ServiceRequest,
    post,
    web::{self, Json},
    HttpRequest, HttpResponse, Responder,
};
use chrono::Local;
use clerk_rs::{apis::users_api::User, validators::actix::clerk_authorize};
use futures::future::join_all;
use serde::Deserialize;
use serde_json::json;
use std::fs;
use tokio::task;
use tracing::info;

use crate::app_state::AppState;
use crate::services::tts_service::call_openai_tts;
use crate::utils::{chunk_text_unicode::chunk_text_unicode, concat_mp3::concat_mp3};

#[derive(Deserialize)]
pub struct UserInput {
    pub input: String,
}

#[post("/speech")]
pub async fn get_speech(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: Json<UserInput>,
) -> impl Responder {
    info!("POST /speech endpoint called");

    // 1) Authorize the request with Clerk
    let srv_req = ServiceRequest::from_request(req);
    info!("Attempting to authorize request with Clerk");

    let jwt = match clerk_authorize(&srv_req, &state.client, true).await {
        Ok(value) => {
            info!("clerk_authorize successful for user_id: {}", value.1.sub);
            value.1
        }
        Err(e) => {
            info!("clerk_authorize failed: {:?}", e);
            return e; // e is an actix_web::Error
        }
    };

    // 2) Optionally fetch the user
    info!("Fetching user data from Clerk for sub: {}", jwt.sub);
    let Ok(user) = User::get_user(&state.client, &jwt.sub).await else {
        info!("Unable to retrieve user data");
        return HttpResponse::InternalServerError().json(json!({
            "message": "Unable to retrieve user",
        }));
    };

    // 3) Obtain user's first name
    let user_first_name = user
        .first_name
        .clone()
        .unwrap_or(Some("User".to_string()))
        .unwrap_or("User".to_string());
    info!("User first_name is: {}", user_first_name);

    // 4) Prepare text for TTS
    info!("Preparing text for TTS");
    let text_to_speak = if payload.input.trim().is_empty() {
        info!("User provided empty input; using default message");
        format!("Hello, {}! This is a default TTS message.", user_first_name)
    } else {
        info!("User provided custom input");
        payload.input.trim().to_owned()
    };

    // 5) Chunk text at Unicode boundaries
    info!("Chunking text at Unicode boundaries");
    let chunks = chunk_text_unicode(&text_to_speak, 4096);
    info!("Number of chunks created: {}", chunks.len());

    if chunks.is_empty() {
        info!("No text provided after trimming, returning error");
        let err = json!({ "error": "No text provided." });
        return HttpResponse::BadRequest().json(err);
    }

    // 6) Create folder path: user_files/<user_id>/<timestamp>
    let user_id = &jwt.sub;
    let now = Local::now();
    let timestamp = now.format("%Y-%m-%d-%H:%M").to_string();
    let folder_path = format!("user_files/{}/{}", user_id, timestamp);
    info!("Creating directory: {}", folder_path);

    if let Err(e) = fs::create_dir_all(&folder_path) {
        info!("Failed to create directory {}: {:?}", folder_path, e);
        let err = json!({ "error": format!("Failed to create directory {folder_path}: {e}") });
        return HttpResponse::InternalServerError().json(err);
    }

    // 7) For each chunk, spawn a parallel TTS task
    info!("Spawning TTS tasks for each chunk");
    let mut tasks = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        let chunk_cloned = chunk.clone();
        let index = i + 1;
        let voice = "onyx".to_string(); // or whichever voice your TTS supports
        let chunk_filename = format!("{}/speech-chunk-{}.mp3", folder_path, index);

        info!(
            "Spawning TTS task #{}, output file: {}",
            index, chunk_filename
        );

        tasks.push(task::spawn(async move {
            info!("Task #{}: calling TTS API", index);
            let tts_result = call_openai_tts(&chunk_cloned, &voice).await;
            match tts_result {
                Ok(bytes) => {
                    info!(
                        "Task #{}: TTS API call succeeded, writing chunk to disk",
                        index
                    );
                    match fs::write(&chunk_filename, &bytes) {
                        Ok(_) => {
                            info!("Task #{}: successfully wrote {}", index, chunk_filename);
                            Ok(chunk_filename)
                        }
                        Err(e) => {
                            info!(
                                "Task #{}: failed to write {}: {:?}",
                                index, chunk_filename, e
                            );
                            Err(format!("Failed to write {chunk_filename}: {e}"))
                        }
                    }
                }
                Err(e) => {
                    info!("Task #{}: TTS API call failed: {}", index, e);
                    Err(format!("Chunk #{index} TTS error: {e}"))
                }
            }
        }));
    }

    // 8) Wait for all tasks to finish
    info!("Waiting for TTS tasks to finish...");
    let results = join_all(tasks).await;
    let mut saved_files = Vec::new();
    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(Ok(filename)) => {
                info!(
                    "TTS task #{} completed successfully, file: {}",
                    i + 1,
                    filename
                );
                saved_files.push(filename);
            }
            Ok(Err(e)) => {
                info!("TTS task #{} returned an error: {}", i + 1, e);
                let err = json!({ "error": format!("Task #{} error: {}", i+1, e) });
                return HttpResponse::InternalServerError().json(err);
            }
            Err(join_err) => {
                info!("TTS task #{} join error: {:?}", i + 1, join_err);
                let err = json!({ "error": format!("Join error on task #{}: {join_err}", i+1) });
                return HttpResponse::InternalServerError().json(err);
            }
        }
    }

    // 9) Merge the chunk MP3 files into one final MP3
    let final_mp3_path = format!("{}/{}.mp3", folder_path, "final");
    info!(
        "Merging {} chunk MP3 files into {}",
        saved_files.len(),
        final_mp3_path
    );
    let saved_files_ref: Vec<&str> = saved_files.iter().map(|s| s.as_str()).collect();
    if let Err(e) = concat_mp3(&saved_files_ref, &final_mp3_path) {
        info!("Failed to merge MP3 files: {:?}", e);
        let err = json!({ "error": format!("Failed to merge mp3: {e}") });
        return HttpResponse::InternalServerError().json(err);
    }

    // 10) Read the merged MP3 into memory to return
    info!("Reading merged MP3 file from disk: {}", final_mp3_path);
    let merged_file_bytes = match fs::read(&final_mp3_path) {
        Ok(bytes) => {
            info!("Successfully read merged MP3 file");
            bytes
        }
        Err(e) => {
            info!("Failed to read merged MP3 file: {:?}", e);
            let err = json!({ "error": format!("Failed to read merged mp3: {e}") });
            return HttpResponse::InternalServerError().json(err);
        }
    };

    // 11) Return the combined audio as an MP3
    info!("Returning merged MP3 file in response");
    HttpResponse::Ok()
        .content_type("audio/mpeg")
        .body(merged_file_bytes)
}
