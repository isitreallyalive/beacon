use std::io;

#[derive(serde::Deserialize)]
pub struct Config {
    pub port: u16,
}

impl Config {
    pub fn read<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path)?;
        let config: Config =
            toml::from_str(&contents).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(config)
    }
}
