use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::{Error, ErrorKind};

const DOC_TYPE_WEBM: &'static str = "webm";
const DOC_TYPE_MATROSKA: &'static str = "matroska";

trait Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
    fn get_position(&self) -> u64;
    fn set_position(&mut self, position: u64) -> io::Result<u64>;
    fn seekable(&self) -> bool;
    fn element_start_notify(&self, element_id: u64, position: i64) {}
}

pub struct MkvWriter {
    file: Box<File>,
    position: u64,
}

impl MkvWriter {
    fn new(file: File) -> MkvWriter {
        MkvWriter {
            file: Box::new(file),
            position: 0,
        }
    }
}

impl Writer for MkvWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let size = self.file.write(buffer)?;
        self.position += size as u64;
        if size == buffer.len() {
            Ok(size)
        } else {
            Err(Error::new(
                ErrorKind::Other,
                "Write size is not equal to buffer size",
            ))
        }
    }

    fn get_position(&self) -> u64 {
        self.position
    }

    fn set_position(&mut self, position: u64) -> io::Result<u64> {
        let size = self.file.seek(SeekFrom::Start(position))?;
        self.position = size;
        if size == position {
            Ok(size)
        } else {
            Err(Error::new(
                ErrorKind::Other,
                "Seek position is not equal to input position",
            ))
        }
    }

    fn seekable(&self) -> bool {
        true
    }
}
