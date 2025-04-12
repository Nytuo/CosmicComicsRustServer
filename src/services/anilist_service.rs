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

        /* if let Some(staff_nodes) = base_object["staff"]["nodes"].as_array_mut() {
            for staff in staff_nodes {
                if let Some(staff_obj) = staff.as_object_mut() {
                    staff_obj.retain(|key, _| key == "id" || key == "name");
                    if let Some(name) = staff_obj.get_mut("name") {
                        if let Some(full_name) = name["full"].take() {
                            *name = full_name;
                        }
                    }
                }
            }
            base_object["staff"] = json!(staff_nodes);
        }

        if let Some(character_nodes) = base_object["characters"]["nodes"].as_array_mut() {
            for character in character_nodes {
                if let Some(character_obj) = character.as_object_mut() {
                    character_obj.retain(|key, _| key == "id" || key == "name");
                    if let Some(name) = character_obj.get_mut("name") {
                        if let Some(full_name) = name["full"].take() {
                            *name = full_name;
                        }
                    }
                }
            }
            base_object["characters"] = json!(character_nodes);
        }
         */
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
            Media(id: $id, type: MANGA) {
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

    if let Some(media) = json_response["data"]["Media"].as_object() {
        let base_object = json!(media);

        let mut result = HashMap::new();
        result.insert("base".to_string(), base_object);

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
        let original = r#"{
  "base": {
    "id": 36444,
    "title": {
      "romaji": "NARUTO",
      "english": null,
      "native": "NARUTO"
    },
    "status": "FINISHED",
    "startDate": {
      "year": 1997,
      "month": null,
      "day": null
    },
    "endDate": {
      "year": 1997,
      "month": null,
      "day": null
    },
    "description": "A one-shot manga by Kishimoto that was published in the August 1997 issue of Akamaru Jump which the now famous Naruto manga is based on. ",
    "meanScore": 59,
    "genres": [
      "Action"
    ],
    "coverImage": {
      "large": "https://s4.anilist.co/file/anilistcdn/media/manga/cover/medium/bx36444-ts0MBsP8sdY5.jpg"
    },
    "bannerImage": null,
    "trending": 0,
    "siteUrl": "https://anilist.co/manga/36444",
    "volumes": null,
    "chapters": 1,
    "staff": [
      {
        "id": 96879,
        "name": "Masashi Kishimoto"
      }
    ],
    "characters": [
      {
        "id": 17,
        "name": "Naruto Uzumaki"
      },
      {
        "id": 17312,
        "name": "Teuchi"
      }
    ]
  },
  "staff": [
    {
      "id": 96879,
      "name": {
        "full": "Masashi Kishimoto",
        "native": "岸本斉史"
      },
      "image": {
        "medium": "https://s4.anilist.co/file/anilistcdn/staff/medium/n96879-f6xMfzTXvLUn.png"
      },
      "description": "Masashi Kishimoto is a Japanese manga artist, well known for creating the manga series [Naruto](https://anilist.co/manga/30011/Naruto/). His younger twin brother, [Seishi Kishimoto](https://anilist.co/staff/96897/Seishi-Kishimoto), is also a manga artist and creator of the manga series [666 Satan](https://anilist.co/manga/30032/OParts-Hunter/) and [Blazer Drive](https://anilist.co/manga/36612/Blazer-Drive/). He is good friends with [Eiichirou Oda](https://anilist.co/staff/96881/Eiichirou-Oda)",
      "siteUrl": "https://anilist.co/staff/96879"
    }
  ],
  "characters": [
    {
      "id": 17,
      "name": {
        "full": "Naruto Uzumaki",
        "native": "うずまきナルト"
      },
      "image": {
        "medium": "https://s4.anilist.co/file/anilistcdn/character/medium/b17-phjcWCkRuIhu.png"
      },
      "description": "__Height__: 145-180 cm \n__Family:__ ~![Minato Namikaze](https://anilist.co/character/2535) (father), [Kushina Uzumaki](https://anilist.co/character/7302) (mother), [Jiraiya](https://anilist.co/character/2423) (godfather)  !~\n\nBorn in Konohagakure, a ninja village hidden in the leaves, Naruto Uzumaki was destined for greatness. When born, a powerful [nine-tailed demon fox](https://anilist.co/character/7407) attacked his village. With a wave of its tail, the demon fox could raise tsunamis and shatter mountains. In a valiant attempt to save the village from destruction, the Fourth Hokage and leader of the Hidden Leaf Village sealed the demon fox within Naruto's newborn body. This was his final act, for the battle with the fox cost him his life.  Despite the Fourth Hokage's dying wish that Naruto is viewed as a hero for serving as the container for the demon (a  _Jinchuuriki_), the adult villagers of Konoha harbored a fierce hatred for him, with many believing that Naruto and the demons were one and the same. Cast aside as an inhuman monster, Naruto was outcast and ostracised by the villagers for reasons he could not understand. The children his age could only ever follow their parents' example; and they too came to harbor a fierce hatred for Naruto.  Naruto eventually came to accept that he would live and die alone, and his external response was to perform harmless pranks on the village. Coy, raffish, and full of life, Naruto soon came to display a somewhat unexpected determination to succeed and be accepted by others. Upon being assigned to \"Team Seven\" as a Genin-ranked ninja, his true potential soon became outwardly apparent.  Vowing to become Hokage one day and using his will to never give in, Naruto saves the village from invading forces and earns his acceptance. Eventually, Naruto learns to harness the power of the Demon Fox sealed inside him to perform acts of strength far beyond what any other human is capable of.  In all, Naruto is an admirable character whose sheer determination to succeed despite the odds, earns him respect and devotion from his fellow villagers.",
      "siteUrl": "https://anilist.co/character/17"
    },
    {
      "id": 17312,
      "name": {
        "full": "Teuchi",
        "native": "テウチ"
      },
      "image": {
        "medium": "https://s4.anilist.co/file/anilistcdn/character/medium/17312.jpg"
      },
      "description": "__Height:__ 170 cm\n\nTeuchi is the owner of the Ichiraku Ramen Bar. His daughter is [Ayame](https://anilist.co/character/17310), and she works at his Ramen bar.",
      "siteUrl": "https://anilist.co/character/17312"
    }
  ],
  "relations": [
    {
      "id": 30011,
      "title": {
        "romaji": "NARUTO",
        "english": "Naruto",
        "native": "NARUTO -ナルト-"
      },
      "coverImage": {
        "large": "https://s4.anilist.co/file/anilistcdn/media/manga/cover/medium/nx30011-9yUF1dXWgDOx.jpg"
      },
      "type": "MANGA",
      "format": "MANGA",
      "relationType": "ALTERNATIVE"
    }
  ]
}"#;
        //compare data and original
        assert_eq!(serde_json::to_string_pretty(&data).unwrap(), original);
    }
}
