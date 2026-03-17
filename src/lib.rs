use anyhow::{Context, Result};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    queue,
    style::{Print, ResetColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use image::{GenericImageView, Pixel};
use std::{
    io::{self, Write, stdout},
    time::Instant,
    process,
};

const ASCII_CHARS: &[char] = &['@', '#', '$', '%', '?', '*', '+', ';', ':', ',', '.'];

pub fn process_image(path: &str, width: u32) -> Result<()> {
    let img = image::open(path).context("Failed to open image")?;
    let ascii = image_to_ascii(&img, width);
    print!("{ascii}");

    print!("\x1b[0m");
    Ok(())
}

pub fn process_video(path: &str, width: u32) -> Result<()> {
    let probe = process::Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=width,height,r_frame_rate",
            "-of", "csv=p=0",
            path,
        ])
        .output()
        .context("Failed to run `ffprobe` -- is ffmpeg installed?")?;

    let info = String::from_utf8_lossy(&probe.stdout);
    let parts: Vec<&str> = info.trim().split(',').collect();
    if parts.len() < 3 {
        anyhow::bail!("Could not parse video info from ffprobe");
    }

    let vid_width: u32 = parts[0].parse().context("Bad width")?;
    let vid_height: u32 = parts[1].parse().context("Bad height")?;
    let fps: f64 = {
        let rate_parts: Vec<&str> = parts[2].split('/').collect();
        let num: f64 = rate_parts[0].parse().unwrap_or(30.0);
        let den: f64 = rate_parts.get(1).and_then(|d| d.parse().ok()).unwrap_or(1.0);
        if den == 0.0 { 30.0 } else { num / den }
    };

    let (term_width, _) = terminal::size()?;
    let max_width = std::cmp::min(width, term_width as u32);
    let aspect_ratio = vid_height as f32 / vid_width as f32;
    let out_height = (max_width as f32 * aspect_ratio * 0.5) as u32;

    let mut child = process::Command::new("ffmpeg")
        .args([
            "-i", path,
            "-vf", &format!("scale={}:{}", max_width, out_height),
            "-pix_fmt", "rgb24",
            "-f", "rawvideo",
            "-v", "quiet",
            "-",
        ])
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::null())
        .spawn()
        .context("Failed to run `ffmpeg` -- is ffmpeg installed?")?;

    let pipe = child.stdout.take().context("No stdout from ffmpeg")?;
    let mut reader = io::BufReader::new(pipe);

    let frame_size = (max_width * out_height * 3) as usize;
    let frame_delay = 1.0 / fps;

    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    queue!(stdout, EnterAlternateScreen, Hide)?;

    let mut buf = vec![0u8; frame_size];
    let mut frame_count: u64 = 0;
    let start_time = Instant::now();

    use std::io::Read;
    loop {
        if event::poll(std::time::Duration::from_millis(0))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.code == KeyCode::Char('q') || key_event.code == KeyCode::Char('Q') {
                    break;
                }
            }
        }

        match reader.read_exact(&mut buf) {
            Ok(()) => {}
            Err(_) => break,
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let target_time = frame_count as f64 * frame_delay;
        if elapsed < target_time {
            std::thread::sleep(std::time::Duration::from_secs_f64(target_time - elapsed));
        }

        let img = image::RgbImage::from_raw(max_width, out_height, buf.clone())
            .context("Failed to create image from frame")?;
        let ascii = image_to_ascii(&image::DynamicImage::ImageRgb8(img), max_width);

        queue!(stdout, MoveTo(0, 0))?;
        for (y, line) in ascii.lines().enumerate() {
            queue!(
                stdout,
                MoveTo(0, y as u16),
                Clear(ClearType::CurrentLine),
                Print(line),
                Print("\r\n")
            )?;
        }
        stdout.flush()?;

        frame_count += 1;
    }

    let _ = child.kill();
    let _ = child.wait();

    queue!(
        stdout,
        ResetColor,
        MoveTo(0, 0),
        Clear(ClearType::All),
        LeaveAlternateScreen,
        Show
    )?;
    stdout.flush()?;
    terminal::disable_raw_mode()?;

    Ok(())
}

fn image_to_ascii(img: &image::DynamicImage, new_width: u32) -> String {
    let (width, height) = img.dimensions();
    let aspect_ratio = height as f32 / width as f32;
    let new_height = (new_width as f32 * aspect_ratio * 0.5) as u32;
    let resized_img = img.resize_exact(new_width, new_height, image::imageops::FilterType::Nearest);

    let mut ascii_art = String::with_capacity((new_width * new_height) as usize);
    for y in 0..new_height {
        for x in 0..new_width {
            let pixel = resized_img.get_pixel(x, y);
            let brightness = pixel.to_luma()[0] as f32 / 255.0;
            let char_index = (brightness * (ASCII_CHARS.len() - 1) as f32) as usize;
            let ascii_char = ASCII_CHARS[char_index];

            let rgb = pixel.to_rgb();
            ascii_art.push_str(&format!(
                "\x1b[38;2;{};{};{}m{}",
                rgb[0], rgb[1], rgb[2], ascii_char
            ));
        }
        ascii_art.push_str("\x1b[0m");
        ascii_art.push('\n');
    }
    ascii_art
}
