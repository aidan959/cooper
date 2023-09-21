use std::{
    fs::File,
    path::Path,
    io::{self, BufWriter, Write},
};
fn flush(){
    io::stdout().flush().unwrap();
}
fn main() {
    let img_wdth : i32 = 256;
    let img_hght : i32 = 256;
    
    // unwrap raises the error
    let path = Path::new("tmp/output.ppm");
    let write_file: File = File::create(path.as_os_str()).unwrap();
    let mut writer: BufWriter<&File> = BufWriter::new(&write_file);

    write!(&mut writer ,"P3\n{} {}\n255\n", img_wdth, img_hght).unwrap();
    for j in 0..img_hght{
        println!("Lines remaining: {}", img_hght - j);
        flush(); 
        for i in 0..img_wdth{
            let r : f64 = (i as f64) / (img_wdth as f64 - 1.);
            let g : f64 = (j as f64) / (img_hght as f64 -1.);
            let b : f64 = 0.;

            let ir : u8 = (255.999 * r) as u8;
            let ig : u8 = (255.999 * g) as u8;
            let ib : u8 = (255.999 * b) as u8;

            write!(&mut writer, "{} {} {}\n", ir, ig, ib).unwrap();
            
        }

    }
}
