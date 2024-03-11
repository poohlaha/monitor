//!
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use serde::de::DeserializeOwned;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub(crate) code: u16,
    pub(crate) body: serde_json::Value,
    pub(crate) error: String,
}


/// 获取成功 response
pub(crate) fn get_success_response(body: Option<serde_json::Value>) -> HttpResponse {
    let mut data = serde_json::Value::String(String::new());
    if let Some(body) = body {
        data = body
    }
    HttpResponse { code: 200, body: data, error: String::new() }
}

/// 获取失败 response
pub(crate) fn get_error_response(error: &str) -> HttpResponse {
    HttpResponse {
        code: 500,
        body: serde_json::Value::String(String::new()),
        error: String::from(error),
    }
}

/// 转成 String
pub(crate) fn to_result(response: HttpResponse) -> String {
    return serde_json::to_string(&response).unwrap_or(String::new());
}

/// 转换
#[allow(dead_code)]
pub(crate) fn convert_response<T>(response: HttpResponse) -> T
    where
        T: Default + Serialize + DeserializeOwned + 'static
{
    let data: Result<T, serde_json::Error> = serde_json::from_value(response.body);
    return match data {
        Ok(data) => {
            data
        },
        Err(_) => {
            // println!("convert response to T error: {:#?}", err);
            T::default()
        }
    }
}


