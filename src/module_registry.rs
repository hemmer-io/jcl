//! Module registry client for JCL
//!
//! This module provides functionality to interact with JCL module registries,
//! similar to npm, PyPI, or crates.io. It supports:
//! - Module discovery and search
//! - Semantic versioning
//! - Module publishing
//! - Dependency resolution

use anyhow::{anyhow, Context, Result};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Module registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Registry name (e.g., "default", "company-internal")
    pub name: String,

    /// Registry URL (e.g., "https://registry.jcl.io")
    pub url: String,

    /// Authentication token (optional)
    pub token: Option<String>,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            url: "https://registry.jcl.io".to_string(),
            token: None,
        }
    }
}

/// Module metadata in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryModule {
    /// Module name (e.g., "aws-ec2", "kubernetes-config")
    pub name: String,

    /// Latest version
    pub version: Version,

    /// Short description
    pub description: Option<String>,

    /// Module author
    pub author: Option<String>,

    /// License (e.g., "MIT", "Apache-2.0")
    pub license: Option<String>,

    /// Repository URL
    pub repository: Option<String>,

    /// Homepage URL
    pub homepage: Option<String>,

    /// Keywords for search
    pub keywords: Vec<String>,

    /// Download count
    pub downloads: u64,

    /// All available versions
    pub versions: Vec<Version>,
}

/// Module version details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleVersion {
    /// Module name
    pub name: String,

    /// Version
    pub version: Version,

    /// Download URL for this version
    pub download_url: String,

    /// Checksum (SHA-256)
    pub checksum: String,

    /// Dependencies (module name -> version requirement)
    pub dependencies: HashMap<String, VersionReq>,

    /// Published date (ISO 8601)
    pub published_at: String,
}

/// Module search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Matching modules
    pub modules: Vec<RegistryModule>,

    /// Total count (for pagination)
    pub total: usize,
}

/// Module registry client
pub struct RegistryClient {
    /// Registry configuration
    config: RegistryConfig,

    /// Local cache directory
    cache_dir: PathBuf,

    /// HTTP client (using curl for now, could use reqwest in future)
    #[allow(dead_code)]
    http_client: (),
}

impl RegistryClient {
    /// Create a new registry client
    pub fn new(config: RegistryConfig) -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".jcl-cache"))
            .join("jcl")
            .join("registry")
            .join(&config.name);

        Self {
            config,
            cache_dir,
            http_client: (),
        }
    }

    /// Create a new registry client with default configuration
    pub fn default_registry() -> Self {
        Self::new(RegistryConfig::default())
    }

    /// Search for modules in the registry
    pub fn search(&self, query: &str, limit: usize) -> Result<SearchResult> {
        // Build search URL
        let url = format!(
            "{}/api/v1/modules/search?q={}&limit={}",
            self.config.url,
            urlencoding::encode(query),
            limit
        );

        // Make HTTP request
        let response = self.http_get(&url)?;

        // Parse response
        let result: SearchResult =
            serde_json::from_str(&response).context("Failed to parse search results")?;

        Ok(result)
    }

    /// Get module metadata
    pub fn get_module(&self, name: &str) -> Result<RegistryModule> {
        // Build module URL
        let url = format!("{}/api/v1/modules/{}", self.config.url, name);

        // Make HTTP request
        let response = self.http_get(&url)?;

        // Parse response
        let module: RegistryModule =
            serde_json::from_str(&response).context("Failed to parse module metadata")?;

        Ok(module)
    }

    /// Get specific version of a module
    pub fn get_module_version(&self, name: &str, version: &Version) -> Result<ModuleVersion> {
        // Build version URL
        let url = format!(
            "{}/api/v1/modules/{}/versions/{}",
            self.config.url, name, version
        );

        // Make HTTP request
        let response = self.http_get(&url)?;

        // Parse response
        let module_version: ModuleVersion =
            serde_json::from_str(&response).context("Failed to parse module version")?;

        Ok(module_version)
    }

    /// Resolve version requirement to a specific version
    pub fn resolve_version(&self, name: &str, req: &VersionReq) -> Result<Version> {
        // Get module metadata
        let module = self.get_module(name)?;

        // Find the highest version that satisfies the requirement
        let matching_version = module
            .versions
            .iter()
            .filter(|v| req.matches(v))
            .max()
            .ok_or_else(|| anyhow!("No version of '{}' satisfies requirement '{}'", name, req))?;

        Ok(matching_version.clone())
    }

    /// Download a module version
    pub fn download(&self, name: &str, version: &Version) -> Result<PathBuf> {
        // Get version metadata
        let module_version = self.get_module_version(name, version)?;

        // Check if already cached
        let cache_path = self.cache_dir.join(name).join(version.to_string());

        if cache_path.exists() {
            return Ok(cache_path);
        }

        // Create cache directory
        fs::create_dir_all(&cache_path).context("Failed to create cache directory")?;

        // Download tarball
        let tarball_path = cache_path.join("module.tar.gz");
        self.http_download(&module_version.download_url, &tarball_path)?;

        // Verify checksum
        let actual_checksum = self.compute_checksum(&tarball_path)?;
        if actual_checksum != module_version.checksum {
            return Err(anyhow!(
                "Checksum mismatch for {}: expected {}, got {}",
                name,
                module_version.checksum,
                actual_checksum
            ));
        }

        // Extract tarball
        self.extract_tarball(&tarball_path, &cache_path)?;

        // Remove tarball
        fs::remove_file(&tarball_path).ok();

        Ok(cache_path)
    }

    /// Publish a module to the registry
    pub fn publish(&self, module_path: &Path) -> Result<()> {
        // Verify authentication token
        let token = self
            .config
            .token
            .as_ref()
            .ok_or_else(|| anyhow!("Authentication token required for publishing"))?;

        // Read module manifest
        let manifest_path = module_path.join("jcl.json");
        let manifest_content = fs::read_to_string(&manifest_path)
            .context("Failed to read module manifest (jcl.json)")?;

        let manifest: ModuleManifest =
            serde_json::from_str(&manifest_content).context("Failed to parse module manifest")?;

        // Create tarball
        let tarball_path = self.create_tarball(module_path, &manifest)?;

        // Compute checksum
        let checksum = self.compute_checksum(&tarball_path)?;

        // Build publish URL
        let url = format!("{}/api/v1/modules/publish", self.config.url);

        // Upload tarball
        self.http_upload(&url, &tarball_path, token, &manifest, &checksum)?;

        // Cleanup
        fs::remove_file(&tarball_path).ok();

        println!(
            "Successfully published {} v{}",
            manifest.name, manifest.version
        );

        Ok(())
    }

    /// List all versions of a module
    pub fn list_versions(&self, name: &str) -> Result<Vec<Version>> {
        let module = self.get_module(name)?;
        let mut versions = module.versions;
        versions.sort();
        versions.reverse(); // Newest first
        Ok(versions)
    }

    // Private helper methods

    #[cfg(not(target_arch = "wasm32"))]
    fn http_get(&self, url: &str) -> Result<String> {
        use std::process::Command;

        let mut cmd = Command::new("curl");
        cmd.arg("-s") // Silent
            .arg("-L"); // Follow redirects

        // Add auth token if present
        if let Some(token) = &self.config.token {
            cmd.arg("-H")
                .arg(format!("Authorization: Bearer {}", token));
        }

        cmd.arg(url);

        let output = cmd.output().context("Failed to execute curl")?;

        if !output.status.success() {
            return Err(anyhow!(
                "HTTP request failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    #[cfg(target_arch = "wasm32")]
    fn http_get(&self, _url: &str) -> Result<String> {
        Err(anyhow!("HTTP requests not supported in WASM builds"))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn http_download(&self, url: &str, dest: &Path) -> Result<()> {
        use std::process::Command;

        let output = Command::new("curl")
            .arg("-s")
            .arg("-L")
            .arg("-o")
            .arg(dest)
            .arg(url)
            .output()
            .context("Failed to download file")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Download failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn http_download(&self, _url: &str, _dest: &Path) -> Result<()> {
        Err(anyhow!("HTTP downloads not supported in WASM builds"))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn http_upload(
        &self,
        url: &str,
        tarball: &Path,
        token: &str,
        manifest: &ModuleManifest,
        checksum: &str,
    ) -> Result<()> {
        use std::process::Command;

        let output = Command::new("curl")
            .arg("-s")
            .arg("-X")
            .arg("POST")
            .arg("-H")
            .arg(format!("Authorization: Bearer {}", token))
            .arg("-F")
            .arg(format!("name={}", manifest.name))
            .arg("-F")
            .arg(format!("version={}", manifest.version))
            .arg("-F")
            .arg(format!("checksum={}", checksum))
            .arg("-F")
            .arg(format!("file=@{}", tarball.display()))
            .arg(url)
            .output()
            .context("Failed to upload module")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Upload failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn http_upload(
        &self,
        _url: &str,
        _tarball: &Path,
        _token: &str,
        _manifest: &ModuleManifest,
        _checksum: &str,
    ) -> Result<()> {
        Err(anyhow!("HTTP uploads not supported in WASM builds"))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn extract_tarball(&self, tarball: &Path, dest: &Path) -> Result<()> {
        use std::process::Command;

        let output = Command::new("tar")
            .arg("-xzf")
            .arg(tarball)
            .arg("-C")
            .arg(dest)
            .output()
            .context("Failed to extract tarball")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Extraction failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn extract_tarball(&self, _tarball: &Path, _dest: &Path) -> Result<()> {
        Err(anyhow!("Tarball extraction not supported in WASM builds"))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn create_tarball(&self, source: &Path, manifest: &ModuleManifest) -> Result<PathBuf> {
        use std::process::Command;

        let tarball_name = format!("{}-{}.tar.gz", manifest.name, manifest.version);
        let tarball_path = std::env::temp_dir().join(&tarball_name);

        let output = Command::new("tar")
            .arg("-czf")
            .arg(&tarball_path)
            .arg("-C")
            .arg(source.parent().unwrap())
            .arg(source.file_name().unwrap())
            .output()
            .context("Failed to create tarball")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Tarball creation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(tarball_path)
    }

    #[cfg(target_arch = "wasm32")]
    fn create_tarball(&self, _source: &Path, _manifest: &ModuleManifest) -> Result<PathBuf> {
        Err(anyhow!("Tarball creation not supported in WASM builds"))
    }

    fn compute_checksum(&self, path: &Path) -> Result<String> {
        use sha2::{Digest, Sha256};

        let content = fs::read(path).context("Failed to read file for checksum")?;
        let hash = Sha256::digest(&content);
        Ok(format!("{:x}", hash))
    }
}

/// Module manifest (jcl.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleManifest {
    /// Module name (must be unique in registry)
    pub name: String,

    /// Version (semantic versioning)
    pub version: Version,

    /// Short description
    pub description: Option<String>,

    /// Author
    pub author: Option<String>,

    /// License
    pub license: Option<String>,

    /// Repository URL
    pub repository: Option<String>,

    /// Homepage URL
    pub homepage: Option<String>,

    /// Keywords
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Dependencies (module name -> version requirement)
    #[serde(default)]
    pub dependencies: HashMap<String, VersionReq>,

    /// Main module file
    #[serde(default = "default_main")]
    pub main: String,
}

fn default_main() -> String {
    "module.jcl".to_string()
}

impl ModuleManifest {
    /// Create a new module manifest
    pub fn new(name: String, version: Version) -> Self {
        Self {
            name,
            version,
            description: None,
            author: None,
            license: None,
            repository: None,
            homepage: None,
            keywords: Vec::new(),
            dependencies: HashMap::new(),
            main: default_main(),
        }
    }

    /// Load manifest from file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).context("Failed to read manifest file")?;

        let manifest: Self = serde_json::from_str(&content).context("Failed to parse manifest")?;

        Ok(manifest)
    }

    /// Save manifest to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self).context("Failed to serialize manifest")?;

        fs::write(path, content).context("Failed to write manifest file")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_serialization() {
        let manifest = ModuleManifest::new("test-module".to_string(), Version::new(1, 0, 0));

        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: ModuleManifest = serde_json::from_str(&json).unwrap();

        assert_eq!(manifest.name, parsed.name);
        assert_eq!(manifest.version, parsed.version);
    }

    #[test]
    fn test_version_requirement() {
        let req = VersionReq::parse("^1.0.0").unwrap();

        assert!(req.matches(&Version::new(1, 0, 0)));
        assert!(req.matches(&Version::new(1, 2, 3)));
        assert!(!req.matches(&Version::new(2, 0, 0)));
    }

    #[test]
    fn test_default_registry_config() {
        let config = RegistryConfig::default();
        assert_eq!(config.name, "default");
        assert_eq!(config.url, "https://registry.jcl.io");
    }
}
