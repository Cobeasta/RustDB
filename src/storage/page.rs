use crate::storage::constants;

// todo split page into header, data. Pointless to do this early because there is no metadata to store in the header.
pub struct Page {
    pub page_id: u64,
    pub data: [u8; constants::PAGE_SIZE_BYTES as usize],
}
impl Page {
    /// Create a new page with zeroed data.
    pub fn new(page_id: u64) -> Self {
        Self {
            page_id,
            data: [0; constants::PAGE_SIZE_BYTES as usize],
        }
    }

    /// Reset the page for reuse.
    pub fn reset(&mut self, page_id: u64) {
        self.page_id = page_id;
        self.data.fill(0);
    }

    /// Read the page's entire contents.
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable access to the page's contents.
    pub fn get_data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Zero out the page's contents without changing its ID.
    pub fn clear(&mut self) {
        self.data.fill(0);
    }
}