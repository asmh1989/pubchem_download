use std::{fs, os::linux::fs::MetadataExt, path::PathBuf};

use log::info;
use mongodb::bson::{self, doc};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{
    chem::{parse_json, Chem},
    db::{Db, COLLECTION_FILTER_SMILES_SOLUBILITY},
    filter_cid,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<bson::oid::ObjectId>,
    pub cid: i64,
    pub smiles: String,
    pub inchi: String,
    pub molecular_weight: String,
    pub solubility: Vec<String>,
}

impl Filter {
    pub fn new(
        cid: i64,
        smiles: String,
        molecular_weight: String,
        inchi: String,
        solubility: Vec<String>,
    ) -> Self {
        Self {
            id: None,
            cid,
            smiles,
            molecular_weight,
            solubility,
            inchi,
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

        if let Err(e) = Db::save(
            COLLECTION_FILTER_SMILES_SOLUBILITY,
            filter_cid!(self.cid.clone()),
            doc.clone(),
        ) {
            info!("db save error {} ", e);
            return Err(format!("db save error {} ", e));
        }
        Ok(())
    }
}

pub fn get_json_files(path: &str, vec: &mut Vec<String>) {
    let paths = fs::read_dir(path).unwrap();

    paths.for_each(|f| {
        if let Ok(d) = f {
            let p = d.path();
            if p.is_dir() {
                get_json_files(p.to_str().unwrap(), vec);
            } else if let Some(k) = p.extension() {
                if k == "json" {
                    // info!("found json file : {:?}", p);
                    let m = p.metadata().unwrap();
                    let path = p
                        .clone()
                        .into_os_string()
                        .into_string()
                        .unwrap()
                        .to_string();
                    if m.st_size() > 1024 {
                        vec.push(path);
                    } else {
                        info!(
                            "remove file = {}, becase size = {} ",
                            path.clone(),
                            m.st_size()
                        );
                        let _ = fs::remove_file(path);
                    }
                }
            }
        }
    });
}

fn parse_chem(chem: &Chem) {
    let cid = chem.record.record_number;
    let mut vec: Vec<String> = Vec::new();
    let mut molecular_weight = "".to_string();
    let mut canonical_smiles = "".to_string();
    let mut isomeric_smiles = "".to_string();
    let mut inchi = "".to_string();
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
                            "Solubility" => {
                                s3.information.iter().for_each(|f| {
                                    if !f.value.string_with_markup.is_empty() {
                                        vec.push(f.value.string_with_markup[0].string.clone());
                                    }
                                });
                            }
                            _ => {}
                        });

                        if vec.is_empty() {
                            return;
                        }
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

    if !vec.is_empty() {
        let f = Filter::new(cid, canonical_smiles, molecular_weight, inchi, vec);
        // info!("filter = {}", serde_json::to_string_pretty(&f).unwrap())
        let _ = f.save_db();
    }
}

pub fn start_parse(dir: &str) {
    let mut vec: Vec<String> = Vec::with_capacity(1000);
    get_json_files(dir, &mut vec);

    info!("path in dir : {}, found json files : {}", dir, vec.len());

    vec.into_par_iter().for_each(|f| {
        // let name = PathBuf::from(&f)
        //     .file_stem()
        //     .unwrap()
        //     .to_os_string()
        //     .into_string()
        //     .unwrap();
        // if !contains(&name.clone()) {
        let result = parse_json(&f);
        if let Ok(chem) = result {
            parse_chem(&chem);
        } else {
            info!("{}, err = {:?}", f, result);
        }
        // } else {
        //     info!("cid = {}, already in db", name);
        // }
    });
}

pub fn start_filter(name: &str, data: &str) {
    match name {
        _ => start_parse(data),
    }
}

fn contains(cid: &str) -> bool {
    Db::contians(
        COLLECTION_FILTER_SMILES_SOLUBILITY,
        doc! {"cid":cid.parse::<i64>().unwrap()},
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        crate::config::init_config();
        crate::db::init_db("mongodb://192.168.2.25:27017");

        assert!(contains("2342"));

        rayon::ThreadPoolBuilder::new()
            .num_threads(24)
            .build_global()
            .unwrap();

        start_parse("data/");
    }
}
