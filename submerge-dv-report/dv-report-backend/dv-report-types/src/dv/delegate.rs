use crate::dv::delegation::Delegation;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Delegate {
    pub id: String,
    pub type_id: u32,
    pub name: String,
    pub short_name: String,
    pub url: Option<String>,
    pub twitter: Option<String>,
    pub delegations: Vec<Delegation>,
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct DelegateTypeRow {
    pub id: i32,
    pub name: String,
    pub code: String,
}

#[derive(Clone, Debug, FromRow)]
pub struct DelegateRow {
    pub id: String,
    pub type_id: i32,
    pub name: String,
    pub short_name: String,
    pub url: Option<String>,
    pub twitter: Option<String>,
}

impl DelegateRow {
    pub fn into_delegate(self, delegations: Vec<Delegation>) -> Delegate {
        Delegate {
            id: self.id.clone(),
            type_id: self.type_id as u32,
            name: self.name.clone(),
            short_name: self.short_name.clone(),
            url: self.url.clone(),
            twitter: self.twitter,
            delegations,
        }
    }
}
