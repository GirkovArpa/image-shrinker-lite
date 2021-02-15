use mozjpeg;
use std::path::{ Path, PathBuf };
use std::{str, fs};
use sciter::Value;
use crate::misc::{ Args, Options, make_error_message, append_dir };

pub fn compress_file(file_name: String, options: Options) -> Args {
  println!("jpg::compress_file");
  let path = Path::new(&file_name);
  if !path.is_file() {
    return make_error_message(format!("Not a file: {}", path.display()));
  }   
  let add_ext = match options.addExt.to_bool().unwrap() {
    true => ".min",
    _ => ""
  };
  let add_folder = options.addFolder.to_bool().unwrap();
  if add_folder {
    let path = path.clone().parent().unwrap().join("minified");
    fs::create_dir_all(path);
  }
  let out_file_name_string = format!(
    "{}{}.{}",
    path.file_stem().unwrap().to_str().unwrap(),
    add_ext,
    path.extension().unwrap().to_str().unwrap()
  );
  let mut out_file_name_path_buf = path.with_file_name(out_file_name_string);
  if add_folder {
    out_file_name_path_buf = append_dir(Path::new(&out_file_name_path_buf), "minified");
  }


  let data = std::fs::read(path).unwrap();
  let sizeBefore = data.len() as i32;
  println!("{:?}", data.len());
  let dinfo = mozjpeg::Decompress::new_mem(&data[..]).unwrap();
  println!("{:?}", dinfo.size());
  let mut dinfo = dinfo.raw().unwrap();
  let width = dinfo.width();
  let height = dinfo.height();
  let mut bitmaps = [&mut Vec::new(), &mut Vec::new(), &mut Vec::new()];
  dinfo.read_raw_data(&mut bitmaps);
  println!("{:?} {:?} {:?}", bitmaps[0].len(), bitmaps[1].len(), bitmaps[2].len());
  println!("{:?}", dinfo.finish_decompress());

  let mut cinfo = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_YCbCr);
  cinfo.set_size(width, height);
  #[allow(deprecated)] {
      cinfo.set_gamma(1.0);
  }
  cinfo.set_progressive_mode();
  cinfo.set_scan_optimization_mode(mozjpeg::ScanMode::Auto);
  cinfo.set_raw_data_in(true);
  cinfo.set_quality(88.);
  cinfo.set_mem_dest();
  for (c, samp) in cinfo.components_mut().iter_mut().zip(vec![2, 1, 1]) {
    c.v_samp_factor = samp;
    c.h_samp_factor = samp;
  }
  cinfo.start_compress();
  //cinfo.write_marker(Marker::APP(2), "Hello World".as_bytes());
  cinfo.write_raw_data(&bitmaps.iter().map(|c| &c[..]).collect::<Vec<_>>());
  cinfo.finish_compress();
  let output = cinfo.data_to_vec().unwrap();
  println!("{:?}", output.len());
  let sizeAfter = output.len() as i32;
  match std::fs::write(&out_file_name_path_buf, output) {
    Ok(_) => Args { 
      path: Value::from(out_file_name_path_buf.display().to_string()),
      sizeBefore: Value::from(sizeBefore),
      sizeAfter: Value::from(sizeAfter),
      error: Value::from(false)
    },
    Err(_) => make_error_message(format!("Failed to write the file: {}", out_file_name_path_buf.display()))
  }
}