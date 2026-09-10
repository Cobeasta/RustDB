use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::prelude::*;
use std::io::{self, SeekFrom};
use std::path::Path;

const PAGE_SIZE_BYTES: u64 = 4096;
const DATABASE_FILEPATH: &str = "database.db";
const FILE_HEADER_SIZE: usize = 20;
const PAGE_COUNT_OFFSET: u64 = 12;
pub struct DatabaseFile {
    file: File,
    header: FileHeader,
}
impl DatabaseFile {
    pub fn read_page(&mut self, page_id: u64) -> std::io::Result<Vec<u8>> {
        self.file
            .seek(SeekFrom::Start(page_id * self.header.page_size))?;
        let mut buffer = vec![0; self.header.page_size as usize];
        self.file.read_exact(&mut buffer)?;
        Ok(buffer)
    }
    pub fn write_page(&mut self, page_id: u64, data: &[u8]) -> std::io::Result<()> {
        self.file
            .seek(SeekFrom::Start(page_id * self.header.page_size))?;
        self.file.write_all(data)?;

        Ok(())
    }
    pub fn allocate_page(&mut self) -> std::io::Result<u64> {
        let page_id = self.header.page_count;
        // write empty page to file
        let empty_page = vec![0u8; 4096];
        self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(&empty_page)?;

        // update page count
        self.header.page_count += 1;
        self.file.seek(SeekFrom::Start(PAGE_COUNT_OFFSET))?;
        self.file.write_all(&self.header.page_count.to_le_bytes())?;
        Ok(page_id)
    }
}
// header should be first page.
struct FileHeader {
    version: u32,
    page_size: u64,
    page_count: u64,
}
impl FileHeader {}

pub fn open_database_file() -> Result<(DatabaseFile), std::io::Error> {
    let path = Path::new(DATABASE_FILEPATH);
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;

    let metadata = file.metadata()?;
    let header = if metadata.len() == 0 {
        // construct new header

        let header = FileHeader {
            version: 1,
            page_size: PAGE_SIZE_BYTES,
            page_count: 1,
        };

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&header.version.to_le_bytes());
        bytes.extend_from_slice(&header.page_size.to_le_bytes());
        bytes.extend_from_slice(&header.page_count.to_le_bytes());
        bytes.resize(4096, 0u8);
        file.write_all(&bytes);
        header
    } else {
        let mut header_bytes = [0u8; FILE_HEADER_SIZE];
        file.read_exact(&mut header_bytes);

        let version = u32::from_le_bytes(header_bytes[0..4].try_into().unwrap());
        let page_size = u64::from_le_bytes(header_bytes[4..12].try_into().unwrap());
        let page_count = u64::from_le_bytes(header_bytes[12..20].try_into().unwrap());
        println!(
            "Version: {0} size: {1} count {2}",
            version, page_size, page_count
        );
        FileHeader {
            version: version,
            page_size: page_size,
            page_count: page_count,
        }
    };

    Ok(DatabaseFile { file, header })
}
