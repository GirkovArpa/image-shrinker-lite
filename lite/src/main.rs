// https://stackoverflow.com/a/45520092/13378247
mod png;
mod jpg;
mod gif;
mod svg;

#[macro_use] extern crate sciter;
use sciter::{ Value };
use std::{ env, thread };


struct EventHandler {}
impl EventHandler {
	fn compressPNG(&self, filename: Value, addExt: Value, addFolder: Value, callback: Value) -> () {
        let options = png::Options { addExt: addExt, addFolder: addFolder };
		thread::spawn(move || {
            let args = png::compress_file(filename.as_string().unwrap(), options);
            let png::Args { path, sizeBefore, sizeAfter, error } = args;
            callback.call(None, &make_args!(path, sizeBefore, sizeAfter, error), None).unwrap();
        });
	}
}

impl sciter::EventHandler for EventHandler {
	dispatch_script_call! {
		fn compressPNG(Value, Value, Value, Value);
	}
}
fn main() {
    sciter::set_options(sciter::RuntimeOptions::DebugMode(true)).unwrap();
    sciter::set_options(sciter::RuntimeOptions::ScriptFeatures(
        sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SYSINFO as u8 
        | sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_FILE_IO as u8
    )).unwrap();
    let mut frame = sciter::window::Builder::main().create();
    frame.event_handler(EventHandler {});
    let dir = env::current_dir().unwrap().as_path().display().to_string();
    let filename = format!("{}\\{}", dir, "main.htm");
    frame.load_file(&filename);
    frame.run_app();
}
