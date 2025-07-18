use std::path::{Path, PathBuf};
use std::process::Command;

/// Convert an MP3 file to MP4 using `ffmpeg`.
/// Returns `Ok(())` on success, otherwise an error message.
///
/// The function prints detailed progress messages to `stderr` so you can
/// trace every step of the conversion pipeline.
pub fn convert_to_mp4(input: &str, output: &str) -> Result<(), String> {
    eprintln!("🔧 convert_to_mp4() called");
    eprintln!("  ▶ input  file : {}", input);
    eprintln!("  ▶ output file : {}", output);

    // Resolve the overlay image that will be used as a static cover.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    eprintln!("  ▶ CARGO_MANIFEST_DIR: {}", manifest_dir);

    let img: PathBuf = Path::new(manifest_dir).join("assets").join("wma.png");
    eprintln!("  ▶ overlay image candidate: {}", img.display());

    // Check that the overlay file exists before invoking ffmpeg.
    if !img.exists() {
        eprintln!("  ❌ overlay image not found");
        return Err(format!("overlay image not found at {}", img.display()));
    }
    eprintln!("  ✅ overlay image found");

    // Build the full ffmpeg command for debugging visibility.
    let ffmpeg_args = [
        "-y", // overwrite output without asking
        "-loop",
        "1", // loop the static image forever
        "-i",
        img.to_str()
            .ok_or_else(|| "invalid overlay path".to_string())?, // image input
        "-i",
        input,       // audio input
        "-shortest", // stop when shortest input ends (the audio)
        "-c:v",
        "libx264", // video codec
        "-c:a",
        "aac", // audio codec
        "-b:a",
        "192k", // audio bitrate
        "-pix_fmt",
        "yuv420p", // pixel format
        output,
    ];

    eprintln!("  ▶ spawning ffmpeg with arguments:");
    for (i, arg) in ffmpeg_args.iter().enumerate() {
        eprintln!("      [{}] {}", i, arg);
    }

    let status = Command::new("ffmpeg")
        .args(&ffmpeg_args)
        .status()
        .map_err(|e| {
            eprintln!("  ❌ failed to spawn ffmpeg: {e}");
            format!("failed to spawn ffmpeg: {e}")
        })?;

    eprintln!("  ▶ ffmpeg exited with status: {}", status);

    if !status.success() {
        eprintln!("  ❌ ffmpeg reported failure");
        return Err(format!("ffmpeg exited with status {status}"));
    }

    eprintln!("  ✅ conversion completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::sync::Mutex;

    static LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn overlay_missing_returns_error() {
        let _guard = LOCK.lock().unwrap();

        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let overlay = Path::new(manifest_dir).join("assets").join("wma.png");
        let backup = overlay.with_extension("bak");

        fs::rename(&overlay, &backup).expect("rename overlay to backup");

        let result = convert_to_mp4("dummy.mp3", "dummy.mp4");

        fs::rename(&backup, &overlay).expect("restore overlay from backup");

        let err = result.expect_err("expected error when overlay missing");
        assert!(err.contains("overlay image not found"));
    }

    #[test]
    fn overlay_present_reaches_ffmpeg() {
        let _guard = LOCK.lock().unwrap();

        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let overlay = Path::new(manifest_dir).join("assets").join("wma.png");
        assert!(overlay.exists(), "overlay image should exist for test");

        let temp_dir = std::env::temp_dir();
        let input = temp_dir.join("dummy_input.mp3");
        let output = temp_dir.join("dummy_output.mp4");

        fs::write(&input, b"dummy").unwrap();

        let result = convert_to_mp4(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
        );

        let _ = fs::remove_file(&input);
        let _ = fs::remove_file(&output);

        let err = result.expect_err("expected failure as ffmpeg likely missing");
        assert!(
            !err.contains("overlay image not found"),
            "error should not be about missing overlay"
        );
    }
}
