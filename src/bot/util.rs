use std::path::Path;

#[cfg(debug_assertions)]
pub const VAR_LIB: &str = "target/debug";

#[cfg(not(debug_assertions))]
pub const VAR_LIB: &str = "/var/lib/mk-dl-bot";

// #[cfg(debug_assertions)]
// pub const VAR_LOG: &str = "target/debug";

// #[cfg(not(debug_assertions))]
// pub const VAR_LOG: &str = "/var/log/mk-dl-bot";

pub fn make_database_url() -> String {
    let path = Path::new(VAR_LIB).join("mk-dl-bot.db");
    format!("sqlite://{}", path.as_os_str().to_str().unwrap()).to_string()
}
