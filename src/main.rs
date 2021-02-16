#![windows_subsystem="windows"]

// https://stackoverflow.com/a/45520092/13378247
mod png;
mod jpg;
mod gif;
mod svg;

mod misc;
use misc::{ Args, Options};

#[macro_use] extern crate sciter;
use sciter::{ Value };
use std::{ env, thread };

struct EventHandler {}
impl EventHandler {
	fn compressFile(&self, filename: Value, format: Value, addExt: Value, addFolder: Value, callback: Value) -> () {
        let ext = String::from(format.to_string());
        println!("{}", ext);
        let ext = ext.as_str();
        println!("{}", ext);
        let compression_function = match &ext as &str {
            "\"png\"" => png::compress_file,
            "\"jpg\"" | "\"jpeg\"" => jpg::compress_file,
            ext => { 
                let err_msg = format!("Unsupported file extension: {}", ext);
                callback.call(None, &make_args!("", 0, 0, err_msg), None).unwrap();
                return;
            }
        };
        let options = Options { addExt: addExt, addFolder: addFolder };
		thread::spawn(move || {
            let args = compression_function(filename.as_string().unwrap(), options);
            let Args { path, sizeBefore, sizeAfter, error } = args;
            callback.call(None, &make_args!(path, sizeBefore, sizeAfter, error), None).unwrap();
        });
    }
}

impl sciter::EventHandler for EventHandler {
	dispatch_script_call! {
        fn compressFile(Value, Value, Value, Value, Value);
    }
}
fn main() {
    sciter::set_options(sciter::RuntimeOptions::DebugMode(false)).unwrap();
    let archived = include_bytes!("../target/assets.rc");
    sciter::set_options(sciter::RuntimeOptions::ScriptFeatures(
        sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SYSINFO as u8 
        | sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_FILE_IO as u8
    )).unwrap();
    let mut frame = sciter::Window::new();
    frame.event_handler(EventHandler {});
    frame.archive_handler(archived).expect("Unable to load archive");
    frame.load_file("this://app/main.htm");
    frame.run_app();
}
