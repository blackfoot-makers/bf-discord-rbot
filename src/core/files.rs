//! Abstraction of a file handler, allow you to open a file and give it a structure, to serialize/deserialize from.
//!
//! # Exemple
//! ```rust,no_run
//!struct Credentials {
//!    pub email: String,
//!    pub password: String,
//!    pub domain: String,
//!    pub token: String,
//!}     
//!
//!files::build(
//!    String::from("credentials.json"),
//!        email: String::new(),
//!        Credentials {
//!            password: String::new(),
//!            domain: String::new(),
//!            token: String::new(),
//!        }
//!);
//!
//!main() {
//!    let credentials = &CREDENTIALS_FILE.stored;
//!    println!("{} {}", &credentials.email, &credentials.password);
//!}
//! ```



use serde_json::{from_reader, from_str, to_string, Value};
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;

pub struct FileReader<T> {
    file: File,
    pub stored: T,
}

#[allow(dead_code)]
impl<T> FileReader<T>
where
    for<'de> T: serde::Deserialize<'de>,
    for<'de> T: serde::Serialize,
{
    pub fn read(&mut self) -> io::Result<String> {
        let mut contents = String::new();
        self.file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    pub fn write_string(&mut self, content: String) -> io::Result<()> {
        let len = content.len() as u64;
        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(&content.into_bytes())?;
        self.file.set_len(len)?;
        Ok(())
    }

    // Write the self.stored struct to the file
    pub fn write_stored(&mut self) -> io::Result<()> {
        let content = to_string(&self.stored)?;
        let len = content.len() as u64;
        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(&content.into_bytes())?;
        self.file.set_len(len)?;
        Ok(())
    }

    /// Read the file, deserialise it automaticly and store it in the Structure `self.stored`
    pub fn read_struct(&mut self) {
        self.stored = from_reader(self.file.try_clone().unwrap()).unwrap();
    }

    /// Read the file, deserialise it automaticly and return a `Value`
    pub fn read_json(&mut self) -> serde_json::Result<Value> {
        from_str(&self.read().unwrap())
    }
}

/// Create a new [`FileReader`] and read/fill with [`read_struct`]
///
/// # Exemple
/// ```rust,no_run
///files::build(
///    String::from("credentials.json"),
///        email: String::new(),
///        Credentials {
///            password: String::new(),
///            domain: String::new(),
///            token: String::new(),
///        }
///);
/// ```
/// [`FileReader`]: struct.FileReader.html
/// [`read_struct`]: struct.FileReader.html#method.read_struct

pub fn build<T>(name: String, stored: T) -> FileReader<T>
where
    for<'de> T: serde::Deserialize<'de>,
    for<'de> T: serde::Serialize,
{
    let existed = Path::new(&name).exists();
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(false)
        .open(name)
        .unwrap();
    let mut filereader = FileReader { file, stored };
    if existed {
        filereader.read_struct();
    } else {
        filereader.write_stored().unwrap();
    }
    filereader
}
