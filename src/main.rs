mod storage;

fn main() -> std::io::Result<()> {
    println!("Hello, world!");
    let _df = storage::file::open_database_file()?;

    Ok(())
}
