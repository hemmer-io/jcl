# JCL Go Bindings

Go bindings for the Jack-of-All Configuration Language (JCL).

## Installation

```bash
go get github.com/hemmer-io/jcl
```

Before using, build the JCL library:

```bash
cargo build --release --features ffi
```

## Usage

```go
package main

import (
    "fmt"
    "log"

    "github.com/hemmer-io/jcl"
)

func main() {
    // Evaluate JCL code
    config, err := jcl.Eval(`
        name = "my-app"
        version = "1.0.0"
        ports = [80, 443, 8080]
        database = (
            host = "localhost",
            port = 5432
        )
    `)
    if err != nil {
        log.Fatal(err)
    }

    fmt.Println("Name:", config["name"])
    fmt.Println("Ports:", config["ports"])
    fmt.Println("Database:", config["database"])
}
```

## API Reference

### `Parse(source string) (string, error)`

Parse JCL source code and return a summary.

```go
result, err := jcl.Parse(`name = "my-app"`)
if err != nil {
    log.Fatal(err)
}
fmt.Println(result) // "Parsed 1 statements"
```

### `Eval(source string) (map[string]interface{}, error)`

Evaluate JCL source code and return all defined variables.

```go
config, err := jcl.Eval(`
    env = "production"
    debug = false
`)
if err != nil {
    log.Fatal(err)
}

fmt.Println(config["env"])   // "production"
fmt.Println(config["debug"]) // false
```

### `EvalFile(path string) (map[string]interface{}, error)`

Load and evaluate a JCL file.

```go
config, err := jcl.EvalFile("./config.jcf")
if err != nil {
    log.Fatal(err)
}
fmt.Println(config)
```

### `Format(source string) (string, error)`

Format JCL source code.

```go
formatted, err := jcl.Format(`x=1+2
y   ="hello"`)
if err != nil {
    log.Fatal(err)
}
fmt.Println(formatted)
// Output:
// x = 1 + 2
// y = "hello"
```

### `Lint(source string) ([]LintIssue, error)`

Lint JCL source code and return issues.

```go
type LintIssue struct {
    Rule       string `json:"rule"`
    Message    string `json:"message"`
    Severity   string `json:"severity"`
    Suggestion string `json:"suggestion,omitempty"`
}
```

```go
issues, err := jcl.Lint(`
    x = 1
    unused_var = 2
    y = x + 1
`)
if err != nil {
    log.Fatal(err)
}

for _, issue := range issues {
    fmt.Printf("%s: %s\n", issue.Severity, issue.Message)
    if issue.Suggestion != "" {
        fmt.Printf("  Suggestion: %s\n", issue.Suggestion)
    }
}
```

### `Version() string`

Get the JCL version.

```go
fmt.Println("JCL version:", jcl.Version())
```

## Use Cases

### Kubernetes Operator

```go
package main

import (
    "context"
    "log"

    "github.com/hemmer-io/jcl"
    corev1 "k8s.io/api/core/v1"
    metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
)

func createResources(configPath string) error {
    // Load JCL configuration
    config, err := jcl.EvalFile(configPath)
    if err != nil {
        return err
    }

    // Create Kubernetes resources from JCL config
    deployment := &appsv1.Deployment{
        ObjectMeta: metav1.ObjectMeta{
            Name: config["name"].(string),
        },
        Spec: appsv1.DeploymentSpec{
            Replicas: int32Ptr(int32(config["replicas"].(float64))),
            // ... more configuration
        },
    }

    // Apply to cluster
    // ...

    return nil
}
```

### CLI Tool

```go
package main

import (
    "fmt"
    "os"

    "github.com/hemmer-io/jcl"
    "github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
    Use:   "myapp",
    Short: "My application using JCL config",
}

var validateCmd = &cobra.Command{
    Use:   "validate [config-file]",
    Short: "Validate JCL configuration",
    Args:  cobra.ExactArgs(1),
    Run: func(cmd *cobra.Command, args []string) {
        _, err := jcl.EvalFile(args[0])
        if err != nil {
            fmt.Printf("Validation failed: %v\n", err)
            os.Exit(1)
        }
        fmt.Println("Configuration is valid")
    },
}

func main() {
    rootCmd.AddCommand(validateCmd)
    rootCmd.Execute()
}
```

### Terraform Provider

```go
package main

import (
    "github.com/hashicorp/terraform-plugin-sdk/v2/helper/schema"
    "github.com/hemmer-io/jcl"
)

func resourceInfrastructure() *schema.Resource {
    return &schema.Resource{
        Create: resourceInfrastructureCreate,
        Read:   resourceInfrastructureRead,
        Update: resourceInfrastructureUpdate,
        Delete: resourceInfrastructureDelete,

        Schema: map[string]*schema.Schema{
            "config_file": {
                Type:     schema.TypeString,
                Required: true,
            },
        },
    }
}

func resourceInfrastructureCreate(d *schema.ResourceData, m interface{}) error {
    configFile := d.Get("config_file").(string)
    config, err := jcl.EvalFile(configFile)
    if err != nil {
        return err
    }

    // Create infrastructure based on config
    // ...

    return nil
}
```

### Configuration Management

```go
package config

import (
    "os"
    "sync"

    "github.com/hemmer-io/jcl"
)

var (
    cfg  map[string]interface{}
    once sync.Once
)

// Load loads the application configuration.
func Load() (map[string]interface{}, error) {
    var err error
    once.Do(func() {
        env := os.Getenv("APP_ENV")
        if env == "" {
            env = "development"
        }

        cfg, err = jcl.Eval(`
            import * from "./config/base.jcf"
            import * from "./config/` + env + `.jcf"
        `)
    })
    return cfg, err
}

// Get returns a configuration value.
func Get(key string) interface{} {
    cfg, _ := Load()
    return cfg[key]
}
```

## Type Conversions

JCL types are automatically converted to Go types:

| JCL Type | Go Type |
|----------|---------|
| `string` | `string` |
| `int` | `float64` (via JSON) |
| `float` | `float64` |
| `bool` | `bool` |
| `null` | `nil` |
| `list` | `[]interface{}` |
| `map` | `map[string]interface{}` |

**Note:** All numbers come through as `float64` due to JSON unmarshaling. Cast as needed:

```go
port := int(config["port"].(float64))
```

## Building

The Go bindings require the JCL C library to be built:

```bash
# Build the JCL library
cargo build --release --features ffi

# The library will be in target/release/libjcl.so (Linux)
# or target/release/libjcl.dylib (macOS)
# or target/release/jcl.dll (Windows)
```

## Requirements

- Go 1.19 or later
- Rust toolchain (for building the JCL library)
- C compiler (for cgo)

## License

MIT OR Apache-2.0
