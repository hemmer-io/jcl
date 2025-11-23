//! Module source resolution and caching
//!
//! This module provides functionality to resolve module sources from various locations:
//! - Local file paths (./path/to/module.jcf)
//! - Registry modules (registry::module-name@^1.0.0)
//! - Git repositories (git::https://github.com/user/repo.git//path/to/module.jcf?ref=v1.0.0)
//! - HTTP/HTTPS URLs (https://example.com/modules/module.jcf)
//! - Tarballs (https://example.com/modules/module.tar.gz//module.jcf)
//!
//! It also handles caching of remote modules and version resolution.

use anyhow::{anyhow, Context, Result};
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::module_registry::RegistryClient;

/// Module source types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModuleSource {
    /// Local file path: ./path/to/module.jcf or /absolute/path/to/module.jcl
    Local { path: PathBuf },

    /// Registry module: registry::module-name@^1.0.0
    Registry {
        name: String,
        version_req: String, // Stored as string for serialization
    },

    /// Git repository: git::https://github.com/user/repo.git//path/to/module.jcf?ref=v1.0.0
    Git {
        url: String,
        path: String,
        reference: Option<String>, // branch, tag, or commit hash
    },

    /// HTTP/HTTPS URL: https://example.com/modules/module.jcl
    Http { url: String },

    /// Tarball: https://example.com/modules/module.tar.gz//module.jcl
    Tarball { url: String, path: String },
}

/// Module source resolver
pub struct ModuleSourceResolver {
    /// Cache directory for downloaded modules
    cache_dir: PathBuf,

    /// Module lock entries (for reproducible builds)
    lock_entries: HashMap<String, LockEntry>,

    /// Registry client for resolving registry modules
    registry_client: Option<RegistryClient>,
}

/// Entry in the lock file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockEntry {
    pub source: String,
    pub resolved_url: Option<String>,
    pub checksum: Option<String>,
    pub version: Option<String>,
}

/// Lock file format (.jcf.lock)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    pub version: String,
    pub modules: HashMap<String, LockEntry>,
}

impl ModuleSourceResolver {
    /// Create a new module source resolver
    pub fn new(cache_dir: Option<PathBuf>) -> Self {
        let cache_dir = cache_dir.unwrap_or_else(|| {
            dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from(".jcf-cache"))
                .join("jcl")
                .join("modules")
        });

        Self {
            cache_dir,
            lock_entries: HashMap::new(),
            registry_client: Some(RegistryClient::default_registry()),
        }
    }

    /// Parse a module source string
    pub fn parse_source(source: &str) -> Result<ModuleSource> {
        // Registry source: registry::module-name@^1.0.0 or registry::module-name
        if source.starts_with("registry::") {
            let source = source.strip_prefix("registry::").unwrap();

            // Split by @ to separate name from version
            let (name, version_req) = if let Some(idx) = source.find('@') {
                let name = &source[..idx];
                let version = &source[idx + 1..];
                (name.to_string(), version.to_string())
            } else {
                // No version specified, use latest
                (source.to_string(), "*".to_string())
            };

            return Ok(ModuleSource::Registry { name, version_req });
        }

        // Git source: git::https://github.com/user/repo.git//path/to/module.jcf?ref=v1.0.0
        if source.starts_with("git::") {
            let source = source.strip_prefix("git::").unwrap();

            // Find the position of .git// to split properly (avoiding https://)
            let marker = ".git//";
            let marker_pos = source.find(marker).ok_or_else(|| {
                anyhow!("Invalid git source format. Expected: git::URL.git//PATH?ref=REF")
            })?;

            // URL is everything up to and including ".git"
            let url_part = &source[..marker_pos + 4]; // Include ".git"
                                                      // Path is everything after the "//"
            let path_part = &source[marker_pos + marker.len()..];

            // Extract reference from query string
            let (path, reference) = if let Some(idx) = path_part.find('?') {
                let path = &path_part[..idx];
                let query = &path_part[idx + 1..];

                // Parse query string for ref parameter
                let ref_value = query.split('&').find_map(|param| {
                    let kv: Vec<&str> = param.splitn(2, '=').collect();
                    if kv.len() == 2 && kv[0] == "ref" {
                        Some(kv[1].to_string())
                    } else {
                        None
                    }
                });

                (path.to_string(), ref_value)
            } else {
                (path_part.to_string(), None)
            };

            return Ok(ModuleSource::Git {
                url: url_part.to_string(),
                path,
                reference,
            });
        }

        // Tarball source: https://example.com/modules/module.tar.gz//module.jcl
        if source.contains(".tar.gz//") || source.contains(".tgz//") {
            let split_marker = if source.contains(".tar.gz//") {
                ".tar.gz//"
            } else {
                ".tgz//"
            };

            let split_pos = if let Some(pos) = source.find(split_marker) {
                pos + split_marker.len() - 2 // Position after .tar.gz or .tgz, excluding //
            } else {
                return Err(anyhow!(
                    "Invalid tarball source format. Expected: URL.tar.gz//PATH"
                ));
            };

            let url_part = &source[..split_pos];
            let path_part = &source[split_pos + 2..]; // Skip the //

            return Ok(ModuleSource::Tarball {
                url: url_part.to_string(),
                path: path_part.to_string(),
            });
        }

        // HTTP source: https://example.com/modules/module.jcl
        if source.starts_with("http://") || source.starts_with("https://") {
            return Ok(ModuleSource::Http {
                url: source.to_string(),
            });
        }

        // Local file path (default)
        Ok(ModuleSource::Local {
            path: PathBuf::from(source),
        })
    }

    /// Resolve a module source to a local file path
    pub fn resolve(&mut self, source: &str, base_dir: &Path) -> Result<PathBuf> {
        let parsed = Self::parse_source(source)?;

        match parsed {
            ModuleSource::Local { path } => {
                // Resolve relative to base directory
                let resolved = if path.is_absolute() {
                    path
                } else {
                    base_dir.join(&path)
                };

                if !resolved.exists() {
                    return Err(anyhow!("Module file not found: {}", resolved.display()));
                }

                Ok(resolved)
            }

            ModuleSource::Registry { name, version_req } => {
                self.resolve_registry(&name, &version_req)
            }

            ModuleSource::Git {
                url,
                path,
                reference,
            } => self.resolve_git(&url, &path, reference.as_deref()),

            ModuleSource::Http { url } => self.resolve_http(&url),

            ModuleSource::Tarball { url, path } => self.resolve_tarball(&url, &path),
        }
    }

    /// Resolve a registry module source
    fn resolve_registry(&mut self, name: &str, version_req_str: &str) -> Result<PathBuf> {
        // Get registry client
        let registry_client = self
            .registry_client
            .as_ref()
            .ok_or_else(|| anyhow!("Registry client not initialized"))?;

        // Parse version requirement
        let version_req =
            VersionReq::parse(version_req_str).context("Invalid version requirement")?;

        // Resolve version
        let version = registry_client.resolve_version(name, &version_req)?;

        // Download module
        let module_dir = registry_client.download(name, &version)?;

        // Get manifest to find main file
        let manifest_path = module_dir.join("jcl.json");
        let manifest = crate::module_registry::ModuleManifest::load(&manifest_path)?;

        // Return path to main module file
        let main_path = module_dir.join(&manifest.main);

        if !main_path.exists() {
            return Err(anyhow!(
                "Main module file '{}' not found in {}",
                manifest.main,
                name
            ));
        }

        Ok(main_path)
    }

    /// Resolve a Git repository source
    #[cfg(not(target_arch = "wasm32"))]
    fn resolve_git(&mut self, url: &str, path: &str, reference: Option<&str>) -> Result<PathBuf> {
        use std::process::Command;

        // Create a cache key from the URL
        let cache_key = format!("{:x}", md5::compute(url.as_bytes()));
        let repo_dir = self.cache_dir.join("git").join(&cache_key);

        // Clone or update the repository
        if !repo_dir.exists() {
            fs::create_dir_all(&repo_dir).context("Failed to create cache directory")?;

            let output = Command::new("git")
                .args(["clone", url, repo_dir.to_str().unwrap()])
                .output()
                .context("Failed to clone git repository")?;

            if !output.status.success() {
                return Err(anyhow!(
                    "Git clone failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        } else {
            // Update existing repository
            let output = Command::new("git")
                .args(["-C", repo_dir.to_str().unwrap(), "fetch", "--all"])
                .output()
                .context("Failed to fetch git repository")?;

            if !output.status.success() {
                return Err(anyhow!(
                    "Git fetch failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }

        // Checkout the specified reference
        if let Some(ref_name) = reference {
            let output = Command::new("git")
                .args(["-C", repo_dir.to_str().unwrap(), "checkout", ref_name])
                .output()
                .context("Failed to checkout git reference")?;

            if !output.status.success() {
                return Err(anyhow!(
                    "Git checkout failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }

        // Return the path to the module file
        let module_path = repo_dir.join(path);
        if !module_path.exists() {
            return Err(anyhow!("Module file not found in git repository: {}", path));
        }

        Ok(module_path)
    }

    #[cfg(target_arch = "wasm32")]
    fn resolve_git(
        &mut self,
        _url: &str,
        _path: &str,
        _reference: Option<&str>,
    ) -> Result<PathBuf> {
        Err(anyhow!("Git sources are not supported in WASM builds"))
    }

    /// Resolve an HTTP/HTTPS source
    #[cfg(not(target_arch = "wasm32"))]
    fn resolve_http(&mut self, url: &str) -> Result<PathBuf> {
        // Create a cache key from the URL
        let cache_key = format!("{:x}", md5::compute(url.as_bytes()));
        let cache_file = self.cache_dir.join("http").join(&cache_key);

        // Download if not cached
        if !cache_file.exists() {
            fs::create_dir_all(cache_file.parent().unwrap())
                .context("Failed to create cache directory")?;

            // Use curl or wget to download
            let output = std::process::Command::new("curl")
                .args(["-L", "-o", cache_file.to_str().unwrap(), url])
                .output()
                .context("Failed to download module via HTTP")?;

            if !output.status.success() {
                return Err(anyhow!(
                    "HTTP download failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }

        Ok(cache_file)
    }

    #[cfg(target_arch = "wasm32")]
    fn resolve_http(&mut self, _url: &str) -> Result<PathBuf> {
        Err(anyhow!("HTTP sources are not supported in WASM builds"))
    }

    /// Resolve a tarball source
    #[cfg(not(target_arch = "wasm32"))]
    fn resolve_tarball(&mut self, url: &str, path: &str) -> Result<PathBuf> {
        use std::process::Command;

        // Create a cache key from the URL
        let cache_key = format!("{:x}", md5::compute(url.as_bytes()));
        let tarball_dir = self.cache_dir.join("tarball").join(&cache_key);

        // Download and extract if not cached
        if !tarball_dir.exists() {
            fs::create_dir_all(&tarball_dir).context("Failed to create cache directory")?;

            // Download tarball
            let tarball_file = tarball_dir.join("module.tar.gz");
            let output = Command::new("curl")
                .args(["-L", "-o", tarball_file.to_str().unwrap(), url])
                .output()
                .context("Failed to download tarball")?;

            if !output.status.success() {
                return Err(anyhow!(
                    "Tarball download failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            // Extract tarball
            let output = Command::new("tar")
                .args([
                    "-xzf",
                    tarball_file.to_str().unwrap(),
                    "-C",
                    tarball_dir.to_str().unwrap(),
                ])
                .output()
                .context("Failed to extract tarball")?;

            if !output.status.success() {
                return Err(anyhow!(
                    "Tarball extraction failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            // Remove tarball file
            fs::remove_file(tarball_file).ok();
        }

        // Return the path to the module file
        let module_path = tarball_dir.join(path);
        if !module_path.exists() {
            return Err(anyhow!("Module file not found in tarball: {}", path));
        }

        Ok(module_path)
    }

    #[cfg(target_arch = "wasm32")]
    fn resolve_tarball(&mut self, _url: &str, _path: &str) -> Result<PathBuf> {
        Err(anyhow!("Tarball sources are not supported in WASM builds"))
    }

    /// Load a lock file
    pub fn load_lock_file(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path).context("Failed to read lock file")?;
        let lock_file: LockFile =
            serde_json::from_str(&content).context("Failed to parse lock file")?;

        self.lock_entries = lock_file.modules;
        Ok(())
    }

    /// Save a lock file
    pub fn save_lock_file(&self, path: &Path) -> Result<()> {
        let lock_file = LockFile {
            version: "1".to_string(),
            modules: self.lock_entries.clone(),
        };

        let content =
            serde_json::to_string_pretty(&lock_file).context("Failed to serialize lock file")?;

        fs::write(path, content).context("Failed to write lock file")?;
        Ok(())
    }

    /// Add a lock entry
    pub fn add_lock_entry(&mut self, name: String, entry: LockEntry) {
        self.lock_entries.insert(name, entry);
    }

    /// Get the cache directory
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_local_source() {
        let source = "./path/to/module.jcf";
        let parsed = ModuleSourceResolver::parse_source(source).unwrap();
        assert_eq!(
            parsed,
            ModuleSource::Local {
                path: PathBuf::from("./path/to/module.jcf")
            }
        );
    }

    #[test]
    fn test_parse_registry_source() {
        let source = "registry::aws-ec2@^1.0.0";
        let parsed = ModuleSourceResolver::parse_source(source).unwrap();
        assert_eq!(
            parsed,
            ModuleSource::Registry {
                name: "aws-ec2".to_string(),
                version_req: "^1.0.0".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_registry_source_no_version() {
        let source = "registry::aws-ec2";
        let parsed = ModuleSourceResolver::parse_source(source).unwrap();
        assert_eq!(
            parsed,
            ModuleSource::Registry {
                name: "aws-ec2".to_string(),
                version_req: "*".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_git_source() {
        let source = "git::https://github.com/user/repo.git//path/to/module.jcf?ref=v1.0.0";
        let parsed = ModuleSourceResolver::parse_source(source).unwrap();
        assert_eq!(
            parsed,
            ModuleSource::Git {
                url: "https://github.com/user/repo.git".to_string(),
                path: "path/to/module.jcf".to_string(),
                reference: Some("v1.0.0".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_http_source() {
        let source = "https://example.com/modules/module.jcf";
        let parsed = ModuleSourceResolver::parse_source(source).unwrap();
        assert_eq!(
            parsed,
            ModuleSource::Http {
                url: "https://example.com/modules/module.jcf".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_tarball_source() {
        let source = "https://example.com/modules/module.tar.gz//module.jcf";
        let parsed = ModuleSourceResolver::parse_source(source).unwrap();
        assert_eq!(
            parsed,
            ModuleSource::Tarball {
                url: "https://example.com/modules/module.tar.gz".to_string(),
                path: "module.jcf".to_string(),
            }
        );
    }
}
