use anyhow::{Result, anyhow};
use serde_json::Value;

pub async fn get_gbapi_comics_by_id(id: &str) -> Result<Value> {
    let url = format!("https://www.googleapis.com/books/v1/volumes/{}", id);
    println!("{}", url);

    let response = reqwest::get(&url).await?;
    let data = response.json::<Value>().await?;
    println!("{:?}", data);

    Ok(data)
}
pub async fn search_gbapi_comics_by_name(name: &str, cred: String) -> Result<Value> {
    if name.is_empty() {
        println!("search_gbapi_comics_by_name: name is empty");
        return Err(anyhow!("Name is empty"));
    }

    let sanitized_name = name
        .replace(
            |c: char| c == '(' || c == ')' || c == '[' || c == ']' || c == '{' || c == '}',
            "",
        )
        .replace("#", "")
        .trim()
        .to_string();

    let url = format!(
        "https://www.googleapis.com/books/v1/volumes?q={}&maxResults=1&key={}",
        urlencoding::encode(&sanitized_name),
        cred
    );

    println!("search_gbapi_comics_by_name: URL: {}", url);

    let response = reqwest::get(&url).await?;
    let data = response.json::<Value>().await?;
    println!("{:?}", data);

    Ok(data)
}
