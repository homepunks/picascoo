use anyhow::Result;
use picascoo::{process_image, process_video};
use std::{env::args, path::Path};

fn main() -> Result<()> {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <image/video_path> [output_width]", args[0]);
        return Ok(());
    }

    let path = &args[1];
    let width = args.get(2).and_then(|w| w.parse().ok()).unwrap_or(100);

    if let Some(ext) = Path::new(path).extension() {
        match ext.to_str().unwrap() {
            "jpg" | "jpeg" | "png" => process_image(path, width)?,
            "mp4" | "avi" | "mov" => process_video(path, width)?,
            _ => anyhow::bail!("Unsupported file format"),
        }
    } else {
        anyhow::bail!("File has no extension");
    }

    Ok(())
}
