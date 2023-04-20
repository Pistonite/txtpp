use std::io::{self, Write};
use std::{fs::File, io::BufWriter};

pub struct WriteWrapper {
    str: Option<String>,
    file: Option<BufWriter<File>>,
}

impl WriteWrapper {
    pub fn new_str() -> Self {
        WriteWrapper {
            str: Some(String::new()),
            file: None,
        }
    }
    pub fn new_file(file: File) -> Self {
        WriteWrapper {
            str: None,
            file: Some(BufWriter::new(file)),
        }
    }
    pub fn write(&mut self, output: &str) -> io::Result<()> {
        match &mut self.str {
            Some(s) => {
                s.push_str(output);
                Ok(())
            }
            None => self.file.as_mut().unwrap().write_all(output.as_bytes())
        }
    }
    pub fn done(&mut self) -> io::Result<()> {
        match self.file.as_mut() {
            Some(f) => f.flush(),
            None => Ok(()),
        }
    }

    pub fn get(self) -> String {
        self.str.unwrap()
    }
}
