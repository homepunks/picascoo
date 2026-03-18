use anyhow::Result;
use picascoo::{process_image, process_video};
use std::{env::args, fs};

fn main() -> Result<()> {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <image/video_path> [output_width]", args[0]);
        return Ok(());
    }

    let path = &args[1];
    let width = args.get(2).and_then(|w| w.parse().ok()).unwrap_or(100);

    let file = fs::read(path).unwrap();
    match (infer::is_image(&file), infer::is_video(&file)) {
        (true, false) => process_image(path, width)?,
        (false, true) => process_video(path, width)?,
        _ => anyhow::bail!("Unsupported file format"),
    }

    Ok(())
}
