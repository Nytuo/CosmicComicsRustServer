use reqwest::Client;
use serde_json::{Value, json};
use std::collections::HashMap;

pub async fn api_anilist_get(name: &str) -> Result<Option<HashMap<String, Value>>, reqwest::Error> {
    let query = r#"
        query ($page: Int, $perPage: Int, $search: String) {
            Page(page: $page, perPage: $perPage) {
                pageInfo {
                    total
                }
                media(type: MANGA, search: $search) {
                    id
                    title {
                        romaji
                        english
                        native
                    }
                    status
                    startDate {
                        year
                        month
                        day
                    }
                    endDate {
                        year
                        month
                        day
                    }
                    description
                    meanScore
                    genres
                    coverImage {
                        large
                    }
                    bannerImage
                    trending
                    siteUrl
                    volumes
                    chapters
                    staff {
                        nodes {
                            id
                            name {
                                full
                                native
                            }
                            image {
                                medium
                            }
                            description
                            siteUrl
                        }
                        edges {
                            role
                        }
                    }
                    characters {
                        nodes {
                            id
                            name {
                                full
                                native
                            }
                            image {
                                medium
                            }
                            description
                            siteUrl
                        }
                        edges {
                            role
                        }
                    }
                    relations {
                        nodes {
                            id
                            title {
                                romaji
                                english
                                native
                            }
                            coverImage {
                                large
                            }
                            type
                            format
                        }
                        edges {
                            relationType
                        }
                    }
                }
            }
        }
    "#;

    let variables = json!({
        "search": name,
        "page": 1,
        "perPage": 5
    });

    let url = "https://graphql.anilist.co";
    let client = Client::new();
    let response = client
        .post(url)
        .json(&json!({
            "query": query,
            "variables": variables
        }))
        .send()
        .await?;

    let json_response: Value = response.json().await?;

    if let Some(media) = json_response["data"]["Page"]["media"].as_array() {
        if media.is_empty() {
            return Ok(None);
        }

        let mut base_object = media[0].clone();
        let staff_object = base_object["staff"]["nodes"].clone();
        let characters_object = base_object["characters"]["nodes"].clone();
        let relations_nodes = base_object["relations"]["nodes"].clone();
        let relations_edges = base_object["relations"]["edges"].clone();

        let mut relations_object = Vec::new();
        if let (Some(nodes), Some(edges)) = (relations_nodes.as_array(), relations_edges.as_array())
        {
            for (i, node) in nodes.iter().enumerate() {
                let mut relation = node.clone();
                if let Some(relation_type) =
                    edges.get(i).and_then(|edge| edge["relationType"].as_str())
                {
                    relation["relationType"] = json!(relation_type);
                }
                relations_object.push(relation);
            }
        }

        base_object.as_object_mut().unwrap().remove("relations");

        if let Some(staff_nodes) = base_object["staff"]["nodes"].as_array_mut() {
            let mod_staff_nodes: Vec<Value> = staff_nodes
                .iter()
                .map(|staff| {
                    let mut new_staff = staff.clone();
                    new_staff
                        .as_object_mut()
                        .unwrap()
                        .retain(|key, _| key == "id" || key == "name");
                    new_staff
                })
                .collect();
            base_object["staff"] = json!(mod_staff_nodes);
        }

        if let Some(character_nodes) = base_object["characters"]["nodes"].as_array_mut() {
            let mod_character_nodes: Vec<Value> = character_nodes
                .iter()
                .map(|character| {
                    let mut new_character = character.clone();
                    new_character
                        .as_object_mut()
                        .unwrap()
                        .retain(|key, _| key == "id" || key == "name");
                    new_character
                })
                .collect();
            base_object["characters"] = json!(mod_character_nodes);
        }

        let mut result = HashMap::new();
        result.insert("base".to_string(), base_object);
        result.insert("staff".to_string(), staff_object);
        result.insert("characters".to_string(), characters_object);
        result.insert("relations".to_string(), json!(relations_object));

        return Ok(Some(result));
    }

    Ok(None)
}

pub async fn api_anilist_get_search(
    name: &str,
) -> Result<Option<HashMap<String, Value>>, reqwest::Error> {
    let query = r#"
        query ($page: Int, $perPage: Int, $search: String) {
            Page(page: $page, perPage: $perPage) {
                pageInfo {
                    total
                }
                media(type: MANGA, search: $search) {
                    id
                    title {
                        romaji
                        english
                        native
                    }
                    coverImage {
                        large
                    }
                }
            }
        }
    "#;

    let variables = json!({
        "search": name,
        "page": 1,
        "perPage": 20
    });

    let url = "https://graphql.anilist.co";
    let client = Client::new();
    let response = client
        .post(url)
        .json(&json!({
            "query": query,
            "variables": variables
        }))
        .send()
        .await?;

    let json_response: Value = response.json().await?;

    if let Some(media) = json_response["data"]["Page"]["media"].as_array() {
        if media.is_empty() {
            return Ok(None);
        }

        // Clone the media array to create the base object
        let base_object = json!(media);

        let mut result = HashMap::new();
        result.insert("base".to_string(), base_object);

        return Ok(Some(result));
    }

    Ok(None)
}

pub async fn api_anilist_get_by_id(
    id: &str,
) -> Result<Option<HashMap<String, Value>>, reqwest::Error> {
    let query = r#"
        query ($id: Int) {
            Media(type: MANGA, id: $id) {
                    id
                    title {
                        romaji
                        english
                        native
                    }
                    status
                    startDate {
                        year
                        month
                        day
                    }
                    endDate {
                        year
                        month
                        day
                    }
                    description
                    meanScore
                    genres
                    coverImage {
                        large
                    }
                    bannerImage
                    trending
                    siteUrl
                    volumes
                    chapters
                    staff {
                        nodes {
                            id
                            name {
                                full
                                native
                            }
                            image {
                                medium
                            }
                            description
                            siteUrl
                        }
                        edges {
                            role
                        }
                    }
                    characters {
                        nodes {
                            id
                            name {
                                full
                                native
                            }
                            image {
                                medium
                            }
                            description
                            siteUrl
                        }
                        edges {
                            role
                        }
                    }
                    relations {
                        nodes {
                            id
                            title {
                                romaji
                                english
                                native
                            }
                            coverImage {
                                large
                            }
                            type
                            format
                        }
                        edges {
                            relationType
                        }
                    }
                }
        }
    "#;

    let variables = json!({
        "id": id.parse::<i32>().unwrap_or(0)
    });

    let url = "https://graphql.anilist.co";
    let client = Client::new();
    let response = client
        .post(url)
        .json(&json!({
            "query": query,
            "variables": variables
        }))
        .send()
        .await?;

    let json_response: Value = response.json().await?;

    println!("Response: {:?}", json_response);

    if let Some(media) = json_response["data"]["Media"].as_object() {
        if media.is_empty() {
            return Ok(None);
        }

        let mut base_object = media.clone();
        let staff_object = base_object["staff"]["nodes"].clone();
        let characters_object = base_object["characters"]["nodes"].clone();
        let relations_nodes = base_object["relations"]["nodes"].clone();
        let relations_edges = base_object["relations"]["edges"].clone();

        let mut relations_object = Vec::new();
        if let (Some(nodes), Some(edges)) = (relations_nodes.as_array(), relations_edges.as_array())
        {
            for (i, node) in nodes.iter().enumerate() {
                let mut relation = node.clone();
                if let Some(relation_type) =
                    edges.get(i).and_then(|edge| edge["relationType"].as_str())
                {
                    relation["relationType"] = json!(relation_type);
                }
                relations_object.push(relation);
            }
        }
        if let Some(staff_nodes) = base_object["staff"]["nodes"].as_array_mut() {
            let mod_staff_nodes: Vec<Value> = staff_nodes
                .iter()
                .map(|staff| {
                    let mut new_staff = staff.clone();
                    new_staff
                        .as_object_mut()
                        .unwrap()
                        .retain(|key, _| key == "id" || key == "name");
                    new_staff
                })
                .collect();
            base_object["staff"] = json!(mod_staff_nodes);
        }

        if let Some(character_nodes) = base_object["characters"]["nodes"].as_array_mut() {
            let mod_character_nodes: Vec<Value> = character_nodes
                .iter()
                .map(|character| {
                    let mut new_character = character.clone();
                    new_character
                        .as_object_mut()
                        .unwrap()
                        .retain(|key, _| key == "id" || key == "name");
                    new_character
                })
                .collect();
            base_object["characters"] = json!(mod_character_nodes);
        }
        base_object.remove("relations");

        let mut result: HashMap<String, Value> = HashMap::new();
        result.insert("base".to_string(), json!(base_object));
        result.insert("staff".to_string(), staff_object);
        result.insert("characters".to_string(), characters_object);
        result.insert("relations".to_string(), json!(relations_object));

        return Ok(Some(result));
    }

    Ok(None)
}

//##########################################

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn api_anilist_get_test() {
        let name = "Naruto";
        let result = api_anilist_get(name).await;

        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(data.is_some());

        let data = data.unwrap();
        assert!(data.contains_key("base"));
        assert!(data.contains_key("staff"));
        assert!(data.contains_key("characters"));
        assert!(data.contains_key("relations"));
        assert_ne!(data["staff"], data["base"]["staff"]);
        assert_ne!(data["characters"], data["base"]["characters"]);
    }

    #[tokio::test]
    async fn api_anilist_get_by_id_test() {
        let id = "30011";
        let result = api_anilist_get_by_id(id).await;

        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(data.is_some());

        let data = data.unwrap();
        assert!(data.contains_key("base"));
        assert!(data.contains_key("staff"));
        assert!(data.contains_key("characters"));
        assert!(data.contains_key("relations"));
        assert_ne!(data["staff"], data["base"]["staff"]);
        assert_ne!(data["characters"], data["base"]["characters"]);
    }

    #[tokio::test]
    async fn api_anilist_get_search_test() {
        let name = "Naruto";
        let result = api_anilist_get_search(name).await;

        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(data.is_some());

        let data = data.unwrap();
        assert!(data.contains_key("base"));
    }
}
