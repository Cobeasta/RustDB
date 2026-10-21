use crate::storage::file::{self, DatabaseFile};
use crate::storage::page::Page;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;

pub const DEFAULT_POOL_SIZE: usize = 16;

/// Caches pages from the database file in a fixed number of in-memory frames.
///
/// Pages are accessed through `PageGuard`s. While a guard is alive its page is
/// pinned and cannot be evicted; dropping the guard unpins it. Several guards
/// can be held at once (e.g. a parent and child node during a B+ tree split).
///
/// Bookkeeping (page table, pin counts, dirty flags) lives in one `RefCell`,
/// and each frame's page data lives in its own `RefCell`, so methods take
/// `&self` and guards for different pages don't conflict with each other.
pub struct BufferPool {
    state: RefCell<PoolState>,
    pages: Vec<RefCell<Page>>,
}

struct PoolState {
    db_file: DatabaseFile,
    /// page_id -> frame index
    page_table: HashMap<u64, usize>,
    frames: Vec<BufferFrame>,
}

/// Metadata for one frame. The page bytes are stored in `BufferPool::pages`
/// at the same index.
#[derive(Default)]
struct BufferFrame {
    /// `None` if the frame is empty.
    page_id: Option<u64>,
    pin_count: usize,
    /// True if the page was modified since it was read from / written to disk.
    is_dirty: bool,
}

impl BufferPool {
    pub fn new(path: impl AsRef<Path>, pool_size: usize) -> Result<Self> {
        assert!(pool_size > 0, "buffer pool needs at least one frame");
        let db_file = file::open_database_file(path)?;
        Ok(Self {
            state: RefCell::new(PoolState {
                db_file,
                page_table: HashMap::new(),
                frames: (0..pool_size).map(|_| BufferFrame::default()).collect(),
            }),
            pages: (0..pool_size).map(|_| RefCell::new(Page::new(0))).collect(),
        })
    }

    /// Fetch a page, reading it from disk if it isn't already cached.
    pub fn fetch_page(&self, page_id: u64) -> Result<PageGuard<'_>> {
        if page_id == 0 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "page 0 is the file header and is not accessible through the buffer pool",
            ));
        }
        let mut state = self.state.borrow_mut();

        // Cache hit.
        if let Some(&frame_idx) = state.page_table.get(&page_id) {
            state.frames[frame_idx].pin_count += 1;
            return Ok(PageGuard { pool: self, frame_idx, page_id });
        }

        // Cache miss: load into a free (or freed-up) frame.
        let frame_idx = self.acquire_frame(&mut state)?;
        let data = state.db_file.read_page(page_id)?;
        {
            let mut page = self.pages[frame_idx].borrow_mut();
            page.page_id = page_id;
            page.data = data;
        }
        self.install(&mut state, frame_idx, page_id);
        Ok(PageGuard { pool: self, frame_idx, page_id })
    }

    /// Allocate a new zeroed page on disk and return it pinned.
    pub fn new_page(&self) -> Result<PageGuard<'_>> {
        let mut state = self.state.borrow_mut();
        // Grab a frame first so we don't allocate a page on disk and then
        // fail to find room for it.
        let frame_idx = self.acquire_frame(&mut state)?;
        let page_id = state.db_file.allocate_page()?;
        self.pages[frame_idx].borrow_mut().reset(page_id);
        self.install(&mut state, frame_idx, page_id);
        Ok(PageGuard { pool: self, frame_idx, page_id })
    }

    /// Write a page back to disk if it's cached and dirty.
    pub fn flush_page(&self, page_id: u64) -> Result<()> {
        let mut state = self.state.borrow_mut();
        if let Some(&frame_idx) = state.page_table.get(&page_id) {
            self.flush_frame(&mut state, frame_idx)?;
        }
        Ok(())
    }

    /// Write every dirty cached page back to disk.
    pub fn flush_all_pages(&self) -> Result<()> {
        let mut state = self.state.borrow_mut();
        for frame_idx in 0..state.frames.len() {
            self.flush_frame(&mut state, frame_idx)?;
        }
        Ok(())
    }

    fn flush_frame(&self, state: &mut PoolState, frame_idx: usize) -> Result<()> {
        let frame = &state.frames[frame_idx];
        if let (Some(page_id), true) = (frame.page_id, frame.is_dirty) {
            let page = self.pages[frame_idx].borrow();
            state.db_file.write_page(page_id, page.get_data())?;
            state.frames[frame_idx].is_dirty = false;
        }
        Ok(())
    }

    /// Find an empty frame, evicting an unpinned page if necessary.
    /// The returned frame is empty and not in the page table.
    fn acquire_frame(&self, state: &mut PoolState) -> Result<usize> {
        if let Some(idx) = state.frames.iter().position(|f| f.page_id.is_none()) {
            return Ok(idx);
        }
        // Simple eviction policy: first unpinned frame. Swap for clock/LRU later.
        let victim = state
            .frames
            .iter()
            .position(|f| f.pin_count == 0)
            .ok_or_else(|| Error::other("buffer pool full: every frame is pinned"))?;

        self.flush_frame(state, victim)?;
        let old_id = state.frames[victim].page_id.take().unwrap();
        state.page_table.remove(&old_id);
        Ok(victim)
    }

    /// Record that `frame_idx` now holds `page_id`, pinned once.
    fn install(&self, state: &mut PoolState, frame_idx: usize, page_id: u64) {
        state.frames[frame_idx] = BufferFrame {
            page_id: Some(page_id),
            pin_count: 1,
            is_dirty: false,
        };
        state.page_table.insert(page_id, frame_idx);
    }

    fn unpin(&self, frame_idx: usize) {
        let mut state = self.state.borrow_mut();
        let frame = &mut state.frames[frame_idx];
        debug_assert!(frame.pin_count > 0, "unpinning a page that isn't pinned");
        frame.pin_count = frame.pin_count.saturating_sub(1);
    }

    fn mark_dirty(&self, frame_idx: usize) {
        self.state.borrow_mut().frames[frame_idx].is_dirty = true;
    }
}

impl Drop for BufferPool {
    fn drop(&mut self) {
        // Best effort: can't return an error from drop. Call flush_all_pages
        // explicitly if you need to handle failures.
        let _ = self.flush_all_pages();
    }
}

/// A pinned page. The page stays in memory until the guard is dropped.
///
/// `read()` and `write()` follow `RefCell` rules: you can have many reads or
/// one write of the same page at a time, otherwise it panics.
pub struct PageGuard<'a> {
    pool: &'a BufferPool,
    frame_idx: usize,
    page_id: u64,
}

impl PageGuard<'_> {
    pub fn page_id(&self) -> u64 {
        self.page_id
    }

    pub fn read(&self) -> Ref<'_, Page> {
        self.pool.pages[self.frame_idx].borrow()
    }

    /// Mutable access. Marks the page dirty so it's written back on eviction/flush.
    pub fn write(&self) -> RefMut<'_, Page> {
        self.pool.mark_dirty(self.frame_idx);
        self.pool.pages[self.frame_idx].borrow_mut()
    }
}

impl Drop for PageGuard<'_> {
    fn drop(&mut self) {
        self.pool.unpin(self.frame_idx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Temp database file that is deleted when dropped.
    struct TempDb(PathBuf);
    impl TempDb {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir()
                .join(format!("rustdb_test_{}_{}.db", name, std::process::id()));
            let _ = std::fs::remove_file(&path);
            TempDb(path)
        }
    }
    impl Drop for TempDb {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    #[test]
    fn new_page_ids_start_after_header() {
        let db = TempDb::new("ids");
        let pool = BufferPool::new(&db.0, 4).unwrap();
        assert_eq!(pool.new_page().unwrap().page_id(), 1);
        assert_eq!(pool.new_page().unwrap().page_id(), 2);
    }

    #[test]
    fn data_survives_eviction() {
        let db = TempDb::new("evict");
        let pool = BufferPool::new(&db.0, 2).unwrap();
        let mut ids = Vec::new();
        for i in 0..5u8 {
            let g = pool.new_page().unwrap();
            g.write().data[0] = i + 1;
            ids.push(g.page_id());
        } // only 2 frames, so earlier pages were evicted and written back
        for (i, id) in ids.iter().enumerate() {
            assert_eq!(pool.fetch_page(*id).unwrap().read().data[0], i as u8 + 1);
        }
    }

    #[test]
    fn data_survives_reopen() {
        let db = TempDb::new("reopen");
        let id = {
            let pool = BufferPool::new(&db.0, 4).unwrap();
            let g = pool.new_page().unwrap();
            g.write().data[100] = 42;
            g.page_id()
        }; // pool dropped -> flushed
        let pool = BufferPool::new(&db.0, 4).unwrap();
        assert_eq!(pool.fetch_page(id).unwrap().read().data[100], 42);
    }

    #[test]
    fn all_pinned_is_an_error() {
        let db = TempDb::new("pinned");
        let pool = BufferPool::new(&db.0, 2).unwrap();
        let _a = pool.new_page().unwrap();
        let _b = pool.new_page().unwrap();
        assert!(pool.new_page().is_err());
    }

    #[test]
    fn unpinned_frame_is_reused() {
        let db = TempDb::new("reuse");
        let pool = BufferPool::new(&db.0, 1).unwrap();
        let a = pool.new_page().unwrap().page_id();
        let b = pool.new_page().unwrap().page_id();
        assert_ne!(a, b);
        assert!(pool.fetch_page(a).is_ok());
    }

    #[test]
    fn fetching_cached_page_shares_frame() {
        let db = TempDb::new("shared");
        let pool = BufferPool::new(&db.0, 2).unwrap();
        let id = pool.new_page().unwrap().page_id();
        let g1 = pool.fetch_page(id).unwrap();
        let g2 = pool.fetch_page(id).unwrap();
        g1.write().data[0] = 7;
        assert_eq!(g2.read().data[0], 7);
    }

    #[test]
    fn missing_page_and_header_are_errors() {
        let db = TempDb::new("missing");
        let pool = BufferPool::new(&db.0, 2).unwrap();
        assert!(pool.fetch_page(0).is_err());
        assert!(pool.fetch_page(99).is_err());
        // a failed fetch must not leak a frame
        assert!(pool.new_page().is_ok());
        assert!(pool.new_page().is_ok());
    }
}
