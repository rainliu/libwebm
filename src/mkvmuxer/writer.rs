use crate::MkvId;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::{Error, ErrorKind};

pub trait Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<()>;
    fn get_position(&self) -> u64;
    fn set_position(&mut self, position: u64) -> io::Result<()>;
    fn seekable(&self) -> bool;
    fn element_start_notify(&self, _element_id: MkvId, _position: u64) {}
}

pub struct MkvWriter {
    file: Box<File>,
    position: u64,
}

impl MkvWriter {
    pub fn new(file: File) -> MkvWriter {
        MkvWriter {
            file: Box::new(file),
            position: 0,
        }
    }
}

impl Writer for MkvWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<()> {
        let size = self.file.write(buffer)?;
        self.position += size as u64;
        if size == buffer.len() {
            Ok(())
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

    fn set_position(&mut self, position: u64) -> io::Result<()> {
        let size = self.file.seek(SeekFrom::Start(position))?;
        self.position = size;
        if size == position {
            Ok(())
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
