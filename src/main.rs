mod storage;

use storage::buffer_pool::{BufferPool, DEFAULT_POOL_SIZE};

fn main() -> std::io::Result<()> {
    let pool = BufferPool::new("database.db", DEFAULT_POOL_SIZE)?;
    let page = pool.new_page()?;
    println!("allocated page {}", page.page_id());
    Ok(())
}
