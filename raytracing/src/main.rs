mod primitives;
mod raytrace;

use clap::{App, Arg};
use image::RgbaImage;
use indicatif::{ProgressBar, ProgressStyle};
use minifb::{Key, Window, WindowOptions};
use nalgebra::Vector3;
use primitives::SphereBuilder;
use rand::{seq::SliceRandom, thread_rng};
use raytrace::{raycast, Scene};
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::Instant;

fn to_argb_u32(rgba: [u8; 4]) -> u32 {
    let (r, g, b, a) = (
        rgba[0] as u32,
        rgba[1] as u32,
        rgba[2] as u32,
        rgba[3] as u32,
    );
    a << 24 | r << 16 | g << 8 | b
}

fn to_xy(scene: &Scene, index: u32) -> (f32, f32) {
    let (w, h) = (scene.width as f32, scene.height as f32);
    let (x, y) = ((index % scene.width) as f32, (index / scene.width) as f32);
    let aspect = w / h;
    let fov = (scene.fov.to_radians() / 2.0).tan();
    let x = (((x + 0.5) / w) * 2.0 - 1.0) * fov;
    let y = (1.0 - ((y + 0.5) / h) * 2.0) * fov;

    if scene.width < scene.height {
        return (x * aspect, y);
    }

    (x, y / aspect)
}

fn raytrace_fb(scene: Scene, buffer_mutex: &Arc<Mutex<Vec<u32>>>, bar: Option<ProgressBar>) {
    let buffer_mutex = Arc::clone(&buffer_mutex);
    let mut indexes: Vec<u32> = (0..scene.width * scene.height).collect();
    indexes.shuffle(&mut thread_rng());

    println!("Raytracing...");
    spawn(move || {
        for index in indexes.iter() {
            if let Some(bar) = &bar {
                bar.inc(1);
            }

            let (x, y) = to_xy(&scene, *index);
            let color = raycast(&scene, x, y);
            let index = *index as usize;
            let mut buffer = buffer_mutex.lock().unwrap();
            buffer[index] = to_argb_u32(color);
            drop(buffer);
        }

        if let Some(bar) = bar {
            bar.finish_and_clear();
        }

        println!("Done.");
    });
}

fn raytrace(scene: Scene, image_buffer: &mut Vec<u8>, bar: Option<ProgressBar>) {
    for index in 0..scene.width * scene.height {
        if let Some(bar) = &bar {
            bar.inc(1);
        }

        let (x, y) = to_xy(&scene, index);
        let color = raycast(&scene, x, y);

        let index = (index * 4) as usize;
        image_buffer[index] = color[0];
        image_buffer[index + 1] = color[1];
        image_buffer[index + 2] = color[2];
        image_buffer[index + 3] = color[3];
    }

    if let Some(bar) = bar {
        bar.finish();
    }
}

fn main() {
    let matches = App::new("raytracer")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .takes_value(true)
                .help("Output raytracer image to file"),
        )
        .arg(
            Arg::with_name("progress")
                .long("progress")
                .help("Show progress bar"),
        )
        .get_matches();

    let output_filename = matches.value_of("file");
    let show_progress = matches.is_present("progress");

    let mut scene: Scene = Scene {
        width: 800,
        height: 800,
        fov: 65.0,
        objects: Vec::new(),
    };
    let (width, height) = (scene.width, scene.height);
    scene.objects.push(Box::new(
        SphereBuilder::default()
            .center(Vector3::from([0.0, 0.0, -5.0]))
            .build()
            .unwrap(),
    ));
    scene.objects.push(Box::new(
        SphereBuilder::default()
            .radius(3.0)
            .center(Vector3::from([3.0, -2.0, -4.0]))
            .color([0, 64, 127, 255])
            .build()
            .unwrap(),
    ));
    scene.objects.push(Box::new(
        SphereBuilder::default()
            .radius(7.0)
            .center(Vector3::from([-6.0, 6.0, -18.0]))
            .color([255, 60, 30, 255])
            .build()
            .unwrap(),
    ));
    scene.objects.push(Box::new(
        SphereBuilder::default()
            .radius(9.0)
            .center(Vector3::from([22.0, 5.0, -100.0]))
            .color([30, 120, 30, 255])
            .build()
            .unwrap(),
    ));

    let bar = if show_progress {
        let bar = ProgressBar::new((width * height).into());
        bar.set_draw_delta((width * height / 200).into());
        bar.set_style(
            ProgressStyle::default_bar().template("[{elapsed_precise}] {bar:40} {pos}/{len} rays"),
        );
        Some(bar)
    } else {
        None
    };

    if output_filename.is_some() {
        let mut image_buffer: Vec<u8> = vec![0; (width * height * 4) as usize];

        let start = Instant::now();
        raytrace(scene, &mut image_buffer, bar);
        let duration = start.elapsed();

        let filename = output_filename.unwrap();
        let image =
            RgbaImage::from_raw(width, height, image_buffer).expect("Failed to convert buffer");
        image.save(filename).expect("Unable to write image");
        println!("Output written to {} in {:?}", filename, duration);

        return;
    }

    let mut window: Window = Window::new(
        "raytracer",
        width as usize,
        height as usize,
        WindowOptions {
            borderless: true,
            ..Default::default()
        },
    )
    .unwrap();

    println!("Rendering to window. Press escape to exit");

    let image_buffer: Vec<u32> = vec![0; (width * height) as usize];
    let buffer_mutex = Arc::new(Mutex::new(image_buffer));
    raytrace_fb(scene, &buffer_mutex, bar);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let buffer = buffer_mutex.lock().unwrap();
        window.update_with_buffer(&buffer).unwrap();
        drop(buffer);
        sleep(std::time::Duration::from_millis(100));
    }
}
