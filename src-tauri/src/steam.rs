use reqwest::{Client, StatusCode};
use serde_json::Value;
use serde::{Serialize, Deserialize};
use crate::app::ErrorType;
use serde_with::serde_as;
use serde_with::DisplayFromStr;

#[derive(Debug, Deserialize)]
pub struct ChangedDates {
    pub dates: Option<Vec<String>>,
    pub result_highwatermark: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DetailedSales {
    pub results: Option<Vec<CPartnerFinancialsDetailedSalesResult>>,
    pub key_request_info: Option<Vec<CPartnerFinancialsKeyRequestInfo>>,
    pub package_info: Option<Vec<CPartnerFinancialsPackageInfo>>,
    pub app_info: Option<Vec<CPartnerFinancialsAppInfo>>,
    pub bundle_info: Option<Vec<CPartnerFinancialsBundleInfo>>,
    pub discount_info: Option<Vec<CPartnerFinancialsDiscountInfo>>,
    pub combined_discount_info: Option<Vec<CPartnerFinancialsCombinedDiscountInfo>>,
    pub game_item_info: Option<Vec<CPartnerFinancialsGameItemInfo>>,
    pub country_info: Option<Vec<CPartnerFinancialsCountryInfo>>,
    pub partner_info: Option<Vec<CPartnerFinancialsPartnerInfo>>,
    pub max_id: String,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CPartnerFinancialsDetailedSalesResult {
    pub partnerid: i32,
    pub date: String,
    pub line_item_type: Option<String>,
    pub packageid: Option<i32>,
    pub bundleid: Option<i32>,
    pub appid: Option<i32>,
    pub game_item_id: Option<i32>,
    pub package_sale_type: Option<String>,
    pub key_request_id: Option<i32>,
    pub platform: Option<String>,
    pub country_code: Option<String>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub base_price: Option<i64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub sale_price: Option<i64>,
    pub currency: Option<String>,
    pub gross_units_sold: Option<i32>,
    pub gross_units_returned: Option<i32>,
    #[serde_as(deserialize_as = "Option<DisplayFromStr>")]
    pub gross_sales_usd: Option<f32>,
    #[serde_as(deserialize_as = "Option<DisplayFromStr>")]
    pub gross_returns_usd: Option<f32>,
    #[serde_as(deserialize_as = "Option<DisplayFromStr>")]
    pub net_tax_usd: Option<f32>,
    pub gross_units_activated: Option<i32>,
    pub view_grant_partnerid: Option<i32>,
    pub net_units_sold: Option<i32>,
    #[serde_as(deserialize_as = "Option<DisplayFromStr>")]
    pub net_sales_usd: Option<f32>,
    #[serde_as(deserialize_as = "Option<DisplayFromStr>")]
    pub avg_sale_price_usd: Option<f32>,
    pub combined_discount_id: Option<i32>,
    pub primary_appid: Option<i32>,
    pub additional_revenue_share_tier: Option<i32>,
    pub partner_name: Option<String>,
    pub package_name: Option<String>,
    pub bundle_name: Option<String>,
    pub app_name: Option<String>,
    pub game_item_description: Option<String>,
    pub game_item_category: Option<String>,
    pub key_request_notes: Option<String>,
    pub game_code_id: Option<i32>,
    pub game_code_description: Option<String>,
    pub territory_code_id: Option<i32>,
    pub territory_code_description: Option<String>,
    pub country_name: Option<String>,
    pub region: Option<String>,
    pub combined_discount_name: Option<String>,
    pub total_discount_percentage: Option<i32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CPartnerFinancialsDiscountInfo {
    pub discountid: i32,
    pub discount_description: Option<String>,
    pub discount_group: Option<String>,
    pub discount_percentage: Option<i32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CPartnerFinancialsCombinedDiscountInfo {
    pub combined_discount_id: i32,
    pub combined_discount_name: Option<String>,
    pub total_discount_percentage: Option<i32>,
    pub discount_ids: Option<Vec<i32>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CPartnerFinancialsKeyRequestInfo {
    pub key_request_id: i32,
    pub key_request_notes: Option<String>,
    pub game_code_id: Option<i32>,
    pub game_code_description: Option<String>,
    pub territory_code_id: Option<i32>,
    pub territory_code_description: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CPartnerFinancialsPackageInfo {
    pub packageid: i32,
    pub package_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CPartnerFinancialsAppInfo {
    pub appid: i32,
    pub app_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CPartnerFinancialsBundleInfo {
    pub bundleid: i32,
    pub bundle_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CPartnerFinancialsGameItemInfo {
    pub appid: i32,
    pub game_item_id: Option<i32>,
    pub game_item_description: Option<String>,
    pub game_item_category: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CPartnerFinancialsCountryInfo {
    pub country_code: Option<String>,
    pub country_name: Option<String>,
    pub region: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CPartnerFinancialsPartnerInfo {
    pub partnerid: i32,
    pub partner_name: Option<String>,
}


pub async fn get_changed_dates_for_partner(token: Option<String>, highwatermark: Option<String>) -> Result<ChangedDates, ErrorType> {
    let token = token.ok_or(ErrorType::BadToken("Token not found".into()))?;
    let highwatermark = highwatermark.unwrap_or(String::from("0"));
    let url = format!("https://partner.steam-api.com/IPartnerFinancialsService/GetChangedDatesForPartner/v001/?key={}&highwatermark={}", token, highwatermark);

    let client = Client::new();
    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| ErrorType::BadHttpRequest(e.to_string()))?;

    if response.status() == StatusCode::FORBIDDEN {
        return Err(ErrorType::BadToken("Access forbidden (403): Check steam key or permissions".into()));
    }

    if !response.status().is_success() {
        return Err(ErrorType::BadHttpRequest(format!("HTTP error: {}", response.status())));
    }

    let json_body: Value = response.json()
        .await
        .map_err(|e| ErrorType::BadFormatting(format!("Failed to parse JSON response: {}", e)))?;

    let json_results = json_body["response"].to_string();

    let response: ChangedDates = serde_json::from_str(&json_results)
        .map_err(|e| ErrorType::BadFormatting(format!("Failed to deserialize response: {}", e)))?;

    Ok(response)
}


pub async fn get_detailed_sales(token: Option<String>, date: &str, highwatermark_id: i64) -> Result<DetailedSales, ErrorType> {
    let token = token.ok_or(ErrorType::BadToken("Token not found".into()))?;
    let url = format!("https://partner.steam-api.com/IPartnerFinancialsService/GetDetailedSales/v001/?key={}&date={}&highwatermark_id={}", token, date, highwatermark_id);

    let client = Client::new();
    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| ErrorType::BadHttpRequest(e.to_string()))?;

    if response.status() == StatusCode::FORBIDDEN {
        return Err(ErrorType::BadToken("Access forbidden (403): Check steam key or permissions".into()));
    }

    if !response.status().is_success() {
        return Err(ErrorType::BadHttpRequest(format!("HTTP error: {}", response.status())));
    }

    let json_body: Value = response.json()
        .await
        .map_err(|e| ErrorType::BadFormatting(format!("Failed to parse JSON response: {}", e)))?;

    let json_results = json_body["response"].clone();

    let response: DetailedSales = serde_json::from_value(json_results)
        .map_err(|e| ErrorType::BadFormatting(format!("Failed to deserialize response: {}", e)))?;

    Ok(response)
}


pub async fn check_api_key(api_key: Option<String>) -> Result<String, ErrorType> {
    let api_key = api_key.ok_or(ErrorType::BadToken("API key not found".into()))?;
    let url = format!("https://partner.steam-api.com/IPartnerFinancialsService/GetDetailedSales/v001/?key={}", api_key);

    let client = Client::new();
    let response = client.head(&url)
        .send()
        .await
        .map_err(|e| ErrorType::BadHttpRequest(e.to_string()))?;

    if response.status() == StatusCode::FORBIDDEN {
        return Err(ErrorType::BadToken("Access forbidden (403): Check steam key or permissions".into()));
    }

    if !response.status().is_success() {
        return Err(ErrorType::BadHttpRequest(format!("HTTP error: {}", response.status())));
    }

    Ok("ok".to_string())
}