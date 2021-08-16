use log::info;
use once_cell::sync::OnceCell;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reqwest::header::{self, HeaderValue};
use std::{fs, io::Cursor, os::unix::prelude::MetadataExt};

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

fn fetch_url(url: String, file_name: String) -> Result<(), String> {
    let path = std::path::Path::new(&file_name);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();

    let client = reqwest::blocking::Client::new();

    let headers = HEADERS.get().expect("header not init");

    let response = client
        .get(url)
        // .headers(headers.clone())
        .send()
        .map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err("请求失败!".to_string());
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

pub fn download_chems(start: usize) {
    init_header();
    let step = 1000000;
    (start * step..(start + 1) * step).into_par_iter().for_each(|f| {
        let path = format!("data/{}", get_path_by_id(f as usize));
        let url = format!("https://pubchem.ncbi.nlm.nih.gov/rest/pug_view/data/compound/{}/JSON/?response_type=save&response_basename=compound_CID_{}", f, f);

        if !file_exist(&path) {
            info!("start download id = {}, path = {}", f, path);
            let result = fetch_url(url, path);
            if result.is_err() {
                info!("id = {}, result = {:?}", f, result);
            }
        } else {
            // info!("already download {}", f);
        }


    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_exist() {
        crate::config::init_config();
        let meta = fs::metadata("data/test");
        assert!(meta.is_ok());

        assert!(meta.unwrap().is_file());

        let meta = fs::metadata("data/1000000/1000/1.json");

        assert!(meta.is_ok());

        let mm = meta.unwrap();

        assert!(mm.is_file());

        info!("size = {}", mm.size());

        // assert!(file_exist("data/1000000/1000/1.json"));
    }

    #[test]
    fn test_download() {
        crate::config::init_config();
        download_chems(0);
    }
}
