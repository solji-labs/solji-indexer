use chrono::NaiveDateTime;
use serde::{Deserialize, Deserializer, Serialize};
const FMT: &str = "%Y-%m-%d %H:%M:%S";

pub fn ndt_opt_from_str<'de, D>(d: D) -> Result<Option<NaiveDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(d)?;
    match opt {
        None => Ok(None),
        Some(s) => {
            let s = s.trim();
            if s.is_empty() {
                return Ok(None);
            }
            NaiveDateTime::parse_from_str(s, FMT)
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
    }
}

fn empty_string_none<'de, D>(d: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(d)?;
    Ok(opt.and_then(|s| {
        let s = s.trim();
        if s.is_empty() {
            None
        } else {
            Some(s.to_string())
        }
    }))
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageReq {
    pub page_num: u32,
    pub page_size: u32,
    #[serde(default, deserialize_with = "empty_string_none")]
    pub pubkey: Option<String>,
    #[serde(deserialize_with = "ndt_opt_from_str")]
    pub start_time: Option<NaiveDateTime>,
    #[serde(deserialize_with = "ndt_opt_from_str")]
    pub end_time: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeReq {
    #[serde(deserialize_with = "ndt_opt_from_str")]
    pub start_time: Option<NaiveDateTime>,
    #[serde(deserialize_with = "ndt_opt_from_str")]
    pub end_time: Option<NaiveDateTime>,
}
