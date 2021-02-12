#[macro_use] extern crate sciter;
use sciter::{Value};
use std::env;

use flate2::read::ZlibEncoder;
use flate2::Compression;
use imagequant;
use lodepng::{self, ColorType::PALETTE, CompressSettings, State, RGBA};
use std::io::Read;
use std::os::raw::c_uchar;
use std::path::Path;

use std::{thread};

struct EventHandler {}
impl EventHandler {
	fn compressPNG(&self, filename: sciter::Value, callback: sciter::Value) -> () {
		println!("Compressing {} ...", filename);
		thread::spawn(move || {
			let outcome = compress_file(filename.as_string().unwrap());
			callback.call(None, &make_args!(outcome), None).unwrap();
		});
	}
}

impl sciter::EventHandler for EventHandler {
	dispatch_script_call! {
		fn compressPNG(Value, Value);
	}
}

fn compress_file(image_name: String) -> String {
		let filename = Path::new(&image_name);
		if !filename.is_file() {
        panic!("Invalid filename");
		}
		let output_filename = format!(
        "{}-compressed.{}",
        filename.file_stem().unwrap().to_str().unwrap(),
        filename.extension().unwrap().to_str().unwrap()
    );
    let output_filename = filename.with_file_name(output_filename);

    let file = match lodepng::decode32_file(filename) {
        Ok(file) => file,
        Err(_) => panic!("Could not open file {}", "output_filename"),
		};
		let width = file.width;
    let height = file.height;

    let (palette, pixels) = quantize(&file.buffer, width as usize, height as usize);

    let mut state = make_state();

    add_palette_to_state(&mut state, palette);

    // encode and output final filesize
    match state.encode(&pixels, width, height) {
        Ok(out_buffer) => {
            println!("Output buffer size {}", out_buffer.len());
            filtering(out_buffer, width, height);
        }
        Err(_) => {
            panic!("failed to encode the image");
        }
    }

    state
        .encode_file(output_filename, &pixels, width, height)
				.unwrap();
				
		"Compression complete.".to_string()
	}
fn main() {
	sciter::set_options(sciter::RuntimeOptions::ScriptFeatures(
		sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SYSINFO as u8 
		| sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_FILE_IO as u8
		| sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SOCKET_IO as u8
		| sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_EVAL as u8
  )).unwrap();
	let mut frame = sciter::Window::new();
	frame.event_handler(EventHandler {});
	let dir = env::current_dir().unwrap().as_path().display().to_string();
	let filename = format!("{}\\{}", dir, "main.htm");
	frame.load_file(&filename);
	frame.run_app();
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