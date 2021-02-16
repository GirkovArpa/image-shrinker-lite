use sciter::Value;
use std::path::{ Path, PathBuf };
pub struct Args {
    pub path: sciter::Value,
    pub sizeBefore: sciter::Value,
    pub sizeAfter: sciter::Value,
    pub error: sciter::Value
}

pub struct Options {
    pub addExt: Value,
    pub addFolder: Value,
}

pub fn make_error_message(message: String) -> Args {
    Args {
        path: Value::from(false),
        sizeBefore: Value::from(false),
        sizeAfter: Value::from(false),
        error: Value::from(message)
    }
}
// https://stackoverflow.com/a/66194330/13378247
pub fn append_dir(p: &Path, d: &str) -> PathBuf {
    let dirs = p.parent().unwrap();
    dirs.join(d).join(p.file_name().unwrap())
}