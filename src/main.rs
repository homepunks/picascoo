use anyhow::Result;
use picascoo::{Cmd, process_image, process_video};
use std::{env::args, fs};

fn main() -> Result<()> {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <image/video_path> [output_width]", args[0]);
        return Ok(());
    }

    let cmd = Cmd {
        path: &args[1],
        width: args.get(2).and_then(|w| w.parse().ok()).unwrap_or(100),
    };

    let file = fs::read(cmd.path).unwrap();
    match (infer::is_image(&file), infer::is_video(&file)) {
        (true, false) => process_image(cmd)?,
        (false, true) => process_video(cmd)?,
        _ => anyhow::bail!("Unsupported file format"),
    }

    Ok(())
}
