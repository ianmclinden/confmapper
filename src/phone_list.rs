use actix_web::{get, web::Data, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PhoneNumber {
    #[serde(rename = "countryCode")]
    pub country_code: String,
    #[serde(rename = "tollFree", default)]
    pub toll_free: bool,
    #[serde(rename = "formattedNumber")]
    pub formatted_number: String,
}

impl Default for PhoneNumber {
    fn default() -> Self {
        Self {
            country_code: "".to_owned(),
            toll_free: false,
            formatted_number: "".to_owned(),
        }
    }
}

#[get("/phoneNumberList")]
pub async fn get(phone_list: Data<Vec<PhoneNumber>>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .json(phone_list)
}
