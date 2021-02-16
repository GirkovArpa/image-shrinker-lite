use sciter::{Value};

use flate2::read::ZlibEncoder;
use flate2::Compression;
use imagequant;
use lodepng::{self, ColorType::PALETTE, CompressSettings, State, RGBA};
use std::io::Read;
use std::os::raw::c_uchar;
use std::path::{ Path, PathBuf };
use std::{str, fs};
// https://stackoverflow.com/a/55033999/13378247
use crate::misc::{ Args, Options, make_error_message, append_dir };

pub fn compress_file(file_name: String, options: Options) -> Args {
    println!("png::compress_file");
    let path = Path::new(&file_name);
    if !path.is_file() {
        return make_error_message(format!("Not a file: {}", path.display()));
    }    
    let in_file_name_path_buf = PathBuf::from(path);
    let file = match lodepng::decode32_file(&in_file_name_path_buf) {
        Ok(file) => file,
        Err(_) => return make_error_message(format!("Could not open file: {}", in_file_name_path_buf.display()))
    };
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
	let width = file.width;
    let height = file.height;
    let in_buffer_len = file.buffer.len();
    let (palette, pixels) = quantize(&file.buffer, width as usize, height as usize);
    let mut state = make_state();
    add_palette_to_state(&mut state, palette);
    let out_buffer_len: usize;
    // encode and output final filesize
    match state.encode(&pixels, width, height) {
        Ok(out_buffer) => {
            out_buffer_len = out_buffer.len();
            filtering(out_buffer, width, height);
        }
        Err(_) => {
            return make_error_message("Failed to encode the image.".to_string())
        }
    }
    match state.encode_file(&out_file_name_path_buf, &pixels, width, height) {
        Err(e) => {
            let err_msg = str::from_utf8(e.c_description());
            let err_msg = err_msg.ok().unwrap();
            return make_error_message(format!("{:?}", err_msg))
        }
        _ => Args { 
            path: Value::from(out_file_name_path_buf.display().to_string()),
            sizeBefore: Value::from(in_buffer_len as i32),
            sizeAfter: Value::from(out_buffer_len as i32),
            error: Value::from(false)
        }
    }
}


// using imagequant quantize the PNG to reduce the file size
// returns the palette and vector of pixels
fn quantize(buffer: &[RGBA], width: usize, height: usize) -> (Vec<RGBA>, Vec<u8>) {
    // quantize
    let mut liq = imagequant::new();
    liq.set_speed(1);
    liq.set_quality(70, 99);

    let ref mut img = liq
        .new_image(&buffer, width as usize, height as usize, 0.45455)
        .unwrap();

    let mut res = match liq.quantize(img) {
        Ok(res) => res,
        Err(_) => panic!("Failed to quantize image"),
    };

    res.remapped(img).unwrap()
}

// create the initial state with default settings for PNG with a palette
// use flate2 for the image compression rather than the compression that comes with the
// lonepng package, flate2 tends to be significantly faster as well as produces a smaller image
fn make_state() -> State {
    let mut state = lodepng::ffi::State::new();
    state.info_png_mut().color.colortype = PALETTE;
    state.info_png_mut().color.set_bitdepth(8);

    state.info_raw_mut().colortype = PALETTE;
    state.info_raw_mut().set_bitdepth(8);

    // lib uses custom deflate which is slower and creates a larger filesize than flate2
    unsafe {
        state.set_custom_zlib(Some(deflate_ffi), std::ptr::null());
    }

    state.encoder.add_id = 0;
    state.encoder.text_compression = 1;

    state
}

// add the palette from the quantization to the image state
fn add_palette_to_state(state: &mut State, palette: Vec<RGBA>) {
    palette.iter().for_each(|palette| {
        state
            .info_png_mut()
            .color
            .palette_add(palette.clone())
            .unwrap();

        state.info_raw_mut().palette_add(palette.clone()).unwrap();
    });
}

// to override the default compressor for lodepng, an unsafe external c function has to be passed to used
unsafe extern "C" fn deflate_ffi(
    out: &mut *mut c_uchar,
    out_size: &mut usize,
    input: *const c_uchar,
    input_size: usize,
    settings: *const CompressSettings,
) -> u32 {
    let input = vec_from_raw(input, input_size);
    let settings = std::ptr::read(settings);

    let (mut buffer, size) = deflate(&input, settings);

    std::mem::replace(out, buffer.as_mut_ptr());
    std::ptr::replace(out_size, size);

    return 0;
}

// call flate2's zlib encoder return the buffer and length
fn deflate(input: &[u8], _settings: CompressSettings) -> (Vec<u8>, usize) {
    let mut z = ZlibEncoder::new(input, Compression::best());
    let mut buffer = vec![];
    match z.read_to_end(&mut buffer) {
        Ok(len) => (buffer, len),
        Err(_) => panic!("Failed to compress buffer"),
    }
}

// convert the raw buffer to a vector
unsafe fn vec_from_raw(data: *const c_uchar, len: usize) -> Vec<u8> {
    std::slice::from_raw_parts(data, len).to_owned()
}

fn filtering(buffer: Vec<u8>, _width: usize, _height: usize) {
    //    let w_buffer: &
    //    let encoder = png::Encoder::new(&buffer as &[u8], width, height);
    //dbg!(buffer);
}