use anyhow::{Result, anyhow};
use serde_json::Value;

pub async fn get_olapi_comics_by_id(id: &str) -> Result<Value> {
    let url = format!(
        "https://openlibrary.org/api/books?bibkeys=OLID:{}&jscmd=details&format=json",
        id.replace("_3", "")
    );
    println!("{}", url);

    let response = reqwest::get(&url).await?;
    let data = response.json::<Value>().await?;
    println!("{:?}", data);

    Ok(data)
}

pub async fn get_olapi_search(name: &str) -> Result<Value> {
    if name.is_empty() {
        println!("OL API: name is empty");
        return Err(anyhow!("Name is empty"));
    }

    let mut sanitized_name = name.to_string();
    sanitized_name = sanitized_name.replace(&['(', ')', '[', ']', '{', '}', '#'][..], "");
    sanitized_name = sanitized_name.trim_end().to_string();

    println!("OL API: name: {}", sanitized_name);

    let url = format!(
        "http://openlibrary.org/search.json?q={}",
        urlencoding::encode(&sanitized_name)
    );

    let response = reqwest::get(&url).await?;
    let data = response.json::<Value>().await?;
    Ok(data)
}

pub async fn get_olapi_book(key: &str) -> Result<Value> {
    if key.is_empty() {
        println!("OL API: key is empty");
        return Err(anyhow!("Key is empty"));
    }

    println!("OL API: book: {}", key);

    let url = format!(
        "https://openlibrary.org/api/books?bibkeys=OLID:{}&jscmd=details&format=json",
        key
    );

    let response = reqwest::get(&url).await?;
    let data = response.json::<Value>().await?;
    Ok(data)
}
