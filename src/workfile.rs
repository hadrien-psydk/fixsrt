use std;
use std::io;
use std::io::Lines;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::str::FromStr;
use std::io::{Error, ErrorKind};

// Helps creating a working file and move the working file
// to the final file when done, or automatically delete the
// working file in case of error.
pub struct WorkFile {
	file_path: String,
	work_file_path: String,
	file: Option<File>
}

impl WorkFile {
	pub fn create(file_path: &str) -> io::Result<WorkFile> {
		let work_file_path: String = format!("{}.work", file_path);
		let file = match File::create(&work_file_path) {
			Ok(file) => file,
			Err(err) => { return Err(err); }
		};
		Ok(WorkFile {
			file_path: file_path.to_string(),
			work_file_path: work_file_path,
			file: Some(file)
		})
	}

	pub fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		let ret = match self.file {
			Some(ref mut some_file) => some_file.write(buf),
			None => Err( Error::new(ErrorKind::Other, "oops") )
		};
		ret
	}

	pub fn commit(&mut self) {
		let file = self.file.take();
		drop(file);
		match std::fs::rename(&self.work_file_path, &self.file_path) {
			Ok(_) => (),
			Err(err) => panic!("commit failed: {}", err)
		}
	}
}

impl Drop for WorkFile {
	fn drop(&mut self) {
		if self.file.is_some() {
			drop(self.file.take());
			match std::fs::remove_file(&self.work_file_path) {
				Ok(_) => (),
				Err(err) => panic!("rollback failed: {}", err)
			}
		}
	}
}
