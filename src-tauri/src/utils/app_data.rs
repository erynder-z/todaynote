use std::path::PathBuf;

pub fn get_app_data_dir() -> PathBuf {
    if let Some(home_dir) = dirs::home_dir() {
        return home_dir.join(".todaynote");
    }

    PathBuf::from(".todaynote")
}
