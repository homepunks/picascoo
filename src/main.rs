use image::{GenericImageView, Pixel};
use colored::*;

const ASCII_CHARS: &[char] = &['@', '#', '$', '%', '?', '*', '+', ';', ':', ',', '.'];

fn main() {
    let img_path = "./1209.jpg";
    let img = image::open(img_path).expect("[ERROR] Failed to open the image...");
    let (width, height) = img.dimensions();

    let new_width = 175;
    let new_height = (height as f32 / width as f32 * new_width as f32 * 0.5) as u32;
    let resized_img = img.resize_exact(new_width, new_height, image::imageops::FilterType::Nearest);
    
    for y in 0..new_height {
        for x in 0..new_width {
            let pixel = resized_img.get_pixel(x, y);
            let brightness = pixel.to_luma()[0] as f32 / 255.0;
            let ascii_char = ASCII_CHARS[(brightness * (ASCII_CHARS.len() - 1) as f32) as usize];
        
            let rgb = pixel.to_rgb();
            print!("{}", ascii_char.to_string().truecolor(rgb[0], rgb[1], rgb[2]));
	}
	println!();
    }
}

