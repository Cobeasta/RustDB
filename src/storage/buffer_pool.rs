use crate::storage::file::{self, DatabaseFile};
use crate::storage::page::Page;
use std::collections::HashMap;
use std::io::Error;


const BUFFER_POOL_SIZE: u64 = 16;
/**
 * Fetches 
 */
pub struct BufferPool {
    db_file: DatabaseFile,
    page_table: HashMap<u64,u64>,
    frames: [BufferFrame; BUFFER_POOL_SIZE as usize]
    // page table
    // frames
}
impl BufferPool {
    pub fn new() -> std::io::Result<Self> {
        let db_file = file::open_database_file()?;
        Ok(Self {db_file, frames: std::array::from_fn(|_| BufferFrame::default()), page_table: HashMap::new()})
    }
    pub fn fetch_page(&mut self, page_id: u64) -> Result<Page, Error>{
        let raw = self.db_file.read_page(page_id)?;
        Ok(Page {page_id: page_id, data: raw})
        
    }
    pub fn unpin_page() {
        unimplemented!()
    }
    pub fn new_page(&mut self) -> Result<u64, Error>{
        let page = self.db_file.allocate_page()?;
        
        println!("Page: {}", page);
        Ok(page)
    }
    pub fn flush_page() {
        unimplemented!()
    }
    pub fn flush_all_pages() {
        unimplemented!()
    }
    // fetch page524

}
struct BufferFrame {
    page: Option<Page>,
    pin_count: usize,
    is_dirty: bool
}
impl BufferFrame {
    fn new(page: Page) -> Self {
        Self {
            page: Some(page),
            pin_count: 0,
            is_dirty: false
        }
    }
    fn new_with_pin_count(page: Page, pin_count: usize) -> Self {
        Self {
            page: Some(page),
            pin_count,
            is_dirty: false
        }
    }
    fn default() -> Self {
        Self {
            page: None,
            pin_count: 0,
            is_dirty: false
        }
    }
}
// pick eviction victim
