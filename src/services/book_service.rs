use std::collections::HashMap;

use sqlx::Row;
use sqlx::SqlitePool;

pub async fn get_books_with_blank_covers(
    db_pool: SqlitePool,
) -> Result<Vec<HashMap<String, String>>, Box<dyn std::error::Error>> {
    let query =
        "select * from Books where URLCover IS NULL OR URLCover = 'null' OR URLCover='undefined';";
    let rows = sqlx::query(query).fetch_all(&db_pool).await?;

    let mut books = Vec::new();
    for row in rows {
        let mut book = HashMap::new();
        book.insert("ID_book".to_string(), row.get("ID_book"));
        book.insert("PATH".to_string(), row.get("PATH"));
        book.insert("NOM".to_string(), row.get("NOM"));
        books.push(book);
    }

    Ok(books)
}
