use std::{fs, path::PathBuf};

use log::info;

use crate::chem::{parse_json, Chem};

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    pub cid: i64,
    pub smiles: String,
    pub molecular_weight: String,
    pub solubility: Vec<String>,
}

impl Filter {
    pub fn new(
        cid: i64,
        smiles: String,
        molecular_weight: String,
        solubility: Vec<String>,
    ) -> Self {
        Self {
            cid,
            smiles,
            molecular_weight,
            solubility,
        }
    }
}

pub fn get_json_files(path: &str, vec: &mut Vec<PathBuf>) {
    let paths = fs::read_dir(path).unwrap();

    paths.for_each(|f| {
        if let Ok(d) = f {
            let p = d.path();
            if p.is_dir() {
                get_json_files(p.to_str().unwrap(), vec);
            } else if let Some(k) = p.extension() {
                if k == "json" {
                    // info!("found json file : {:?}", p);
                    vec.push(p);
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
                                    } else {
                                        return;
                                    }
                                });
                            }
                            _ => {}
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

    if !vec.is_empty() {
        let f = Filter::new(cid, canonical_smiles, molecular_weight, vec);
        info!("filter = {}", serde_json::to_string_pretty(&f).unwrap())
    }
}

pub fn start_parse(dir: &str) {
    let mut vec: Vec<PathBuf> = Vec::with_capacity(100000);
    get_json_files(dir, &mut vec);

    vec.iter().for_each(|f| {
        let str = f.clone().into_os_string().into_string().unwrap();
        let result = parse_json(&str);
        if let Ok(chem) = result {
            parse_chem(&chem);
        } else {
            info!("{}, err = {:?}", str, result);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        crate::config::init_config();

        start_parse("data/1000000/3000");
    }
}
