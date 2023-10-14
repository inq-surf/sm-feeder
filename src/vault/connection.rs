use std::{env, error::Error};

use surrealdb::{Surreal, engine::local::{Db, RocksDb}};

pub async fn get_db() -> Result<Surreal<Db>, Box<dyn Error>> {
    let path = env::current_dir()?;
    let path = path.join("db");

    let db = Surreal::new::<RocksDb>(path).await?;
    db.use_ns("dbo").use_db("default").await?;

    Ok(db)
}
