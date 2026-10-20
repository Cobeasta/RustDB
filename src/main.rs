mod storage;

fn main() -> std::io::Result<()> {
    println!("Hello, world!");
    let mut _pool = storage::buffer_pool::BufferPool::new()?;
    _pool.new_page();
    let _page = _pool.fetch_page(5)?;
    Ok(())
}
