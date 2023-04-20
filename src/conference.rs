use actix_web::{
    get, post,
    web::{Data, Json, Query},
    HttpResponse,
};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

#[derive(Deserialize, Serialize)]
pub struct Conference {
    #[serde(skip_deserializing, skip_serializing_if = "String::is_empty")]
    pub message: String,
    /// ID to search for existing conference mapping. Only used when provided alone (search by ID)
    #[serde(default)]
    pub id: u64,
    /// Full JID (room@conference.server.domain) for the conference to create or return existing conference mapping.
    /// Used preferentially over all other input parameters (search by conference)
    #[serde(default)]
    pub conference: String,
}

impl Default for Conference {
    fn default() -> Self {
        Self {
            message: "".to_owned(),
            id: 0,
            conference: "".to_owned(),
        }
    }
}

#[derive(Deserialize)]
pub struct ConferenceParams {
    pub id: Option<u64>,
    pub conference: Option<String>,
}

fn conference_hash(conf: &str, digits: u32) -> u64 {
    let mut s = DefaultHasher::new();

    loop {
        // Lazily hash until we have exactly `digits` digits
        conf.hash(&mut s);
        let hash = s.finish() % u64::pow(10, digits);
        if hash.to_string().len() == (digits as usize) {
            break hash;
        }
    }
}

fn insert_conference(db: Data<Db>, try_id: u64, try_conf: &str, digits: u32) -> Conference {
    let mut conference = Conference::default();

    match (try_id, try_conf.is_empty()) {
        (id, true) if id > 0 => {
            conference.id = id;
            match db.get(id.to_string()) {
                Err(e) => conference.message = e.to_string(),
                Ok(None) => conference.message = "No conference mapping was found".to_string(),
                Ok(Some(jid)) => {
                    conference.message = "Successfully retrieved conference mapping".to_string();
                    conference.conference = std::str::from_utf8(&jid).unwrap().to_string();
                }
            }
        }
        (_, false) => {
            let mut conf_name = try_conf.to_lowercase();
            // URL encode, but preserve "@"
            conf_name = urlencoding::encode(&conf_name).to_string();
            conf_name = conf_name.replace("%40", "@");
            conference.conference = conf_name.clone();

            // Generate a hash
            let id = conference_hash(&conf_name, digits);
            match db.insert(id.to_string(), conference.conference.as_bytes()) {
                Ok(_) => {
                    conference.id = id;
                    conference.message = "Successfully created conference mapping".to_string();
                }
                Err(_) => conference.message = "".to_string(),
            }
        }
        (_, _) => conference.message = "No conference or id provided".to_owned(),
    };
    conference
}

/// When called with a conference, creates a new ID and both stores and returns the result.
/// When called with an ID (only), returns the mapping if previously created.
#[get("/conferenceMapper")]
pub async fn get(
    query: Query<ConferenceParams>,
    db: Data<Db>,
    id_length: Data<u32>,
) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .insert_header(("access-control-allow-origin", "*"))
        .json(insert_conference(
            db,
            query.id.unwrap_or(0),
            &query.conference.to_owned().unwrap_or("".to_string()),
            **id_length,
        ))
}

/// When called with a conference, creates a new ID and both stores and returns the result. When called with an ID, returns the mapping if previously created.
#[post("/conferenceMapper")]
pub async fn set(conference: Json<Conference>, db: Data<Db>, id_length: Data<u32>) -> HttpResponse {
    HttpResponse::Created()
        .content_type("application/json")
        .insert_header(("access-control-allow-origin", "*"))
        .json(insert_conference(
            db,
            conference.id,
            &conference.conference,
            **id_length,
        ))
}

#[cfg(test)]
mod test {

    use super::*;
    use rand::distributions::{Alphanumeric, DistString};

    #[test]
    fn test_generate_hash() {
        // Test 20 times for better collision checking
        for _ in 0..20 {
            // Test all strings from {1..100} for {6..13} digit hashes
            for digits in 6..13 {
                // Hashes are only going to be unique for a given digit count
                let mut known_hashes: Vec<u64> = Vec::new();

                for len in 1..100 {
                    // Value is always <sample>@<muc domain>
                    let value = format!(
                        "{}@muc.jitsi.meet",
                        Alphanumeric.sample_string(&mut rand::thread_rng(), len)
                    );
                    let hash = conference_hash(&value, digits);

                    assert!(hash >= u64::pow(10, digits - 1));
                    assert!(hash < u64::pow(10, digits));

                    assert!(!known_hashes.contains(&hash));
                    known_hashes.push(hash);
                }
            }
        }
    }
}
