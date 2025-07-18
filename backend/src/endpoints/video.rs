use actix_web::{
    post,
    web::Json,
    HttpResponse, Responder,
};
use chrono::Local;
use serde::Deserialize;
use serde_json::json;
use std::fs;
use tracing::info;

use crate::services::tts_service::call_openai_tts;
use crate::utils::convert_to_mp4::convert_to_mp4;


#[derive(Deserialize)]
pub struct UserInput {
    pub input: String,
}

#[post("/video")]
pub async fn get_video(
    payload: Json<UserInput>,
) -> impl Responder {
    info!("POST /video endpoint called");

    // Authentication removed
    let user_first_name = "User";

    let text_to_speak = if payload.input.trim().is_empty() {
        format!("Hello, {}! This is a default TTS message.", user_first_name)
    } else {
        payload.input.trim().to_owned()
    };

    if text_to_speak.is_empty() {
        let err = json!({ "error": "No text provided." });
        return HttpResponse::BadRequest().json(err);
    }

    let user_id = "public";
    let now = Local::now();
    let timestamp = now.format("%Y-%m-%d-%H:%M").to_string();
    let folder_path = format!("user_files/{}/{}", user_id, &timestamp);

    if let Err(e) = fs::create_dir_all(&folder_path) {
        let err = json!({ "error": format!("Failed to create directory {folder_path}: {e}") });
        return HttpResponse::InternalServerError().json(err);
    }

    let final_mp3_path = format!("{}/{}.mp3", folder_path, "final");
    let voice = "onyx".to_string();
    let tts_result = call_openai_tts(&text_to_speak, &voice).await;
    match tts_result {
        Ok(bytes) => {
            if let Err(e) = fs::write(&final_mp3_path, &bytes) {
                let err = json!({ "error": format!("Failed to write {final_mp3_path}: {e}") });
                return HttpResponse::InternalServerError().json(err);
            }
        }
        Err(e) => {
            let err = json!({ "error": format!("TTS error: {e}") });
            return HttpResponse::InternalServerError().json(err);
        }
    }

    let final_mp4_path = format!("{}/{}.mp4", folder_path, "final");
    if let Err(e) = convert_to_mp4(&final_mp3_path, &final_mp4_path) {
        let err = json!({ "error": e });
        return HttpResponse::InternalServerError().json(err);
    }

    let video_bytes = match fs::read(&final_mp4_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            let err = json!({ "error": format!("Failed to read mp4: {e}") });
            return HttpResponse::InternalServerError().json(err);
        }
    };

    HttpResponse::Ok()
        .content_type("video/mp4")
        .body(video_bytes)
}
