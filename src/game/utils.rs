use std :: io		:: { prelude :: * };
use std :: fs		:: { File };
use std :: path		:: { Path, PathBuf };

pub fn file_path_to_string(buf: &Option<PathBuf>) -> String {
	match buf {
		Some(path) => path.display().to_string(),
		None => String::from(""),
	}
}