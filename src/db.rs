use std::{sync::Arc, time::Duration};

use log::info;

use mongodb::{
    bson::{doc, Bson, Document},
    error::Error,
    options::{ClientOptions, FindOneOptions},
    sync::Client,
};
use once_cell::sync::OnceCell;

static INSTANCE: OnceCell<Arc<Client>> = OnceCell::new();

const TABLE_NAME: &'static str = "pub_chem";
pub const COLLECTION_CID_NOT_FOUND: &'static str = "cid_not_found";
pub const COLLECTION_FILTER_SMILES_SOLUBILITY: &'static str = "filter_smiles_solubility";

const KEY_UPDATE_TIME: &'static str = "updateTime";
const KEY_CREATE_TIME: &'static str = "createTime";

#[macro_export]
macro_rules! filter_cid {
    ($e:expr) => {
        mongodb::bson::doc! {"cid" : $e}
    };
}

#[derive(Clone, Debug)]
pub struct Db;

impl Db {
    pub fn get_instance() -> &'static Arc<Client> {
        INSTANCE.get().expect("db need init first")
    }

    // pub fn find<T>(
    //     table: &str,
    //     filter: impl Into<Option<Document>>,
    //     options: impl Into<Option<FindOptions>>,
    //     call_back: &dyn Fn(T),
    // ) -> Result<(), Error>
    // where
    //     T: DeserializeOwned,
    // {
    //     let client = Db::get_instance();
    //     let db = client.database(TABLE_NAME);
    //     let collection = db.collection::<Document>(table);

    //     let mut cursor = collection.find(filter, options)?;

    //     // Iterate over the results of the cursor.
    //     while let Some(result) = cursor.next() {
    //         match result {
    //             Ok(document) => {
    //                 let result = bson::from_bson::<T>(Bson::Document(document));
    //                 match result {
    //                     Ok(app) => call_back(app),
    //                     Err(err) => {
    //                         info!("err = {:?}", err);
    //                     }
    //                 }
    //             }
    //             Err(e) => {
    //                 info!("error = {:?}", e);
    //                 return Err(e.into());
    //             }
    //         }
    //     }

    //     Ok(())
    // }

    pub fn find_one(
        table: &str,
        filter: impl Into<Option<Document>>,
        options: impl Into<Option<FindOneOptions>>,
    ) -> Result<Option<Document>, Error> {
        let client = Db::get_instance();
        let db = client.database(TABLE_NAME);
        let collection = db.collection(table);

        collection.find_one(filter, options)
    }

    pub fn save(table: &str, filter: Document, app: Document) -> Result<(), Error> {
        let client = Db::get_instance();
        let db = client.database(TABLE_NAME);
        let collection = db.collection(table);

        let mut update_doc = app;
        let date = Bson::DateTime(chrono::Utc::now());
        update_doc.insert(KEY_UPDATE_TIME, date.clone());

        let result = collection.find_one(filter.clone(), None)?;

        if let Some(_) = result {
            // info!("db update");
            collection.update_one(filter.clone(), doc! {"$set": update_doc}, None)?;
        } else {
            update_doc.insert(KEY_CREATE_TIME, date);
            let result = collection.insert_one(update_doc, None)?;

            info!("db insert {:?}", result);
        }

        Ok(())
    }

    pub fn delete(table: &str, filter: Document) -> Result<(), Error> {
        let client = Db::get_instance();
        let db = client.database(TABLE_NAME);
        let collection = db.collection(table);

        let result = collection.delete_one(filter, None)?;

        info!("db delete {:?}", result);

        Ok(())
    }

    pub fn contians(table: &str, filter: Document) -> bool {
        let client = Db::get_instance();
        let db = client.database(TABLE_NAME);
        let collection = db.collection(table);

        let result = collection.find_one(filter, None);

        match result {
            Ok(d) => d.is_some(),
            Err(_) => false,
        }
    }
}

pub fn init_db(url: &str) {
    if INSTANCE.get().is_some() {
        return;
    }
    let mut client_options = ClientOptions::parse(url).unwrap();
    client_options.connect_timeout = Some(Duration::new(4, 0));
    // 选择超时
    client_options.server_selection_timeout = Some(Duration::new(8, 0));

    INSTANCE
        .set(Arc::new(Client::with_options(client_options).unwrap()))
        .expect("db init error");
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_init() {
        crate::config::init_config();

        super::init_db("mongodb://192.168.2.25:27017");

        let data = crate::model::PubChemNotFound::new("test");

        assert!(data.save_db().is_ok());

        assert!(data.save_db().is_ok());

        // let filter = doc! {};
        // let find_options = FindOneOptions::builder()
        //     .sort(doc! { "create_time": -1 })
        //     .build();

        assert!(Db::contians(COLLECTION_CID_NOT_FOUND, filter_cid!("test")));

        assert!(Db::delete(COLLECTION_CID_NOT_FOUND, filter_cid!("test")).is_ok());

        assert!(!Db::contians(COLLECTION_CID_NOT_FOUND, filter_cid!("test")));
    }
}
