//! JCL Module Management CLI
//!
//! Commands for managing JCL modules:
//! - init: Scaffold a new module
//! - validate: Validate module structure
//! - get: Download module dependencies
//! - list: List installed modules

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use jcl::module_registry::{ModuleManifest, RegistryClient};
use semver::Version;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "jcl-module")]
#[command(about = "JCL module management", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new JCL module
    Init {
        /// Module name
        #[arg(value_name = "NAME")]
        name: String,

        /// Directory to create module in (defaults to module name)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Module version (default: 0.1.0)
        #[arg(short, long, default_value = "0.1.0")]
        version: String,

        /// Module description
        #[arg(short, long)]
        description: Option<String>,

        /// Module author
        #[arg(short, long)]
        author: Option<String>,

        /// Module license (default: MIT)
        #[arg(short, long, default_value = "MIT")]
        license: String,
    },

    /// Validate module structure and manifest
    Validate {
        /// Path to module directory (defaults to current directory)
        #[arg(value_name = "PATH", default_value = ".")]
        path: PathBuf,
    },

    /// Download module dependencies
    Get {
        /// Path to module directory (defaults to current directory)
        #[arg(value_name = "PATH", default_value = ".")]
        path: PathBuf,
    },

    /// List installed modules
    List {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            name,
            path,
            version,
            description,
            author,
            license,
        } => cmd_init(name, path, version, description, author, license),
        Commands::Validate { path } => cmd_validate(path),
        Commands::Get { path } => cmd_get(path),
        Commands::List { verbose } => cmd_list(verbose),
    }
}

/// Initialize a new module
fn cmd_init(
    name: String,
    path: Option<PathBuf>,
    version_str: String,
    description: Option<String>,
    author: Option<String>,
    license: String,
) -> Result<()> {
    // Parse version
    let version = Version::parse(&version_str)
        .context("Invalid version format. Use semantic versioning (e.g., 1.0.0)")?;

    // Determine module directory
    let module_dir = path.unwrap_or_else(|| PathBuf::from(&name));

    // Create module directory
    if module_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", module_dir.display());
    }

    fs::create_dir_all(&module_dir).context("Failed to create module directory")?;

    println!("Creating module '{}' in {}", name, module_dir.display());

    // Create manifest
    let manifest = ModuleManifest {
        name: name.clone(),
        version,
        description,
        author,
        license: Some(license),
        repository: None,
        homepage: None,
        keywords: Vec::new(),
        dependencies: HashMap::new(),
        main: "module.jcl".to_string(),
    };

    let manifest_path = module_dir.join("jcl.json");
    manifest
        .save(&manifest_path)
        .context("Failed to write manifest")?;

    println!("  ✓ Created jcl.json");

    // Create main module file with template
    let module_template = format!(
        r#"# {} Module

module.interface = (
    inputs = (
        # Define your module inputs here
        # Example:
        # name = (type = string, required = true, description = "Resource name")
    ),
    outputs = (
        # Define your module outputs here
        # Example:
        # id = (type = string, description = "Resource ID")
    )
)

module.outputs = (
    # Implement your module outputs here
    # Example:
    # id = "resource-${{module.inputs.name}}"
)
"#,
        name
    );

    let main_path = module_dir.join("module.jcl");
    fs::write(&main_path, module_template).context("Failed to write main module file")?;

    println!("  ✓ Created module.jcl");

    // Create README
    let readme_template = format!(
        r#"# {}

{}

## Usage

```jcl
module.example.instance = (
    source = "./path/to/{}",
    # Add your inputs here
)

# Access outputs
result = module.example.instance.id
```

## Inputs

(Document your inputs here)

## Outputs

(Document your outputs here)

## License

{}
"#,
        name,
        manifest
            .description
            .as_deref()
            .unwrap_or("Description of this module"),
        name,
        manifest.license.as_deref().unwrap_or("MIT")
    );

    let readme_path = module_dir.join("README.md");
    fs::write(&readme_path, readme_template).context("Failed to write README")?;

    println!("  ✓ Created README.md");

    // Create .gitignore
    let gitignore_content = r#"# JCL cache
.jcl-cache/
.jcl.lock

# OS files
.DS_Store
Thumbs.db
"#;

    let gitignore_path = module_dir.join(".gitignore");
    fs::write(&gitignore_path, gitignore_content).context("Failed to write .gitignore")?;

    println!("  ✓ Created .gitignore");

    println!("\n✓ Module '{}' initialized successfully!", name);
    println!("\nNext steps:");
    println!("  1. cd {}", module_dir.display());
    println!("  2. Edit module.jcl to define your module");
    println!("  3. Run 'jcl-module validate' to check your module");

    Ok(())
}

/// Validate module structure
fn cmd_validate(path: PathBuf) -> Result<()> {
    println!("Validating module in {}", path.display());

    // Check if directory exists
    if !path.exists() {
        anyhow::bail!("Directory '{}' does not exist", path.display());
    }

    if !path.is_dir() {
        anyhow::bail!("'{}' is not a directory", path.display());
    }

    // Check for manifest
    let manifest_path = path.join("jcl.json");
    if !manifest_path.exists() {
        anyhow::bail!("Missing jcl.json manifest file");
    }

    // Load and validate manifest
    let manifest = ModuleManifest::load(&manifest_path).context("Failed to load manifest")?;

    println!("  ✓ Valid manifest (jcl.json)");
    println!("    Name: {}", manifest.name);
    println!("    Version: {}", manifest.version);

    // Check for main module file
    let main_path = path.join(&manifest.main);
    if !main_path.exists() {
        anyhow::bail!("Main module file '{}' not found", manifest.main);
    }

    println!("  ✓ Main module file exists ({})", manifest.main);

    // Parse main module file
    let module_ast = jcl::parse_file(&main_path).context("Failed to parse main module file")?;

    println!("  ✓ Module file parses successfully");

    // Check for module.interface
    let has_interface = module_ast
        .statements
        .iter()
        .any(|stmt| matches!(stmt, jcl::ast::Statement::ModuleInterface { .. }));

    if !has_interface {
        println!("  ⚠ Warning: No module.interface defined");
    } else {
        println!("  ✓ Module interface defined");
    }

    // Check for module.outputs
    let has_outputs = module_ast
        .statements
        .iter()
        .any(|stmt| matches!(stmt, jcl::ast::Statement::ModuleOutputs { .. }));

    if !has_outputs {
        println!("  ⚠ Warning: No module.outputs defined");
    } else {
        println!("  ✓ Module outputs defined");
    }

    // Validate dependencies
    if !manifest.dependencies.is_empty() {
        println!("\n  Dependencies:");
        for (name, version_req) in &manifest.dependencies {
            println!("    {} @ {}", name, version_req);
        }
    }

    println!("\n✓ Module validation successful!");

    Ok(())
}

/// Download module dependencies
fn cmd_get(path: PathBuf) -> Result<()> {
    println!("Downloading dependencies for module in {}", path.display());

    // Load manifest
    let manifest_path = path.join("jcl.json");
    let manifest = ModuleManifest::load(&manifest_path)
        .context("Failed to load manifest. Run 'jcl-module validate' first.")?;

    if manifest.dependencies.is_empty() {
        println!("  No dependencies to download");
        return Ok(());
    }

    // Create registry client
    let client = RegistryClient::default_registry();

    // Download each dependency
    for (name, version_req) in &manifest.dependencies {
        println!("\n  Resolving {} @ {}...", name, version_req);

        // Resolve version
        let version = client
            .resolve_version(name, version_req)
            .context(format!("Failed to resolve version for '{}'", name))?;

        println!("    → Resolved to v{}", version);

        // Download module
        println!("    Downloading...");
        let module_dir = client
            .download(name, &version)
            .context(format!("Failed to download '{}'", name))?;

        println!("    ✓ Downloaded to {}", module_dir.display());
    }

    println!("\n✓ All dependencies downloaded successfully!");

    Ok(())
}

/// List installed modules
fn cmd_list(verbose: bool) -> Result<()> {
    println!("Installed modules:");

    // Get cache directory
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".jcl-cache"))
        .join("jcl")
        .join("registry")
        .join("default");

    if !cache_dir.exists() {
        println!("  No modules installed yet");
        return Ok(());
    }

    // List module directories
    let entries = fs::read_dir(&cache_dir).context("Failed to read cache directory")?;

    let mut count = 0;
    for entry in entries {
        let entry = entry?;
        let module_name = entry.file_name();
        let module_path = entry.path();

        if !module_path.is_dir() {
            continue;
        }

        count += 1;

        if verbose {
            // List versions
            let versions = fs::read_dir(&module_path)?;
            println!("\n  {}", module_name.to_string_lossy());

            for version_entry in versions {
                let version_entry = version_entry?;
                let version_name = version_entry.file_name();
                let version_path = version_entry.path();

                if !version_path.is_dir() {
                    continue;
                }

                // Try to load manifest for details
                let manifest_path = version_path.join("jcl.json");
                if let Ok(manifest) = ModuleManifest::load(&manifest_path) {
                    println!("    v{}", version_name.to_string_lossy());
                    if let Some(desc) = &manifest.description {
                        println!("      {}", desc);
                    }
                } else {
                    println!("    v{}", version_name.to_string_lossy());
                }
            }
        } else {
            println!("  {}", module_name.to_string_lossy());
        }
    }

    if count == 0 {
        println!("  No modules installed yet");
    } else {
        println!("\nTotal: {} module(s)", count);
    }

    Ok(())
}
