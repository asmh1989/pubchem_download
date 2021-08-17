use bson::DateTime;
use log::info;
use mongodb::bson;
use serde::{Deserialize, Serialize};

use crate::{
    db::{Db, COLLECTION_CID},
    filter_cid,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct PubChemNotFound {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<bson::oid::ObjectId>,
    pub cid: String,
    // #[serde(serialize_with = "bson_datetime_as_iso_string")]
    pub create_time: DateTime,
    pub update_time: DateTime,
}

impl PubChemNotFound {
    pub fn new(cid: &str) -> Self {
        let date = DateTime::now();

        Self {
            cid: cid.to_string(),
            create_time: date.clone(),
            update_time: date,
            id: None,
        }
    }

    pub fn save_db(&self) -> Result<(), String> {
        let doc = match bson::to_bson(&self) {
            Ok(d) => d.as_document().unwrap().clone(),
            Err(e) => {
                info!("to_bson err {}", e);
                return Err(format!("to_bson error : {}", e));
            }
        };

        if let Err(e) = Db::save(COLLECTION_CID, filter_cid!(self.cid.clone()), doc.clone()) {
            info!("db save error{} ", e);
            return Err(format!("db save error{} ", e));
        }
        Ok(())
    }
}
