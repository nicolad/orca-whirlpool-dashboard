use reqwest::Client;
use serde::Serialize;

#[derive(Serialize)]
struct TtsRequest {
    model: String,
    input: String,
    voice: String,
}

pub async fn call_openai_tts(input_text: &str, voice: &str) -> Result<Vec<u8>, String> {
    let client = Client::new();

    // 1) Read API key
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "Missing OPENAI_API_KEY environment variable".to_string())?;

    // 2) Prepare request payload
    let body = TtsRequest {
        model: "tts-1".to_string(),
        input: input_text.to_string(),
        voice: voice.to_string(),
    };

    // 3) Make up to 2 attempts total
    for attempt in 1..=2 {
        // Make the HTTP request
        let resp = match client
            .post("https://api.openai.com/v1/audio/speech")
            .bearer_auth(&api_key)
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                let err_msg = format!("Request error (attempt #{attempt}): {e}");
                // If this is the second attempt, return the error; else retry.
                if attempt == 2 {
                    return Err(err_msg);
                } else {
                    eprintln!("{err_msg} — retrying...");
                    continue;
                }
            }
        };

        // If the response is not success, read the text to see the error
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            let err_msg = format!("TTS request failed (attempt #{attempt}): {status} - {text}");

            if attempt == 2 {
                // Return the error if it fails again
                return Err(err_msg);
            } else {
                eprintln!("{err_msg} — retrying...");
                continue;
            }
        }

        // If we get a success status, attempt to parse the bytes
        match resp.bytes().await {
            Ok(bytes) => {
                // Return the MP3 bytes on success
                return Ok(bytes.to_vec());
            }
            Err(e) => {
                let err_msg =
                    format!("Unable to read TTS response bytes (attempt #{attempt}): {e}");
                if attempt == 2 {
                    return Err(err_msg);
                } else {
                    eprintln!("{err_msg} — retrying...");
                }
            }
        }
    }

    // Fallback if something unexpected happens outside the loop
    Err("Unexpected error: call_openai_tts ran out of attempts".to_string())
}
