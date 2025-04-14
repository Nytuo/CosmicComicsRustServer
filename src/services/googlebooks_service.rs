use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Volume {
    pub kind: String,
    pub id: String,
    pub etag: String,
    pub selfLink: String,
    pub volumeInfo: VolumeInfo,
    pub saleInfo: Option<SaleInfo>,
    pub accessInfo: Option<AccessInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VolumeInfo {
    pub title: String,
    pub authors: Option<Vec<String>>,
    pub publisher: Option<String>,
    pub publishedDate: Option<String>,
    pub description: Option<String>,
    pub industryIdentifiers: Option<Vec<IndustryIdentifier>>,
    pub pageCount: Option<u32>,
    pub dimensions: Option<Dimensions>,
    pub printType: Option<String>,
    pub mainCategory: Option<String>,
    pub categories: Option<Vec<String>>,
    pub averageRating: Option<f32>,
    pub ratingsCount: Option<u32>,
    pub contentVersion: Option<String>,
    pub imageLinks: Option<ImageLinks>,
    pub language: Option<String>,
    pub infoLink: Option<String>,
    pub canonicalVolumeLink: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndustryIdentifier {
    pub r#type: String,
    pub identifier: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dimensions {
    pub height: Option<String>,
    pub width: Option<String>,
    pub thickness: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageLinks {
    pub smallThumbnail: Option<String>,
    pub thumbnail: Option<String>,
    pub small: Option<String>,
    pub medium: Option<String>,
    pub large: Option<String>,
    pub extraLarge: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SaleInfo {
    pub country: String,
    pub saleability: String,
    pub isEbook: bool,
    pub listPrice: Option<Price>,
    pub retailPrice: Option<Price>,
    pub buyLink: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Price {
    pub amount: f32,
    pub currencyCode: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessInfo {
    pub country: String,
    pub viewability: String,
    pub embeddable: bool,
    pub publicDomain: bool,
    pub textToSpeechPermission: String,
    pub epub: Option<FormatAvailability>,
    pub pdf: Option<FormatAvailability>,
    pub accessViewStatus: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FormatAvailability {
    pub isAvailable: bool,
    pub acsTokenLink: Option<String>,
}

pub async fn get_gbapi_comics_by_id(id: &str) -> Result<Volume> {
    let url = format!("https://www.googleapis.com/books/v1/volumes/{}", id);
    println!("{}", url);

    let response = reqwest::get(&url).await?;
    let data = response.json::<Value>().await?;
    if data.get("error").is_some() {
        return Err(anyhow!("Error fetching data for ID: {}", id));
    }
    let volume: Volume = serde_json::from_value(data)?;
    println!("{:?}", volume);
    Ok(volume)
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
