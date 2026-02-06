use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let db_path = std::env::current_dir()?.join("tui-money.db");
    let repo = storage::SqliteRepository::new(db_path)?;

    ui::run(Box::new(repo))?;
    Ok(())
}
