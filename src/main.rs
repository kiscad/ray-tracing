mod camera;
mod hit;
mod material;
mod ray;
mod sphere;
mod vec;

use camera::Camera;
use hit::{Hit, World};
use material::{Dielectric, Lambertian, Metal};
use rand::Rng;
use ray::Ray;
use rayon::prelude::*;
use sphere::Sphere;
use std::{io::Write, sync::Arc};
use vec::{Color, Point3, Vec3};

fn ray_color(r: &Ray, world: &World, depth: usize) -> Color {
    if depth < 1 {
        return Color::new(0.0, 0.0, 0.0);
    }
    if let Some(rec) = world.hit(r, 0.001, f32::INFINITY) {
        if let Some((attenuation, scattered)) = rec.mat.scatter(r, &rec) {
            attenuation * ray_color(&scattered, world, depth - 1)
        } else {
            Color::new(0.0, 0.0, 0.0)
        }
    } else {
        let unit_dir = r.direction().normalize();
        let t = 0.5 * (unit_dir.y() + 1.0);
        (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
    }
}

fn main() {
    // Image
    const ASPECT_RATIO: f32 = 3.0 / 2.0;
    const IMAGE_WIDTH: usize = 500;
    const IMAGE_HEIGHT: usize = (IMAGE_WIDTH as f32 / ASPECT_RATIO) as usize;
    const SAMPLES_PER_PIXEL: usize = 50;
    const MAX_DEPTH: usize = 50;

    // World
    let world = random_scene();

    // Camera
    let lookfrom = Point3::new(13.0, 2.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = Vec3::new(0.0, 1.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.1;

    let cam = Camera::new(
        lookfrom,
        lookat,
        vup,
        20.0,
        ASPECT_RATIO,
        aperture,
        dist_to_focus,
    );

    println!("P3");
    println!("{} {}", IMAGE_WIDTH, IMAGE_HEIGHT);
    println!("255");

    let mut pixel_rows: Vec<_> = (0..IMAGE_HEIGHT)
        .into_par_iter()
        .map(|j| {
            let mut pixel_row = [Color::default(); IMAGE_WIDTH];
            let mut rng = rand::thread_rng();
            for i in 0..IMAGE_WIDTH {
                let mut pixel_color = Color::default();
                for _ in 0..SAMPLES_PER_PIXEL {
                    let u = (i as f32 + rng.gen::<f32>()) / (IMAGE_WIDTH - 1) as f32;
                    let v = (j as f32 + rng.gen::<f32>()) / (IMAGE_HEIGHT - 1) as f32;
                    let r = cam.get_ray(u, v);
                    pixel_color += ray_color(&r, &world, MAX_DEPTH);
                }
                pixel_row[i] = pixel_color;
            }
            (j, pixel_row)
        })
        .collect();

    pixel_rows.sort_by_key(|x| IMAGE_HEIGHT - x.0);

    let mut stdout = std::io::stdout();
    pixel_rows.iter().for_each(|(_, pixels)| {
        pixels.iter().for_each(|color| {
            stdout
                .write_all(color.format_color(SAMPLES_PER_PIXEL).as_bytes())
                .unwrap();
        })
    });
    stdout.flush().unwrap();
}

fn random_scene() -> World {
    let mut rng = rand::thread_rng();
    let mut world = World::new();

    let ground_mat = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    let ground_sphere = Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground_mat);

    world.push(Arc::new(ground_sphere));

    for a in -11..=11 {
        for b in -11..=11 {
            let choose_mat: f32 = rng.gen();
            let center = Point3::new(
                (a as f32) + rng.gen_range(0.0..0.9),
                0.2,
                (b as f32) + rng.gen_range(0.0..0.9),
            );
            if choose_mat < 0.8 {
                // Diffuse
                let albedo = Color::random(0.0..1.0) * Color::random(0.0..1.0);
                let sphere_mat = Arc::new(Lambertian::new(albedo));
                let sphere = Sphere::new(center, 0.2, sphere_mat);
                world.push(Arc::new(sphere));
            } else if choose_mat < 0.95 {
                // Metal
                let albedo = Color::random(0.4..1.0);
                let fuzz = rng.gen_range(0.0..0.5);
                let sphere_mat = Arc::new(Metal::new(albedo, fuzz));
                let sphere = Sphere::new(center, 0.2, sphere_mat);
                world.push(Arc::new(sphere));
            } else {
                // Glass
                let sphere_mat = Arc::new(Dielectric::new(1.5));
                let sphere = Sphere::new(center, 0.2, sphere_mat);
                world.push(Arc::new(sphere));
            }
        }
    }

    let mat1 = Arc::new(Dielectric::new(1.5));
    let mat2 = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    let mat3 = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));

    let sphere1 = Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, mat1);
    let sphere2 = Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0, mat2);
    let sphere3 = Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, mat3);

    world.push(Arc::new(sphere1));
    world.push(Arc::new(sphere2));
    world.push(Arc::new(sphere3));

    world
}
