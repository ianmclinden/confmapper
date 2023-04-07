use std::collections::HashMap;

use actix_web::{get, web::Data, HttpResponse};
use serde::{Deserialize, Serialize};

pub type PhoneNumbers = HashMap<String, Vec<String>>;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PhoneNumberList {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub message: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub numbers: PhoneNumbers,
    #[serde(rename = "numbersEnabled")]
    pub numbers_enabled: bool,
}

#[get("/phoneNumberList")]
pub async fn get(phone_list: Data<PhoneNumbers>) -> HttpResponse {
    let mut list = PhoneNumberList {
        message: if !phone_list.is_empty() {
            "Phone numbers available.".to_string()
        } else {
            "".to_string()
        },
        numbers_enabled: !phone_list.is_empty(),
        numbers: PhoneNumbers::new(),
    };
    for number in phone_list.iter() {
        list.numbers
            .insert(number.0.to_owned(), number.1.to_owned());
    }
    HttpResponse::Ok()
        .content_type("application/json")
        .json(list)
}
