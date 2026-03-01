use std::path::PathBuf;

pub fn get_home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

pub fn get_app_data_dir() -> PathBuf {
    get_home_dir().join(".todaynote")
}
