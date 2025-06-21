use anyhow::{Context, Result};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    queue,
    style::{Print, ResetColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use ffmpeg_next::{format, frame, media};
use image::{GenericImageView, Pixel};
use std::{
    env,
    io::{stdout, Write},
    time::Instant,
};

const ASCII_CHARS: &[char] = &['@', '#', '$', '%', '?', '*', '+', ';', ':', ',', '.'];

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <image/video_path> [output_width]", args[0]);
        return Ok(());
    }
>>>>>>> 36d9d20 (Add video uploading functionality)

    let path = &args[1];
    let width = args.get(2).and_then(|w| w.parse().ok()).unwrap_or(100);

    if let Some(ext) = std::path::Path::new(path).extension() {
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

fn process_image(path: &str, width: u32) -> Result<()> {
    let img = image::open(path).context("Failed to open image")?;
    let ascii = image_to_ascii(&img, width);
    print!("{ascii}");

    print!("\x1b[0m");
    Ok(())
}

fn process_video(path: &str, width: u32) -> Result<()> {
    ffmpeg_next::init().context("FFmpeg init failed")?;

    let mut ictx = format::input(&path).context("Couldn't open input file")?;
    let input = ictx
        .streams()
        .best(media::Type::Video)
        .ok_or(anyhow::anyhow!("No video stream found"))?;
    let stream_index = input.index();

    let context = ffmpeg_next::codec::context::Context::from_parameters(input.parameters())
        .context("Couldn't create codec context")?;
    let mut decoder = context.decoder().video().context("Video decoder error")?;

    let mut converter = decoder.converter(ffmpeg_next::format::Pixel::RGB24)?;
    
    let fps = {
        let rate = input.avg_frame_rate();
        if rate.denominator() == 0 {
            30.0
        } else {
            rate.numerator() as f64 / rate.denominator() as f64
        }
    };
    let frame_delay = 1.0 / fps;

    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    queue!(stdout, EnterAlternateScreen, Hide)?;

    let mut frame_count = 0;
    let start_time = Instant::now();

    let (term_width, _term_height) = terminal::size()?;
    let max_width = std::cmp::min(width, term_width as u32);
    let mut prev_height = 0;

    for (stream, packet) in ictx.packets() {
        if stream.index() != stream_index {
            continue;
        }

        decoder.send_packet(&packet)?;
        let mut decoded = frame::Video::empty();
        while decoder.receive_frame(&mut decoded).is_ok() {
            let elapsed = start_time.elapsed().as_secs_f64();
            let target_time = frame_count as f64 * frame_delay;
            if elapsed < target_time {
                std::thread::sleep(std::time::Duration::from_secs_f64(
                    target_time - elapsed,
                ));
            }

            let mut rgb_frame = frame::Video::empty();
            converter.run(&decoded, &mut rgb_frame)?;

            let img = image::RgbImage::from_raw(
                rgb_frame.width() as u32,
                rgb_frame.height() as u32,
                rgb_frame.data(0).to_vec(),
            )
            .context("Failed to create image from frame")?;

            let ascii = image_to_ascii(
                &image::DynamicImage::ImageRgb8(img),
                max_width,
            );
            
            let lines: Vec<&str> = ascii.lines().collect();
            let current_height = lines.len() as u16;
            
            queue!(
                stdout,
                MoveTo(0, 0),
            )?;
            
            if current_height > prev_height {
                for y in 0..prev_height {
                    queue!(stdout, MoveTo(0, y), Clear(ClearType::CurrentLine))?;
                }
            }
            prev_height = current_height;
            
            for (y, line) in lines.iter().enumerate() {
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
    }

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
    let resized_img = img.resize_exact(
        new_width,
        new_height,
        image::imageops::FilterType::Nearest,
    );

    let mut ascii_art = String::with_capacity((new_width * new_height) as usize);
    for y in 0..new_height {
        for x in 0..new_width {
            let pixel = resized_img.get_pixel(x, y);
            let brightness = pixel.to_luma()[0] as f32 / 255.0;
            let char_index = (brightness * (ASCII_CHARS.len() - 1) as f32) as usize;
            let ascii_char = ASCII_CHARS[char_index];

            let rgb = pixel.to_rgb();
            ascii_art.push_str(
                &format!(
                    "\x1b[38;2;{};{};{}m{}",
                    rgb[0], rgb[1], rgb[2], ascii_char
                )
            );
        }
        ascii_art.push_str("\x1b[0m");
        ascii_art.push('\n');
    }
    ascii_art
}
