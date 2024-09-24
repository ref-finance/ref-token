use crate::*;

use near_sdk::serde::Serialize;
use near_sdk::serde_json::json;

const EVENT_STANDARD: &str = "nep141";
const EVENT_STANDARD_VERSION: &str = "1.0.0";

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Event<'a> {
    FtMint {
        owner_id: &'a AccountId,
        amount: U128,
        #[serde(skip_serializing_if = "Option::is_none")]
        memo: Option<&'a str>,
    },
    FtBurn {
        owner_id: &'a AccountId,
        amount: U128,
        #[serde(skip_serializing_if = "Option::is_none")]
        memo: Option<&'a str>,
    }
}

impl Event<'_> {
    pub fn emit(&self) {
        emit_event(&self);
    }
}

pub(crate) fn emit_event<T: ?Sized + Serialize>(data: &T) {
    let result = json!(data);
    let event_json = json!({
        "standard": EVENT_STANDARD,
        "version": EVENT_STANDARD_VERSION,
        "event": result["event"],
        "data": [result["data"]]
    })
    .to_string();
    near_sdk::log!("EVENT_JSON:{}", event_json);
}
