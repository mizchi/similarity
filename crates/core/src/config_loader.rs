use std::path::PathBuf;

pub trait ConfigLoader: Sized + Default + serde::de::DeserializeOwned {
    fn find_config_file() -> Option<PathBuf> {
        let mut dir = std::env::current_dir().ok()?;
        loop {
            let candidate = dir.join("similarity.toml");
            if candidate.exists() {
                return Some(candidate);
            }
            if !dir.pop() {
                return None;
            }
        }
    }

    fn load_from_file(path: PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(&path)
            .map_err(|error| anyhow::anyhow!("Failed to read {}: {}", path.display(), error))?;
        let config = toml::from_str(&content)
            .map_err(|error| anyhow::anyhow!("Failed to parse {}: {}", path.display(), error))?;
        Ok(config)
    }

    fn find_and_load() -> Self {
        let Some(path) = Self::find_config_file() else {
            return Self::default();
        };

        Self::load_from_file(path.clone()).unwrap_or_else(|error| {
            eprintln!("Warning: could not load {}: {error}", path.display());
            Self::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;

    static CWD_LOCK: Mutex<()> = Mutex::new(());

    fn cwd_lock() -> std::sync::MutexGuard<'static, ()> {
        CWD_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[derive(Debug, Default, PartialEq, serde::Deserialize)]
    struct TestConfig {
        threshold: Option<f64>,
        min_lines: Option<u32>,
        skip_test: Option<bool>,
        exclude: Option<Vec<String>>,
    }

    impl ConfigLoader for TestConfig {}

    fn write_config(content: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("similarity.toml"), content).unwrap();
        let path = dir.path().to_path_buf();
        (dir, path)
    }

    #[test]
    fn finds_config_in_current_dir() {
        let _guard = cwd_lock();
        let (_dir, dir_path) = write_config("threshold = 0.9");
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir_path).unwrap();

        let found = TestConfig::find_config_file();

        std::env::set_current_dir(original).unwrap();
        assert!(found.is_some());
    }

    #[test]
    fn finds_config_in_parent_dir() {
        let _guard = cwd_lock();
        let (_dir, dir_path) = write_config("threshold = 0.9");
        let nested = dir_path.join("src/nested");
        fs::create_dir_all(&nested).unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(&nested).unwrap();

        let found = TestConfig::find_config_file();

        std::env::set_current_dir(original).unwrap();
        assert_eq!(
            found.and_then(|path| path.canonicalize().ok()),
            Some(dir_path.join("similarity.toml").canonicalize().unwrap())
        );
    }

    #[test]
    fn returns_none_when_config_missing() {
        let _guard = cwd_lock();
        let dir = tempfile::tempdir().unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let found = TestConfig::find_config_file();

        std::env::set_current_dir(original).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn loads_partial_config() {
        let (_dir, dir_path) = write_config("threshold = 0.75");

        let config = TestConfig::load_from_file(dir_path.join("similarity.toml")).unwrap();

        assert_eq!(
            config,
            TestConfig { threshold: Some(0.75), min_lines: None, skip_test: None, exclude: None }
        );
    }

    #[test]
    fn invalid_toml_returns_error() {
        let (_dir, dir_path) = write_config("not = [valid");

        let result = TestConfig::load_from_file(dir_path.join("similarity.toml"));

        assert!(result.is_err());
    }

    #[test]
    fn find_and_load_falls_back_to_default() {
        let _guard = cwd_lock();
        let dir = tempfile::tempdir().unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let config = TestConfig::find_and_load();

        std::env::set_current_dir(original).unwrap();
        assert_eq!(config, TestConfig::default());
    }
}
