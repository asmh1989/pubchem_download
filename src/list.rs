use jwalk::WalkDirGeneric;
use log::info;

pub fn get_json_files(p: &str) -> usize {
    // let mut c: usize = 0;

    WalkDirGeneric::<((), ())>::new(p)
        .process_read_dir(move |_, _, _, _| {})
        .into_iter()
        .filter(|f| {
            if let Some(k) = f.as_ref().unwrap().path().extension() {
                if k == "json" {
                    return true;
                }
            }
            return false;
        })
        .count()

    // for entry in WalkDirGeneric::<((), ())>::new(p).process_read_dir(move |_, _, _, _| {}) {
    //     if let Some(k) = entry.unwrap().path().extension() {
    //         if k == "json" {
    //             c += 1;
    //         }
    //     }
    // }

    // c
}

pub fn list(dir: &str) {
    info!("start cal json files ..");
    let c = get_json_files(dir);
    info!("path in dir : {}, found json files : {}", dir, c);
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
