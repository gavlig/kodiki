use std :: io		:: { prelude :: * };
use std :: fs		:: { File };
use std :: path		:: { Path, PathBuf };

pub fn file_path_to_string(buf: &Option<PathBuf>) -> String {
	match buf {
		Some(path) => path.display().to_string(),
		None => String::from(""),
	}
}

pub fn load_text_file(path_str: &str) -> Option<String> {
	let source_file_path = Some(PathBuf::from(path_str));
	let load_name 	= file_path_to_string(&source_file_path);
	let path 		= Path::new(&load_name);
	let display 	= path.display();

	let mut file = match File::open(&path) {
		Err(why) 	=> { println!("couldn't open {}: {}", display, why); return None; },
		Ok(file) 	=> file,
	};

	let mut file_content = String::new();
	match file.read_to_string(&mut file_content) {
		Err(why)	=> { println!("couldn't read {}: {}", display, why); return None; },
		Ok(_) 		=> println!("Opened file {} for reading", display.to_string()),
	}

	Some(file_content)
}