use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenLibraryResponse {
    #[serde(flatten)]
    pub books: HashMap<String, OpenLibraryBook>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenLibraryBook {
    pub details: BookDetails,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BookDetails {
    pub title: String,
    pub description: Option<String>,
    pub physical_format: Option<String>,
    pub number_of_pages: Option<u32>,
    pub publish_date: Option<String>,
    pub info_url: Option<String>,
    pub authors: Option<Vec<AuthorRef>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthorRef {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenLibrarySearchResponse {
    pub start: u32,
    pub num_found: u32,
    pub docs: Vec<SearchDoc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchDoc {
    pub cover_i: Option<u32>,
    pub has_fulltext: Option<bool>,
    pub edition_count: Option<u32>,
    pub title: Option<String>,
    pub author_name: Option<Vec<String>>,
    pub first_publish_year: Option<u32>,
    pub key: Option<String>,
    pub ia: Option<Vec<String>>,
    pub author_key: Option<Vec<String>>,
    pub public_scan_b: Option<bool>,
}

pub async fn get_olapi_comics_by_id(id: &str) -> Result<OpenLibraryBook> {
    let url = format!(
        "https://openlibrary.org/api/books?bibkeys=OLID:{}&jscmd=details&format=json",
        id.replace("_3", "")
    );
    println!("{}", url);

    let response = reqwest::get(&url).await?;
    let data = response.json::<Value>().await?;
    if data.get("error").is_some() {
        return Err(anyhow!("Error fetching data for ID: {}", id));
    }
    let book: OpenLibraryResponse = serde_json::from_value(data)?;
    if book.books.is_empty() {
        return Err(anyhow!("No book found for ID: {}", id));
    }
    let book_details = book.books.values().next().cloned().ok_or_else(|| anyhow!("No book details found"))?;
    println!("{:?}", book_details);
    Ok(book_details)
}

pub async fn get_olapi_search(name: &str) -> Result<OpenLibrarySearchResponse> {
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
    if data.get("error").is_some() {
        return Err(anyhow!("Error fetching data for name: {}", name));
    }
    let data: OpenLibrarySearchResponse = serde_json::from_value(data)?;
    if data.docs.is_empty() {
        return Err(anyhow!("No results found for name: {}", name));
    }
    println!("{:?}", data);
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


//########################################

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use tokio;

    #[tokio::test]
    async fn get_olapi_comics_by_id_returns_book_on_valid_id() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/api/books")
                .query_param("bibkeys", "OLID:OL12345")
                .query_param("jscmd", "details")
                .query_param("format", "json");
            then.status(200)
                .json_body_obj(&serde_json::json!({
                    "OLID:OL12345": {
                        "details": {
                            "title": "Test Book",
                            "description": "A test book",
                            "physical_format": "Paperback",
                            "number_of_pages": 123,
                            "publish_date": "2020",
                            "info_url": "http://example.com",
                            "authors": [{ "name": "Author Name" }]
                        },
                        "thumbnail_url": "http://example.com/thumb.jpg"
                    }
                }));
        });

        let url = format!("{}/api/books?bibkeys=OLID:OL12345&jscmd=details&format=json", server.base_url());
        let result = reqwest::get(&url).await.unwrap().json::<serde_json::Value>().await.unwrap();
        let book: OpenLibraryResponse = serde_json::from_value(result).unwrap();
        assert!(book.books.contains_key("OLID:OL12345"));
        mock.assert();
    }

    #[tokio::test]
    async fn get_olapi_comics_by_id_returns_error_on_missing_book() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET);
            then.status(200)
                .json_body_obj(&serde_json::json!({}));
        });

        let url = format!("{}/api/books?bibkeys=OLID:OL00000&jscmd=details&format=json", server.base_url());
        let result = reqwest::get(&url).await.unwrap().json::<serde_json::Value>().await.unwrap();
        let book: Result<OpenLibraryResponse, _> = serde_json::from_value(result);
        assert!(book.is_ok());
        assert!(book.unwrap().books.is_empty());
        mock.assert();
    }

    #[tokio::test]
    async fn get_olapi_search_returns_results_on_valid_name() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/search.json")
                .query_param("q", "TestBook");
            then.status(200)
                .json_body_obj(&serde_json::json!({
                    "start": 0,
                    "num_found": 1,
                    "docs": [{
                        "title": "TestBook",
                        "author_name": ["Author"],
                        "cover_i": 123,
                        "edition_count": 1,
                        "first_publish_year": 2020,
                        "key": "/works/OL12345W"
                    }]
                }));
        });

        let url = format!("{}/search.json?q=TestBook", server.base_url());
        let result = reqwest::get(&url).await.unwrap().json::<serde_json::Value>().await.unwrap();
        let search: OpenLibrarySearchResponse = serde_json::from_value(result).unwrap();
        assert_eq!(search.num_found, 1);
        assert!(!search.docs.is_empty());
        mock.assert();
    }

    #[tokio::test]
    async fn get_olapi_search_returns_error_on_empty_name() {
        let result = get_olapi_search("").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn get_olapi_book_returns_error_on_empty_key() {
        let result = get_olapi_book("").await;
        assert!(result.is_err());
    }
}