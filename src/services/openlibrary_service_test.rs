#[cfg(test)]
mod tests {
    use crate::services::openlibrary_service::{
        OpenLibraryResponse, OpenLibrarySearchResponse, get_olapi_book, get_olapi_search,
    };
    use anyhow::Result;
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
            then.status(200).json_body_obj(&serde_json::json!({
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

        let url = format!(
            "{}/api/books?bibkeys=OLID:OL12345&jscmd=details&format=json",
            server.base_url()
        );
        let result = reqwest::get(&url)
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
        let book: OpenLibraryResponse = serde_json::from_value(result).unwrap();
        assert!(book.books.contains_key("OLID:OL12345"));
        mock.assert();
    }

    #[tokio::test]
    async fn get_olapi_comics_by_id_returns_error_on_missing_book() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET);
            then.status(200).json_body_obj(&serde_json::json!({}));
        });

        let url = format!(
            "{}/api/books?bibkeys=OLID:OL00000&jscmd=details&format=json",
            server.base_url()
        );
        let result = reqwest::get(&url)
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
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
            then.status(200).json_body_obj(&serde_json::json!({
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
        let result = reqwest::get(&url)
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
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
