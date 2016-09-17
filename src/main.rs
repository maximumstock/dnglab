use std::env;
use std::fs::File;
use std::error::Error;
use std::io::prelude::*;
use std::io::BufWriter;
extern crate time;
extern crate toml;
extern crate rawloader;
use rawloader::decoders;
use rawloader::imageops;

fn usage() {
  println!("rawloader <file> [outfile]");
  std::process::exit(1);
}

fn error(err: &str) {
  println!("ERROR: {}", err);
  std::process::exit(2);
}

fn main() {
  let args: Vec<_> = env::args().collect();
  if args.len() < 2 {
    usage();
  }
  let file = &args[1];
  let fileppm = format!("{}.ppm",file);
  let outfile = if args.len() > 2 {
    &args[2]
  } else {
    &fileppm
  };
  println!("Loading file \"{}\" and saving it as \"{}\"", file, outfile);

  let rawloader = decoders::RawLoader::new();
  let from_time = time::precise_time_ns();
  let image = match rawloader.decode_safe(file) {
    Ok(val) => val,
    Err(e) => {error(&e);unreachable!()},
  };
  let to_time = time::precise_time_ns();
  println!("Decoded in {} ms", (to_time - from_time)/1000000);

  println!("Found camera \"{}\" model \"{}\"", image.make, image.model);
  println!("Found canonical named camera \"{}\" model \"{}\"", image.canonical_make, image.canonical_model);
  println!("Image size is {}x{}", image.width, image.height);
  println!("WB coeffs are {:?}", image.wb_coeffs);
  println!("black levels are {:?}", image.blacklevels);
  println!("white levels are {:?}", image.whitelevels);
  println!("color matrix is {:?}", image.color_matrix);
  println!("dcraw filters is {:#x}", image.dcraw_filters);
  println!("crops are {:?}", image.crops);

  let mut sum: u64 = 0;
  for i in 0..(image.width*image.height) {
    sum += image.data[i as usize] as u64;
  }
  println!("Image sum: {}", sum);
  let count: u64 = (image.width as u64) * (image.height as u64);
  println!("Image avg: {}", sum/count);

  let decoded = imageops::simple_decode(&image);

  let uf = match File::create(outfile) {
    Ok(val) => val,
    Err(e) => {error(e.description());unreachable!()},
  };
  let mut f = BufWriter::new(uf);
  let preamble = format!("P6 {} {} {}\n", image.width, image.height, 65535).into_bytes();
  if let Err(err) = f.write_all(&preamble) {
    error(err.description());
  }
  for pix in decoded {
    let pixel = ((pix.max(0.0)*65535.0).min(65535.0)) as u16;
    let bytes = [(pixel>>4) as u8, (pixel&0x0f) as u8];
    if let Err(err) = f.write_all(&bytes) {
      error(err.description());
    }
  }
}
