pub fn init_config() {
    let r = log4rs::init_file("config/log4rs.yaml", Default::default());

    if r.is_err() {
        let _ = log4rs::init_file("rust/config/log4rs.yaml", Default::default());
    }
}

mod tests {}
