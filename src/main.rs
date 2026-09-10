mod storage;

fn main() -> std::io::Result<()> {
    println!("Hello, world!");
    let mut df = storage::file::open_database_file()?;

    Ok(())
}
