use std::path::PathBuf;

/// Implement this trait on any struct that derives `serde::Deserialize` and `Default`
/// to get automatic `similarity.toml` config file loading.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(serde::Deserialize, Default)]
/// struct Config {
///     threshold: Option<f64>,
///     min_lines: Option<u32>,
/// }
///
/// impl similarity_core::ConfigLoader for Config {}
///
/// // In main():
/// let config = Config::find_and_load();
/// ```
pub trait ConfigLoader: Sized + Default + serde::de::DeserializeOwned {
    /// Walk up from the current directory looking for `similarity.toml`.
    /// Returns the first file found, or `None` if none exists in any ancestor.
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

    /// Read and deserialize a `similarity.toml` at the given path.
    fn load_from_file(path: PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))?;
        let config: Self = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse {}: {}", path.display(), e))?;
        Ok(config)
    }

    /// Find and load the nearest `similarity.toml`, returning `Default` if none is found.
    /// Parse errors are printed as warnings and also fall back to `Default`.
    fn find_and_load() -> Self {
        if let Some(path) = Self::find_config_file() {
            Self::load_from_file(path.clone())
                .unwrap_or_else(|e| {
                    eprintln!("Warning: could not load {}: {e}", path.display());
                    Self::default()
                })
        } else {
            Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;

    // set_current_dir is process-global — serialize any test that changes it.
    static CWD_LOCK: Mutex<()> = Mutex::new(());

    #[derive(serde::Deserialize, Default, Debug, PartialEq)]
    struct TestConfig {
        threshold: Option<f64>,
        min_lines: Option<u32>,
        skip_test: Option<bool>,
        exclude: Option<Vec<String>>,
    }

    impl ConfigLoader for TestConfig {}

    /// Write `content` to a `similarity.toml` in a fresh temp dir.
    /// Returns `(TempDir, path_to_dir)` — keep `TempDir` alive for the test duration.
    fn write_config(content: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("similarity.toml"), content).unwrap();
        let dir_path = dir.path().to_path_buf();
        (dir, dir_path)
    }

    // ── find_config_file ──────────────────────────────────────────────────────

    #[test]
    fn finds_config_in_current_dir() {
        let _guard = CWD_LOCK.lock().unwrap();
        let (_dir, dir_path) = write_config("threshold = 0.9");
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir_path).unwrap();

        let found = TestConfig::find_config_file();

        std::env::set_current_dir(&original).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().file_name().unwrap(), "similarity.toml");
    }

    #[test]
    fn returns_none_when_no_config_exists() {
        let _guard = CWD_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let found = TestConfig::find_config_file();

        std::env::set_current_dir(&original).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn finds_config_in_parent_dir() {
        let _guard = CWD_LOCK.lock().unwrap();
        let (_dir, dir_path) = write_config("threshold = 0.9");
        let subdir = dir_path.join("subproject");
        fs::create_dir_all(&subdir).unwrap();

        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(&subdir).unwrap();

        let found = TestConfig::find_config_file();

        std::env::set_current_dir(&original).unwrap();
        assert!(found.is_some());
    }

    // ── load_from_file ────────────────────────────────────────────────────────

    #[test]
    fn loads_all_supported_fields() {
        let (_dir, dir_path) = write_config(
            r#"
threshold = 0.92
min_lines = 10
skip_test = true
exclude = ["target/", "tests/fixtures/"]
"#,
        );

        let config = TestConfig::load_from_file(dir_path.join("similarity.toml")).unwrap();

        assert_eq!(config.threshold, Some(0.92));
        assert_eq!(config.min_lines, Some(10));
        assert_eq!(config.skip_test, Some(true));
        assert_eq!(
            config.exclude,
            Some(vec!["target/".to_string(), "tests/fixtures/".to_string()])
        );
    }

    #[test]
    fn partial_config_leaves_unset_fields_as_none() {
        let (_dir, dir_path) = write_config("threshold = 0.75");

        let config = TestConfig::load_from_file(dir_path.join("similarity.toml")).unwrap();

        assert_eq!(config.threshold, Some(0.75));
        assert_eq!(config.min_lines, None);
        assert_eq!(config.skip_test, None);
    }

    #[test]
    fn empty_config_file_gives_all_none() {
        let (_dir, dir_path) = write_config("");

        let config = TestConfig::load_from_file(dir_path.join("similarity.toml")).unwrap();

        assert_eq!(config, TestConfig::default());
    }

    #[test]
    fn invalid_toml_returns_error() {
        let (_dir, dir_path) = write_config("this is not : valid toml !!!");

        let result = TestConfig::load_from_file(dir_path.join("similarity.toml"));

        assert!(result.is_err());
    }

    #[test]
    fn missing_file_returns_error() {
        let path = PathBuf::from("/tmp/this_does_not_exist_similarity_test.toml");
        assert!(TestConfig::load_from_file(path).is_err());
    }

    // ── find_and_load ─────────────────────────────────────────────────────────

    #[test]
    fn find_and_load_returns_default_when_no_file() {
        let _guard = CWD_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let config = TestConfig::find_and_load();

        std::env::set_current_dir(&original).unwrap();
        assert_eq!(config, TestConfig::default());
    }

    #[test]
    fn find_and_load_reads_config_from_cwd() {
        let _guard = CWD_LOCK.lock().unwrap();
        let (_dir, dir_path) = write_config("threshold = 0.88\nmin_lines = 7");
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir_path).unwrap();

        let config = TestConfig::find_and_load();

        std::env::set_current_dir(&original).unwrap();
        assert_eq!(config.threshold, Some(0.88));
        assert_eq!(config.min_lines, Some(7));
    }
}
