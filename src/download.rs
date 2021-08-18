use log::info;
use mongodb::bson;
use once_cell::sync::OnceCell;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reqwest::header::{self, HeaderValue};
use std::{cmp::max, fs, io::Cursor, os::unix::prelude::MetadataExt};

use crate::{
    db::{init_db, Db, COLLECTION_CID_NOT_FOUND},
    filter_cid,
    model::PubChemNotFound,
};

static HEADERS: OnceCell<header::HeaderMap> = OnceCell::new();

pub fn init_header() {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.141 Safari/537.36 Edg/87.0.664.75"),
    );
    // headers.insert(header::HOST, HeaderValue::from_static("223.4.70.240"));
    // headers.insert(
    //     header::CONTENT_TYPE,
    //     HeaderValue::from_static("application/x-www-form-urlencoded"),
    // );
    headers.insert(
        header::ACCEPT,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );

    // headers.insert(
    //     header::ORIGIN,
    //     HeaderValue::from_static("http://223.4.65.131:8080"),
    // );

    // headers.insert(
    //     header::ACCEPT_ENCODING,
    //     HeaderValue::from_static("gzip, deflate"),
    // );

    headers.insert(
        header::REFERER,
        HeaderValue::from_static("https://pubchem.ncbi.nlm.nih.gov/"),
    );

    let _ = HEADERS.set(headers);
}

fn fetch_url(f: usize, file_name: String, usb_db: bool) -> Result<(), String> {
    let url = get_url(f);
    let path = std::path::Path::new(&file_name);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();

    let client = reqwest::blocking::Client::new();

    // let headers = HEADERS.get().expect("header not init");

    let response = client
        .get(url)
        // .headers(headers.clone())
        .send()
        .map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        let code = response.status().as_u16();

        if code == 404 && usb_db {
            let d = PubChemNotFound::new(&f.to_string());
            let _ = d.save_db();
        }

        return Err(format!("请求失败! code = {}", response.status()));
    }
    let mut file = std::fs::File::create(file_name).map_err(|e| e.to_string())?;
    let bytes = response.bytes().map_err(|e| e.to_string())?;
    if bytes.len() < 1024 {
        return Err("文件大小不对".to_string());
    }
    let mut content = Cursor::new(bytes);
    std::io::copy(&mut content, &mut file).map_err(|e| e.to_string())?;
    Ok(())
}

fn get_path_by_id(id: usize) -> String {
    let million: usize = 1000000;
    let thousand: usize = 1000;

    let first = id / million;

    let second = (id - first * million) / thousand;

    return format!(
        "{}/{}/{}.json",
        (first + 1) * million,
        (second + 1) * thousand,
        id
    );
}

pub fn file_exist(path: &str) -> bool {
    let meta = fs::metadata(path);
    if let Ok(m) = meta {
        if m.is_file() && m.size() > 1024 {
            return true;
        } else {
            if m.is_dir() {
                let _ = fs::remove_dir_all(path);
            }
            return false;
        }
    } else {
        return false;
    }
}

#[inline]
fn get_url(f: usize) -> String {
    format!("https://pubchem.ncbi.nlm.nih.gov/rest/pug_view/data/compound/{}/JSON/?response_type=save&response_basename=compound_CID_{}", f, f)
}

fn get_chem(f: usize, usb_db: bool) {
    let path = format!("data/{}", get_path_by_id(f as usize));

    if !file_exist(&path) {
        info!("start download id = {}, path = {}", f, path);
        let result = fetch_url(f, path, usb_db);
        if result.is_err() {
            info!("id = {}, result = {:?}", f, result);
        }
    } else {
        // info!("already download {}", f);
    }
}

pub fn download_chems(start: usize, use_db: bool) {
    if use_db {
        init_db("mongodb://192.168.2.25:27017");
    }

    init_header();
    let step = 1000000;
    (max(1, start * step)..(start + 1) * step)
        .into_par_iter()
        .for_each(|f| {
            if !use_db || !Db::contians(COLLECTION_CID_NOT_FOUND, filter_cid!(&f.to_string())) {
                get_chem(f, use_db);
            } else {
                // info!("cid = {} is 404 not found, not need download again!", f);
            }
        });
}

#[cfg(test)]
mod tests {
    use crate::db;

    use super::*;

    fn init() {
        db::init_db("mongodb://192.168.2.25:27017");
        crate::config::init_config();
        rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build_global()
            .unwrap();
    }

    #[test]
    fn test_file_exist() {
        crate::config::init_config();

        let meta = fs::metadata("data/1000000/1000/1.json");

        assert!(meta.is_ok());

        let mm = meta.unwrap();

        assert!(mm.is_file());

        info!("size = {}", mm.size());

        // assert!(file_exist("data/1000000/1000/1.json"));
    }

    #[test]
    fn test_download() {
        init();
        download_chems(0, true);
    }

    #[test]
    fn test_download_not_found() {
        init();
        get_chem(25928, true);
    }
}
