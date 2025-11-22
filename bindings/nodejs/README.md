# JCL Node.js Bindings

Node.js/JavaScript/TypeScript bindings for the Jack-of-All Configuration Language (JCL).

## Installation

```bash
npm install @hemmerio/jcl
# or
yarn add @hemmerio/jcl
```

## Usage

### JavaScript

```javascript
const jcl = require('@hemmerio/jcl');

// Evaluate JCL code
const config = jcl.eval(`
  name = "my-app"
  version = "1.0.0"
  ports = [80, 443, 8080]
  database = (
    host = "localhost",
    port = 5432
  )
`);

console.log(config.name);        // "my-app"
console.log(config.ports);       // [80, 443, 8080]
console.log(config.database);    // { host: 'localhost', port: 5432 }
```

### TypeScript

```typescript
import * as jcl from '@hemmerio/jcl';

interface Config {
  name: string;
  version: string;
  ports: number[];
  database: {
    host: string;
    port: number;
  };
}

const config = jcl.eval(`
  name = "my-app"
  version = "1.0.0"
  ports = [80, 443, 8080]
  database = (
    host = "localhost",
    port = 5432
  )
`) as Config;

console.log(config.name);        // Type-safe!
console.log(config.database.host);
```

## API Reference

### `parse(source: string): string`

Parse JCL source code and return a summary.

```javascript
const result = jcl.parse('name = "my-app"');
console.log(result); // "Parsed 1 statements"
```

### `eval(source: string): object`

Evaluate JCL source code and return all defined variables.

```javascript
const config = jcl.eval(`
  env = "production"
  debug = false
`);
console.log(config); // { env: 'production', debug: false }
```

### `evalFile(path: string): object`

Load and evaluate a JCL file.

```javascript
const config = jcl.evalFile('./config.jcf');
console.log(config);
```

### `format(source: string): string`

Format JCL source code.

```javascript
const formatted = jcl.format('x=1+2\ny   ="hello"');
console.log(formatted);
// Output:
// x = 1 + 2
// y = "hello"
```

### `lint(source: string): LintIssue[]`

Lint JCL source code and return issues.

```typescript
interface LintIssue {
  rule: string;
  message: string;
  severity: 'error' | 'warning' | 'info';
  suggestion?: string;
}
```

```javascript
const issues = jcl.lint(`
  x = 1
  unused_var = 2
  y = x + 1
`);

for (const issue of issues) {
  console.log(`${issue.severity}: ${issue.message}`);
}
```

## Use Cases

### VS Code Extension

```typescript
import * as jcl from '@hemmerio/jcl';
import * as vscode from 'vscode';

export function activate(context: vscode.ExtensionContext) {
  // Format on save
  context.subscriptions.push(
    vscode.languages.registerDocumentFormattingEditProvider('jcl', {
      provideDocumentFormattingEdits(document) {
        const text = document.getText();
        const formatted = jcl.format(text);
        // ... apply edits
      }
    })
  );

  // Diagnostics (linting)
  const diagnostics = vscode.languages.createDiagnosticCollection('jcl');
  context.subscriptions.push(diagnostics);

  function updateDiagnostics(document: vscode.TextDocument) {
    const issues = jcl.lint(document.getText());
    const vscodeDiagnostics = issues.map(issue => {
      // Convert to VS Code diagnostic
    });
    diagnostics.set(document.uri, vscodeDiagnostics);
  }
}
```

### Build Tool / Webpack Plugin

```javascript
const jcl = require('@hemmerio/jcl');

class JclWebpackPlugin {
  apply(compiler) {
    compiler.hooks.beforeCompile.tapAsync('JclWebpackPlugin', (params, callback) => {
      const config = jcl.evalFile('./build.jcf');
      // Use config to modify webpack configuration
      callback();
    });
  }
}
```

### Express.js Configuration

```javascript
const express = require('express');
const jcl = require('@hemmerio/jcl');

const config = jcl.evalFile('./server.jcf');

const app = express();
app.listen(config.port, () => {
  console.log(`Server running on port ${config.port}`);
});
```

### Configuration Management

```javascript
const jcl = require('@hemmerio/jcl');

// Load environment-specific config
const env = process.env.NODE_ENV || 'development';
const config = jcl.eval(`
  import * from "./config/base.jcf"
  import * from "./config/${env}.jcf"

  # Override with environment variables
  port = env("PORT", port)
  database_url = env("DATABASE_URL", database_url)
`);

module.exports = config;
```

## Type Conversions

JCL types are automatically converted to JavaScript types:

| JCL Type | JavaScript Type |
|----------|-----------------|
| `string` | `string` |
| `int` | `number` |
| `float` | `number` |
| `bool` | `boolean` |
| `null` | `null` |
| `list` | `Array` |
| `map` | `Object` |
| `function` | `string` (displays as "&lt;function&gt;") |

## Building from Source

```bash
# Install dependencies
npm install

# Build
npm run build

# Build release
npm run build-release
```

## Requirements

- Node.js >= 12
- Rust toolchain (for building from source)

## License

MIT OR Apache-2.0
