use anyhow::Result;
use async_trait::async_trait;
use config::builder::AsyncState;
use config::{
    AsyncSource, ConfigBuilder, ConfigError, FileFormat, FileStoredFormat, Format, Map, Value,
};
use input_linux::Key;
use linux_cec::operand::UiCommand;
use serde::de::{self, Unexpected};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tokio::fs::{read_dir, read_to_string};
use tracing::{debug, error, info};

fn de_mappings<'de, D>(deserializer: D) -> Result<HashMap<UiCommand, Key>, D::Error>
where
    D: Deserializer<'de>,
{
    let string_map = HashMap::<String, u16>::deserialize(deserializer)?;
    let mut mappings = HashMap::new();
    for (k, v) in string_map {
        let Ok(key) = UiCommand::from_str(&k) else {
            return Err(de::Error::invalid_value(
                Unexpected::Str(&k),
                &"an HDMI CEC UI command name",
            ));
        };
        let Ok(value) = Key::try_from(v) else {
            return Err(de::Error::invalid_value(
                Unexpected::Unsigned(v.into()),
                &"a Linux input event key value",
            ));
        };
        mappings.insert(key, value);
    }

    Ok(mappings)
}

#[derive(Deserialize, Clone, Debug, Default)]
pub(crate) struct Config {
    pub osd_name: Option<String>,
    pub vendor_id: Option<[u8; 3]>,
    pub logical_address: Option<u8>,
    #[serde(deserialize_with = "de_mappings", default)]
    pub mappings: HashMap<UiCommand, Key>,
}

#[derive(Debug)]
struct AsyncFileSource<F: Format, P: AsRef<Path> + Sized + Send + Sync> {
    path: P,
    format: F,
}

impl<F: Format, P: AsRef<Path> + Sized + Send + Sync + Debug> AsyncFileSource<F, P> {
    fn from(path: P, format: F) -> AsyncFileSource<F, P> {
        AsyncFileSource { path, format }
    }
}

#[async_trait]
impl<F: Format + Send + Sync + Debug, P: AsRef<Path> + Sized + Send + Sync + Debug> AsyncSource
    for AsyncFileSource<F, P>
{
    async fn collect(&self) -> Result<Map<String, Value>, ConfigError> {
        let path = self.path.as_ref();
        let text = match read_to_string(&path).await {
            Ok(text) => text,
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    info!("No config file {} found", path.to_string_lossy());
                    return Ok(Map::new());
                }
                return Err(ConfigError::Foreign(Box::new(e)));
            }
        };
        let path = path.to_string_lossy().to_string();
        self.format
            .parse(Some(&path), &text)
            .map_err(ConfigError::Foreign)
    }
}

async fn read_config_directory<P: AsRef<Path> + Sync + Send>(
    builder: ConfigBuilder<AsyncState>,
    path: P,
    extensions: &[&str],
    format: FileFormat,
) -> Result<ConfigBuilder<AsyncState>> {
    let mut dir = match read_dir(&path).await {
        Ok(dir) => dir,
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                debug!(
                    "No config fragment directory {} found",
                    path.as_ref().to_string_lossy()
                );
                return Ok(builder);
            }
            error!(
                "Error reading config fragment directory {}: {e}",
                path.as_ref().to_string_lossy()
            );
            return Err(e.into());
        }
    };
    let mut entries = Vec::new();
    while let Some(entry) = dir.next_entry().await? {
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
            if extensions.contains(&ext) {
                entries.push(path);
            }
        }
    }
    entries.sort();
    Ok(entries.into_iter().fold(builder, |builder, path| {
        builder.add_async_source(AsyncFileSource::from(path, format))
    }))
}

pub(crate) async fn read_default_config() -> Result<Config> {
    let builder = ConfigBuilder::<AsyncState>::default();
    let system_config_path = PathBuf::from("/usr/share/cecd");
    let user_config_path = PathBuf::from("/etc/cecd");

    let builder = builder.add_async_source(AsyncFileSource::from(
        system_config_path.join("config.toml"),
        FileFormat::Toml,
    ));
    let builder = read_config_directory(
        builder,
        system_config_path.join("config.toml.d"),
        FileFormat::Toml.file_extensions(),
        FileFormat::Toml,
    )
    .await?;

    let builder = builder.add_async_source(AsyncFileSource::from(
        user_config_path.join("config.toml"),
        FileFormat::Toml,
    ));
    let builder = read_config_directory(
        builder,
        user_config_path.join("config.toml.d"),
        FileFormat::Toml.file_extensions(),
        FileFormat::Toml,
    )
    .await?;
    let config = builder.build().await?;
    Ok(config.try_deserialize()?)
}
