// pub mod math;
// pub mod shapes;
use math::{
    colour::Colour,
    ray::Ray,
    vec3::{dot, unit_vector, Vec3},
};
use shapes::sphere::Sphere;
use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};

fn flush() {
    io::stdout().flush().unwrap();
}
const SPHERE : Vec3 = Vec3{x:0., y: 0., z:-1.};

fn ray_color(ray: &Ray) -> Colour {
    let t:f64 = has_hit_sphere(SPHERE, 0.5, ray);
    if t > 0. {
        let n : Vec3 = (ray.at(t) - SPHERE).unit_vector();

        return 0.5 * Colour::new_fraction(n.x+1.,n.y+1.,n.z+1.);
    }
    let unit_direction : Vec3 = unit_vector(ray.direction);
    let a = 0.5 * (unit_direction.y + 1.);
    return (1.0 - a) * Colour::BLACK + a * Colour::new_fraction(0.5, 0.7, 1.);
}

fn has_hit_sphere(center: Vec3, radius: f64, ray: &Ray) -> f64 {
    let oc = ray.origin - center;
    let a = ray.direction.sqr_len();
    let half_b =  dot(oc, ray.direction);
    let c = oc.sqr_len() - radius*radius;
    let discriminant = half_b * half_b - a * c;
    if discriminant < 0. {
        return -1.;
    }
    (-half_b - discriminant.sqrt()) / a
    
}

fn main() {
    let path = Path::new("tmp/output.ppm");
    let write_file: File = File::create(path.as_os_str()).unwrap();
    let mut writer: BufWriter<&File> = BufWriter::new(&write_file);

    const ASPECT_RATIO: f64 = 16. / 9.;
    const IMG_WDTH: u16 = 400;

    let mut img_hght: u16 = (IMG_WDTH as f64 / ASPECT_RATIO) as u16;
    img_hght = if img_hght < 1 { 1 as u16 } else { img_hght };

    let focal_length = 1.;
    let viewport_height = 2.0;
    let viewport_width = viewport_height * ((IMG_WDTH) as f64 / img_hght as f64);
    let camera_center = Vec3::new(0., 0., 0.);

    let viewport_u = Vec3::new(viewport_width, 0., 0.);
    let viewport_v = Vec3::new(0., -viewport_height, 0.);
    
    let pixel_delta_u = viewport_u / IMG_WDTH as f64;
    let pixel_delta_v = viewport_v / img_hght as f64;

    let viewport_upper_left =
        camera_center - Vec3::new(0., 0., focal_length) - viewport_u / 2. - viewport_v / 2.;
    let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    write!(&mut writer, "P3\n{} {}\n255\n", IMG_WDTH, img_hght).unwrap();
    for j in 0..img_hght {
        println!("Lines remaining: {}", img_hght - j);
        flush();
        for i in 0..IMG_WDTH {
            let pixel_center =
                pixel00_loc + (i as f64 * pixel_delta_u) + (j as f64 * pixel_delta_v);
            let ray_direction = pixel_center - camera_center;

            let r: Ray = Ray::new(camera_center, ray_direction);

            let pixel_color = ray_color(&r);

            writeln!(&mut writer, "{}", pixel_color).unwrap();
        }
    }
}
