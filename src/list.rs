use std::sync::{Arc, Mutex};

use log::info;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

pub fn get_json_files(path: &str, count: &Arc<Mutex<usize>>) {
    let paths = std::fs::read_dir(path).unwrap();
    let mut v = Vec::<String>::with_capacity(1000);

    let mut c: usize = 0;

    paths.for_each(|f| {
        if let Ok(d) = f {
            let p = d.path();
            if p.is_dir() {
                // get_json_files(p.to_str().unwrap(), vec);
                v.push(p.to_str().unwrap().to_string());
            } else if let Some(k) = p.extension() {
                if k == "json" {
                    c += 1;
                }
            }
        }
    });

    *count.lock().unwrap() += c;

    v.into_par_iter().for_each(|f| get_json_files(&f, count));
}

pub fn list(dir: &str) {
    info!("start cal json files ..");
    let vec2 = Arc::new(Mutex::new(0));
    get_json_files(dir, &vec2);
    let vec = vec2.lock().unwrap().to_owned();
    info!("path in dir : {}, found json files : {}", dir, vec);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list() {
        crate::config::init_config();

        list("data");
    }
}
