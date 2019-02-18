extern crate qrcode;
extern crate image;
extern crate piston_window;
extern crate rand;

pub mod lt_code;

use piston_window::*;
use piston_window::texture::*;

use qrcode::QrCode;
use qrcode::EcLevel;
use qrcode::Version;

use image::Rgba;
use image::ImageBuffer;

use std::thread;

use std::fs::File;
use std::io::Read;

use std::env;

fn main() {
    let mut c = 1.0;
    let mut delta = 0.01;
    let mut fullscreen = false;
    let mut combined_packets = 14;
    let mut location = String::new();

    for (idx, arg) in env::args().enumerate() {
        if idx == 0 {
            continue;
        }
        if arg.to_lowercase().eq(&String::from("fullscreen")) {
            fullscreen = true;
        } else {
            let split: Vec<&str> = (arg.as_ref() as &str).split("=").collect();
            if split.len() == 2 {
                let mut value = 0.0;

                //Get Argument Parameter Value
                match split.get(1) {
                    None => {
                        continue;
                    },
                    Some(s) => {
                        match s.parse::<f64>() {
                            Ok(n) => {
                                value = n;
                            },
                            Err(_) => {continue;}
                        }
                    },
                }
                //Get Parameter Being Changed
                match split.get(0) {
                    None => {
                        continue;
                    },
                    Some(s) => {
                        match s {
                            &"c" => {
                                c = value;
                            },
                            &"delta" => {
                                delta = value;
                            },
                            &"packets" => {
                                combined_packets = (value as u32);
                            },
                            _ => {continue;}
                        }
                    },
                }
            } else {
                //Argument is file location
                location = arg;
            }
        }

    }

    println!("c: {}", c);
    println!("delta: {}", delta);
    println!("packets: {}", combined_packets);
    println!("fullscreen: {}", fullscreen);
    println!("location: {}", location);

    let mut window: PistonWindow = WindowSettings::new("Renderer", [400, 400])
                                        .exit_on_esc(true)
                                        .fullscreen(fullscreen)
                                        .build()
                                        .unwrap();

    let factory = &mut window.factory.clone();


//    let message = String::from("Hello World I am a file!");
    let mut file = File::open(location).unwrap();
    let mut message: Vec<u8> = Vec::new();
    file.read_to_end(&mut message).unwrap();


    let size = (message.as_ref() as &[u8]).len();
    let mut fountain = lt_code::Fountain::new( message.as_ref(), size as u32, c,delta);

    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {
            thread::sleep_ms(60);
            let mut combined_bytes: Vec<u8> = Vec::new();

            //Add multiple packets to qr code for higher information density per QR scan
            for _ in 0..combined_packets {
                let bytes: &[u8] = &lt_code::Fountain::serialize_packet(&fountain.generate_packet());
                for byte in bytes {
                    combined_bytes.push(*byte);
                }
            }

            let code = QrCode::with_error_correction_level(combined_bytes.as_ref() as &[u8], EcLevel::L).unwrap();
            let qr_image: ImageBuffer<Rgba<u8>, Vec<u8>> = code.render::<Rgba<u8>>().build();
//            let qr_image: ImageBuffer<Rgba<u8>, Vec<u8>> = image::imageops::flip_horizontal(&qr_image);

            let (width, height) = qr_image.dimensions();
            let texture: G2dTexture = CreateTexture::create(factory, Format::Rgba8,
                                                            &qr_image, [width, height], &TextureSettings::new()).unwrap();
            image(&texture, c.transform, g);
        });
    }
}