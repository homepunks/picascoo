use anyhow::{Result, Context};
use picascoo::{Cmd, process_image, process_video};
use std::{env::args, fs::File, io::Read};

fn main() -> Result<()> {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <image/video_path> [output_width]", args[0]);
        return Ok(());
    }

    let cmd = Cmd {
        path: &args[1],
        width: args.get(2).and_then(|w| w.parse().ok()).unwrap_or(100),
        invert: args.iter().any(|a| a == "--invert"),
    };

    let mut file = File::open(cmd.path).with_context(|| format!("Could not open {}", cmd.path))?;
    let mut header = [0u8; 16];
    file.read_exact(&mut header)?;

    match (infer::is_image(&header), infer::is_video(&header)) {
        (true, false) => process_image(cmd)?,
        (false, true) => process_video(cmd)?,
        _ => anyhow::bail!("Unsupported file format"),
    }

    Ok(())
}
