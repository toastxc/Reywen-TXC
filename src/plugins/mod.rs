pub mod federolt;
pub fn conf_from_file<T: serde::de::DeserializeOwned>(path: &str) -> T {
    toml::from_str::<T>(&String::from_utf8(std::fs::read(path).unwrap()).unwrap()).unwrap()
}
