use actix_web::{
    dev::ServiceRequest,
    post,
    web::{self, Json},
    HttpRequest, HttpResponse, Responder,
};
use chrono::Local;
use clerk_rs::{apis::users_api::User, validators::actix::clerk_authorize};
use serde::Deserialize;
use serde_json::json;
use std::fs;
use tracing::info;

use crate::app_state::AppState;
use crate::services::tts_service::call_openai_tts;
use crate::utils::convert_to_mp4::convert_to_mp4;


#[derive(Deserialize)]
pub struct UserInput {
    pub input: String,
}

#[post("/video")]
pub async fn get_video(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: Json<UserInput>,
) -> impl Responder {
    info!("POST /video endpoint called");

    let srv_req = ServiceRequest::from_request(req);
    let jwt = match clerk_authorize(&srv_req, &state.client, true).await {
        Ok(value) => value.1,
        Err(e) => return e,
    };

    let Ok(user) = User::get_user(&state.client, &jwt.sub).await else {
        return HttpResponse::InternalServerError().json(json!({
            "message": "Unable to retrieve user",
        }));
    };

    let user_first_name = user
        .first_name
        .clone()
        .unwrap_or(Some("User".to_string()))
        .unwrap_or("User".to_string());

    let text_to_speak = if payload.input.trim().is_empty() {
        format!("Hello, {}! This is a default TTS message.", user_first_name)
    } else {
        payload.input.trim().to_owned()
    };

    if text_to_speak.is_empty() {
        let err = json!({ "error": "No text provided." });
        return HttpResponse::BadRequest().json(err);
    }

    let user_id = &jwt.sub;
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
