/// io.rs
/// 
/// Provides functions for reading and writing files.
/// In general files are expected to be in jsonl format with one json object per line.
/// They can be zipped or unzipped, with .gz or .zstd extensions.
/// 
/// 
use std::path::{Path};
use std::io::{self, Result, BufReader, BufRead, BufWriter, Write};
use std::fs::{File};
use flate2::read::{GzDecoder};
use flate2::write::{GzEncoder};
use simd_json::OwnedValue;

const BUFFER_SIZE: usize = 1024 * 1024 * 10; // 10 MB

#[derive(Debug, PartialEq)]
enum FileType {
    Gzip,
    Inflated,
    // Add more types here, e.g., Zstd,
}

fn determine_file_type(path: &Path) -> Result<FileType> {
    
    let ext = match path.extension() {
        Some(ext) => ext.to_str(),
        None => { return Ok(FileType::Inflated) }
    };

    if ext == Some("gz") {
        return Ok(FileType::Gzip)
    }
    Ok(FileType::Inflated)
}

pub struct ArchiveReader {
    archive: Box<dyn BufRead>,
    buffer: Vec<u8>,
}

impl ArchiveReader {
    pub fn new(path: &Path) -> Result<Self> {

        // Check if the file exists
        if !path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "File not found"));
        }

        // Determine the file type based on the extension
        let file_type = determine_file_type(path)?;

        let file = File::open(path)?;
        let reader: Box<dyn BufRead> = match file_type {
            FileType::Gzip => Box::new(BufReader::with_capacity(BUFFER_SIZE, GzDecoder::new(file))),
            FileType::Inflated => Box::new(BufReader::with_capacity(BUFFER_SIZE, file)),
            // Add more types here, e.g., Zstd,
        };
        Ok(ArchiveReader { archive: reader, buffer: Vec::<u8>::new() })
    }
}

impl Iterator for ArchiveReader {
    type Item = Result<OwnedValue>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.archive.read_until(b'\n', &mut self.buffer) {
            Ok(0) => None, // EOF
            Ok(_) => {
                // Remove the trailing newline characters
                while self.buffer.last() == Some(&b'\n') || self.buffer.last() == Some(&b'\r') {
                    self.buffer.pop();
                }

                let mut as_bytes = self.buffer.as_mut();

                let val : OwnedValue = match simd_json::to_owned_value(&mut as_bytes) {
                    Ok(val) => val,
                    Err(e) => return Some(Err(io::Error::new(io::ErrorKind::InvalidData, e))),
                };

                // SAFETY: We need to clear the buffer before reusing it
                self.buffer.clear();
                
                Some(Ok(val))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

pub enum ArchiveWriter {
    Gzip(BufWriter<GzEncoder<File>>),
    Inflated(BufWriter<File>),
    // Add more types here, e.g., Zstd,
}

impl ArchiveWriter {
    pub fn new(path: &Path) -> Result<Self> {

        // Determine the file type based on the extension
        let file_type = if path.extension().and_then(|s| s.to_str()) == Some("gz") {
            FileType::Gzip
        } else if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            FileType::Inflated
        } else {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Unsupported file type"));
        };

        let file = File::create(path)?;
        if file_type == FileType::Gzip {
            let encoder = GzEncoder::new(file, flate2::Compression::default());
            let writer = BufWriter::with_capacity(BUFFER_SIZE, encoder);
            Ok(ArchiveWriter::Gzip(writer))
        } else {
            let writer = BufWriter::with_capacity(BUFFER_SIZE, file);
            Ok(ArchiveWriter::Inflated(writer))
        }
    }

    pub fn write(&mut self, val: &OwnedValue) -> Result<()> {
        match self {
            ArchiveWriter::Gzip(writer) => {
                simd_json::to_writer(&mut *writer, val)?;
                // Newline after each JSON object
                writer.write_all(b"\n")?;
            }
            ArchiveWriter::Inflated(writer) => {
                simd_json::to_writer(&mut *writer, val)?;
                writer.write_all(b"\n")?;

            }
        }
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        match self {
            ArchiveWriter::Gzip(writer) => writer.flush()?,
            ArchiveWriter::Inflated(writer) => writer.flush()?,
        }
        Ok(())
    }

    pub fn close(self) -> Result<()> {
        match self {
            ArchiveWriter::Gzip(writer) => writer.into_inner()?.finish()?,
            ArchiveWriter::Inflated(writer) => writer.into_inner()?,
        };
        Ok(())
    }
}
