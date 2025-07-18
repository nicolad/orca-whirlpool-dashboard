use actix_files::Files;
use actix_web::{
    web::{self, ServiceConfig},
};

use endpoints::files::configure as files_configure;
use endpoints::speech::get_speech;
use endpoints::video::get_video;

use shuttle_actix_web::ShuttleActixWeb;
use shuttle_runtime::SecretStore;
mod app_state;
mod endpoints;
mod services;
mod utils;


#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let app_config = move |cfg: &mut ServiceConfig| {
        let open_api_key = secrets
            .get("OPENAI_API_KEY")
            .expect("OpenAI API key is not set");

        std::env::set_var("OPENAI_API_KEY", open_api_key);

        // Authentication removed

        // Create `./user_files` so that Actix won't throw an error.
        // If it already exists, `create_dir_all` does nothing.
        if let Err(e) = std::fs::create_dir_all("./user_files") {
            tracing::error!("Failed to create user_files directory: {:?}", e);
            // Or handle the error however you'd like
        }

        let state = web::Data::new(app_state::AppState);

        cfg.service(
            Files::new("/user_files", "./user_files")
                .prefer_utf8(true)
                .use_last_modified(true)
                .show_files_listing(), // Optional for debugging (exposes file listing)
        );

        cfg.service(
            web::scope("/api")
                .service(get_speech)
                .service(get_video)
                .configure(files_configure),
        )
        // serve the build files from the frontend
        .service(actix_files::Files::new("/", "./frontend/dist").index_file("index.html"))
        .app_data(state);
    };

    Ok(app_config.into())
}

