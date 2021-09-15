use log::info;

use once_cell::sync::Lazy;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    cmp::max,
    fs,
    io::Cursor,
    os::unix::prelude::MetadataExt,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    config,
    db::{Db, COLLECTION_CID_NOT_FOUND},
    filter_cid,
    model::PubChemNotFound,
};

static HTTP_PROXYS: Lazy<Mutex<Vec<&str>>> = Lazy::new(|| {
    let m = [
        ("139.9.148.153:9993"),
        (""),
        ("106.12.88.204:8888"),
        ("106.12.26.206:8888"),
        ("173.82.20.11:8880"),
        ("127.0.0.1:7890"),
        ("127.0.0.1:7891"),
        ("127.0.0.1:7892"),
        ("127.0.0.1:7893"),
        // (""),
    ]
    .iter()
    .cloned()
    .collect();

    Mutex::new(m)
});

fn fetch_url(f: usize, file_name: String, usb_db: bool, ip: &str) -> Result<(), String> {
    info!(
        "start download id = {}, path = {}, ip = {}",
        f, file_name, ip
    );

    let url = get_url(f);
    let path = std::path::Path::new(&file_name);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();

    let client = if ip.is_empty() {
        reqwest::blocking::Client::new()
    } else {
        reqwest::blocking::Client::builder()
            .proxy(reqwest::Proxy::all(ip).expect("http proxy set error"))
            .build()
            .map_err(|e| e.to_string())?
    };

    let response = client
        .get(url)
        // .headers(headers.clone())
        .send()
        .map_err(|e| e.to_string())?;
    let code = response.status().as_u16();

    if !response.status().is_success() {
        if code == 404 && usb_db {
            let d = PubChemNotFound::new(&f.to_string());
            let _ = d.save_db();
        }

        return Err(format!("请求失败! code = {}", response.status()));
    }
    let bytes = response.bytes().map_err(|e| e.to_string())?;
    if bytes.len() < 1024 {
        return Err("文件大小不对".to_string());
    }
    let mut content = Cursor::new(bytes);
    let mut file = std::fs::File::create(file_name).map_err(|e| e.to_string())?;
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
    format!("http://pubchem.ncbi.nlm.nih.gov/rest/pug_view/data/compound/{}/JSON/?response_type=save&response_basename=compound_CID_{}", f, f)
}

fn lock_ip(flags: &Arc<Mutex<Vec<usize>>>) -> Option<usize> {
    let d = &mut flags.lock().unwrap();
    let f = d.iter_mut().enumerate().find_map(|(k, v)| {
        if *v < config::Config::jobs() {
            *v += 1;
            return Some(k);
        }
        return None;
    });
    f
}

fn unlock_ip(flags: &Arc<Mutex<Vec<usize>>>, index: usize) {
    let d = &mut flags.lock().unwrap();
    d[index] -= 1;
}

fn get_chem(f: usize, use_db: bool, flags: &Arc<Mutex<Vec<usize>>>) {
    let path = format!("data/{}", get_path_by_id(f as usize));
    let ips = HTTP_PROXYS.lock().unwrap().clone();

    if !file_exist(&path) {
        if !use_db || !Db::contians(COLLECTION_CID_NOT_FOUND, filter_cid!(&f.to_string())) {
            loop {
                let ip = lock_ip(flags);
                if let Some(i) = ip {
                    let str = ips.get(i).unwrap().clone();
                    let result = fetch_url(f, path.clone(), use_db, str);
                    if result.is_err() {
                        info!("id = {}, ip = {} , result = {:?}", f, str, result);
                    }
                    unlock_ip(flags, i);
                    break;
                } else {
                    info!(
                        "need sleep ... id = {}, flags = {:?}",
                        f,
                        flags.lock().unwrap().clone()
                    );
                    thread::sleep(Duration::from_millis(3000));
                }
            }
        }
    } else {
        // info!("already download {}", f);
    }
}

pub fn download_chems_proxy(start: usize, use_db: bool, threads: usize) {
    let step = 20000000;
    let count = HTTP_PROXYS.lock().unwrap().len();

    let job = config::Config::jobs();
    if job != threads {
        config::Config::get_instance()
            .lock()
            .unwrap()
            .set_jobs(threads);
    }

    rayon::ThreadPoolBuilder::new()
        .num_threads(count * threads)
        .build_global()
        .unwrap();

    loop {
        let flags = Arc::new(Mutex::new(vec![0; count]));
        (max(1, start * step)..(start + 1) * step)
            .into_par_iter()
            .for_each(|f| {
                get_chem(f, use_db, &flags);
            });

        thread::sleep(Duration::from_millis(1500));
    }
}

pub fn download_chems(start: usize, use_db: bool) {
    let step = 1000000;

    (max(1, start * step)..(start + 1) * step)
        .into_par_iter()
        .for_each(|f| {
            let path = format!("data/{}", get_path_by_id(f as usize));

            if !file_exist(&path) {
                if !use_db || !Db::contians(COLLECTION_CID_NOT_FOUND, filter_cid!(&f.to_string())) {
                    let result = fetch_url(f, path.clone(), use_db, "");
                    if result.is_err() {
                        info!("id = {} , result = {:?}", f, result);
                    }
                }
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
    }

    #[test]
    fn test_proxy_download() {
        init();
        download_chems(1, true);
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
    fn test_request_proxy() {
        init();

        let client = reqwest::blocking::Client::builder()
            .proxy(reqwest::Proxy::http("106.12.88.204:8888").expect("http proxy set error"))
            .build()
            .unwrap();

        let response = client
            .get("http://192.168.0.7/")
            // .headers(headers.clone())
            .send()
            .unwrap();
        let code = response.text().unwrap();

        info!("response str = {}", code)

        // assert!(t.is_ok());
    }

    #[test]
    fn test_download() {
        init();
        download_chems_proxy(4, true, 6);
    }

    #[test]
    fn test_download_not_found() {
        init();
        // get_chem(25928, true);
    }
}
