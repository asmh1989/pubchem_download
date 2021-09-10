use std::os::linux::fs::MetadataExt;

use log::info;
use mongodb::bson::{self, doc, Document};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{
    chem::{parse_json, Chem},
    db::Db,
    filter_cid,
    shell::Shell,
};

pub const COLLECTION_FILTER_ABSORPTION: &'static str = "filter_absorption";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterAbsorption {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<bson::oid::ObjectId>,
    pub cid: i64,
    pub smiles: String,
    pub inchi: String,
    pub absorption: String,
}

impl FilterAbsorption {
    pub fn new(cid: i64, smiles: String, inchi: String, absorption: String) -> Self {
        Self {
            id: None,
            cid,
            smiles,
            inchi,
            absorption,
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

        if let Err(e) = Db::save(
            COLLECTION_FILTER_ABSORPTION,
            filter_cid!(self.cid.clone()),
            doc.clone(),
        ) {
            info!("db save error {} ", e);
            return Err(format!("db save error {} ", e));
        }
        Ok(())
    }
}

pub fn get_json_files(path: &str, dir: &mut Vec<String>, files: &mut Vec<String>) {
    let paths = std::fs::read_dir(path).unwrap();

    paths.for_each(|f| {
        if let Ok(d) = f {
            let p = d.path();
            if p.is_dir() {
                // get_json_files(p.to_str().unwrap(), vec);
                dir.push(p.to_str().unwrap().to_string());
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
                        files.push(path);
                    } else {
                        info!(
                            "remove file = {}, becase size = {} ",
                            path.clone(),
                            m.st_size()
                        );
                        let _ = std::fs::remove_file(path);
                    }
                }
            }
        }
    });
}

fn parse_chem(chem: &Chem) {
    let cid = chem.record.record_number;
    let mut absorption = "".to_string();
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
            "Pharmacology and Biochemistry" => {
                s.section.iter().for_each(|s2| match &s2.tocheading[..] {
                    "Absorption, Distribution and Excretion" => {
                        s2.information.iter().for_each(|s3| {
                            if let Some(name) = s3.name.clone() {
                                match &name[..] {
                                    "Absorption" => {
                                        absorption = s3.value.string_with_markup[0].string.clone();
                                    }
                                    _ => {}
                                }
                            }
                        });
                    }
                    _ => {}
                });
            }
            _ => {}
        });

    if !absorption.is_empty() {
        let f = FilterAbsorption::new(cid, canonical_smiles, inchi, absorption);
        // info!("filter = {}", serde_json::to_string_pretty(&f).unwrap());
        let _ = f.save_db();
    }
}

pub fn start_parse(dir: &str) {
    let mut dirs = Vec::<String>::with_capacity(512);
    let mut files = Vec::<String>::with_capacity(1000);

    let shell = Shell::new(".");

    get_json_files(dir, &mut dirs, &mut files);

    if !files.is_empty() {
        // files.sort_by_key(|a| {
        //     let path = std::path::Path::new(a);
        //     let name = path
        //         .file_name()
        //         .unwrap()
        //         .to_str()
        //         .unwrap()
        //         .replace(".json", "");
        //     name.parse::<usize>().unwrap()
        // });
        info!("find json files = {} in dir = {}", files.len(), dir);
        files.into_par_iter().for_each(|f| {
            if let Ok(_) = shell.run(&format!("cat {} | grep \"Oral bioavailability\"", &f)) {
                info!(" start parse json file = {}", &f);
                let result = parse_json(&f);
                if let Ok(chem) = result {
                    parse_chem(&chem);
                } else {
                    info!("{}, err = {:?}", f, result);
                }
            }
        });
    }
    if !dirs.is_empty() {
        dirs.sort_by_key(|a| {
            let path = std::path::Path::new(a);
            let name = path.file_name().unwrap().to_str().unwrap();
            name.parse::<usize>().unwrap()
        });
        for d in dirs {
            start_parse(&d);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        crate::config::init_config();
        crate::db::init_db("mongodb://192.168.2.25:27017");
        start_parse("data");
    }
}
