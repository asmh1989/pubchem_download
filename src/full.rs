use log::info;
use mongodb::{
    bson::{self, doc, Bson, Document},
    options::FindOptions,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{
    chem::{Chem, StringWithMarkup},
    db::Db,
    download, filter_cid,
};

const DB_TABLE: &'static str = "szdata";
const DB_COLLECT: &'static str = "molecular";

const SOURCE: &'static str = "PubChem";
const STEP: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties {
    pub kind: String,
    pub description: String,
    pub value: Vec<StringWithMarkup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SZData {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<bson::oid::ObjectId>,
    pub cid: i64,
    pub smiles: String,
    pub inchi: String,
    pub inchi_key: String,
    pub cas: String,
    pub molecular_weight: String,
    pub properties: Vec<Properties>,
    pub names: Vec<String>,
    pub source: String,
}

impl SZData {
    pub fn new(
        cid: i64,
        smiles: String,
        molecular_weight: String,
        inchi: String,
        properties: Vec<Properties>,
        cas: String,
        inchi_key: String,
        names: Vec<String>,
    ) -> Self {
        Self {
            id: None,
            cid,
            smiles,
            molecular_weight,
            inchi_key,
            inchi,
            cas,
            properties,
            source: SOURCE.to_string(),
            names,
        }
    }

    pub fn document(&self) -> Result<Document, String> {
        match bson::to_bson(&self) {
            Ok(d) => return Ok(d.as_document().unwrap().clone()),
            Err(e) => {
                info!("to_bson err {}", e);
                return Err(format!("to_bson error : {}", e));
            }
        };
    }

    pub fn save_db(&self) -> Result<(), String> {
        let doc = match bson::to_bson(&self) {
            Ok(d) => d.as_document().unwrap().clone(),
            Err(e) => {
                info!("to_bson err {}", e);
                return Err(format!("to_bson error : {}", e));
            }
        };

        if let Err(e) = Db::save_with_table(
            DB_TABLE,
            DB_COLLECT,
            filter_cid!(self.cid.clone()),
            doc.clone(),
        ) {
            info!("db save error {} ", e);
            return Err(format!("db save error {} ", e));
        }
        Ok(())
    }

    pub fn insert_db(&self) -> Result<(), String> {
        let doc = match bson::to_bson(&self) {
            Ok(d) => d.as_document().unwrap().clone(),
            Err(e) => {
                info!("to_bson err {}", e);
                return Err(format!("to_bson error : {}", e));
            }
        };

        if let Err(e) = Db::insert_with_table(DB_TABLE, DB_COLLECT, doc.clone()) {
            info!("db save error {} ", e);
            return Err(format!("db save error {} ", e));
        }
        Ok(())
    }
}

fn parse_chem(chem: &Chem, update: bool) {
    let cid = chem.record.record_number;
    let mut properties: Vec<Properties> = Vec::new();
    let mut cas = "".to_string();
    let mut molecular_weight = "".to_string();
    let mut canonical_smiles = "".to_string();
    let mut isomeric_smiles = "".to_string();
    let mut inchi = "".to_string();
    let mut inchi_key = "".to_string();
    let mut names = Vec::new();
    chem.record
        .section
        .iter()
        .for_each(|s| match &s.tocheading[..] {
            "Names and Identifiers" => {
                s.section.iter().for_each(|s2| match &s2.tocheading[..] {
                    "Computed Descriptors" => {
                        s2.section.iter().for_each(|s3| match &s3.tocheading[..] {
                            "Canonical SMILES" => {
                                if !s3.information.is_empty()
                                    && !s3.information[0].value.string_with_markup.is_empty()
                                {
                                    canonical_smiles = s3.information[0].value.string_with_markup
                                        [0]
                                    .string
                                    .clone();
                                }
                            }
                            "Isomeric SMILES" => {
                                isomeric_smiles =
                                    s3.information[0].value.string_with_markup[0].string.clone();
                            }
                            "InChI" => {
                                inchi =
                                    s3.information[0].value.string_with_markup[0].string.clone();
                            }
                            "InChI Key" => {
                                inchi_key =
                                    s3.information[0].value.string_with_markup[0].string.clone();
                            }
                            _ => {}
                        });
                    }
                    "Other Identifiers" => {
                        s2.section.iter().for_each(|s3| match &s3.tocheading[..] {
                            "CAS" => {
                                cas = s3.information[0].value.string_with_markup[0].string.clone();
                            }
                            _ => {}
                        });
                    }
                    "Synonyms" => {
                        s2.section.iter().for_each(|s3| match &s3.tocheading[..] {
                            "Depositor-Supplied Synonyms" => s3.information[0]
                                .value
                                .string_with_markup
                                .iter()
                                .for_each(|d| {
                                    names.push(d.string.clone());
                                }),
                            _ => {}
                        });
                    }
                    _ => {}
                });
            }
            "Chemical and Physical Properties" => {
                s.section.iter().for_each(|s2| match &s2.tocheading[..] {
                    "Experimental Properties" => {
                        s2.section.iter().for_each(|s3| match &s3.tocheading[..] {
                            // "Solubility" => {
                            //     s3.information.iter().for_each(|f| {
                            //         if !f.value.string_with_markup.is_empty() {
                            //             vec.push(f.value.string_with_markup[0].string.clone());
                            //         }
                            //     });
                            // }
                            // "Melting Point" => {
                            //     s3.information.iter().for_each(|f| {
                            //         if !f.value.string_with_markup.is_empty() {
                            //             melting_v
                            //                 .push(f.value.string_with_markup[0].string.clone());
                            //         }
                            //     });
                            // }
                            // "LogP" => {
                            //     s3.information.iter().for_each(|f| {
                            //         if !f.value.string_with_markup.is_empty() {
                            //             logp.push(f.value.string_with_markup[0].string.clone());
                            //         }
                            //     });
                            // }
                            _ => {
                                let mut v = Vec::new();
                                s3.information.iter().for_each(|f| {
                                    f.value
                                        .string_with_markup
                                        .iter()
                                        .for_each(|f2| v.push(f2.clone()))
                                });

                                let p = Properties {
                                    kind: s3.tocheading.clone(),
                                    description: s3.description.clone().unwrap(),
                                    value: v,
                                };

                                properties.push(p);
                            }
                        });
                    }
                    "Computed Properties" => {
                        s2.section.iter().for_each(|s3| match &s3.tocheading[..] {
                            "Molecular Weight" => {
                                molecular_weight = format!(
                                    "{} {}",
                                    s3.information[0].value.string_with_markup[0].string.clone(),
                                    s3.information[0].value.unit.clone().unwrap()
                                );
                            }
                            _ => {}
                        });
                    }
                    _ => {}
                });
            }
            _ => {}
        });

    let f = SZData::new(
        cid,
        canonical_smiles,
        molecular_weight,
        inchi,
        properties,
        cas,
        inchi_key,
        names,
    );
    // info!("filter = {}", serde_json::to_string_pretty(&f).unwrap())

    if update {
        let _ = f.save_db();
    } else {
        let _ = f.insert_db();
    }
}

fn save_by_path(path: &str, update: bool) {
    if download::file_exist(path) {
        let chem = crate::chem::parse_json(path);
        if let Ok(r) = chem {
            parse_chem(&r, update);
        } else {
            log::error!("path({}) parse error! please check!", path);
        }
    } else {
        log::info!("path = {}, not exist!!", path);
    }
}

fn find_max_cid() -> usize {
    let find_options = FindOptions::builder()
        .sort(doc! { "cid": -1 })
        .limit(1)
        .build();

    let last = Db::find_with_table(
        DB_TABLE,
        DB_COLLECT,
        doc! {"source" : SOURCE.to_string()},
        find_options,
    );

    let mut start = 0;

    if let Ok(mut c) = last {
        if let Some(ccc) = c.next() {
            if let Ok(cc) = ccc {
                let result = bson::from_bson::<SZData>(Bson::Document(cc));
                if let Ok(ccc) = result {
                    start = ccc.cid as usize;
                }
            }
        }
    }

    start
}

pub fn save_to_db(data: &str) {
    let mut start = find_max_cid();

    let count = Db::count_with_table2(DB_TABLE, DB_COLLECT);

    info!("find already count = {}, start save id = {}", count, start);

    if count > 0 {
        info!("先尝试更新...");
        let p = start - start % STEP;

        (p..(start + STEP)).into_par_iter().for_each(|f| {
            let path = format!("{}/{}", data, crate::download::get_path_by_id(f));
            save_by_path(&path, true);
        });

        start = p + STEP;
    }

    info!("开始插入... start = {}", start);

    loop {
        let max = std::cmp::max(1, start);
        info!("start save {} ... ", max);
        (max..(max + STEP)).into_par_iter().for_each(|f| {
            let path = format!("{}/{}", data, crate::download::get_path_by_id(f));
            save_by_path(&path, false);
        });

        start += STEP;

        if start > 111460000 {
            info!("finish ...");
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_cid() {
        crate::db::init_db("mongodb://sz:sz@192.168.2.26:27017");
        crate::config::init_config();
        // info!("start find max cid");
        // info!("max cid = {}", find_max_cid());
        // info!("count ...");
        // info!("count = {}", Db::count_with_table2(DB_TABLE, DB_COLLECT));

        info!("start find");
        info!(
            "find {:?}",
            Db::find_one_with_table(
                "szdata",
                "molecular",
                // mongodb::bson::doc!("smiles":"C(CC(=O)N)CN=C(N)N"),
                mongodb::bson::doc! {"inchi":"InChI=1S/C5H12N4O/c6-4(10)2-1-3-9-5(7)8/h1-3H2,(H2,6,10)(H4,7,8,9)"},
                None
            )
        );
    }

    #[test]
    fn test_parse() {
        crate::config::init_config();

        let path = format!("data/{}", crate::download::get_path_by_id(42453744));

        let chem = crate::chem::parse_json(&path).unwrap();

        parse_chem(&chem, false);
    }

    fn init() {
        crate::db::init_db("mongodb://192.168.2.25:27017");
        crate::config::init_config();
    }

    #[test]
    fn test_save() {
        init();
        save_to_db("data");
    }

    #[test]
    fn test_delete_error_file() {
        init();
        let mut start = 42450000;

        let step = 10000;

        loop {
            info!("check start = {}", start);
            (start..(start + step)).into_par_iter().for_each(|f| {
                let path = format!("data/{}", crate::download::get_path_by_id(f));
                if download::file_exist(&path) {
                    let chem = crate::chem::parse_json(&path);
                    if let Ok(_) = chem {
                    } else {
                        log::error!("path({}) parse error! will delete!!", &path);
                        let _ = std::fs::remove_file(&path);
                    }
                } else {
                    log::info!("path = {}, not exist!!", path);
                }
            });

            start += step;
        }
    }
}
