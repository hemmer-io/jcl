# GitHub Linguist Submission for JCL

This directory contains files for submitting JCL (Jack-of-All Configuration Language) to GitHub Linguist for syntax highlighting support.

## About JCL

**JCL (Jack-of-All Configuration Language)** is a modern, human-readable configuration language with:
- Clean syntax with minimal punctuation
- Advanced static type inference
- 70+ built-in functions
- Powerful module system
- Multi-language bindings (Python, Node.js, Go, Java, Ruby, WASM)

**Project**: https://github.com/hemmer-io/jcl
**Documentation**: https://jcl.hemmer.io
**License**: MIT OR Apache-2.0

## File Extension

**Extension**: `.jcf` (JCL Configuration Format)

**Why `.jcf` instead of `.jcl`?**
The `.jcl` extension is already used by IBM mainframe Job Control Language in GitHub Linguist (language_id: 316620079). To avoid conflicts, we use `.jcf` which has minimal conflicts and clearly indicates it's a configuration format.

## Files in this Submission

### 1. `languages.yml`
Language definition entry to be added to `lib/linguist/languages.yml` in the github/linguist repository.

```yaml
JCL Configuration Language:
  type: data
  color: "#4F9FCF"
  extensions:
  - ".jcf"
  tm_scope: source.jcl
  ace_mode: text
  language_id: # To be generated
  aliases:
  - jcl
  - jcf
```

### 2. `samples/`
Representative JCL code samples showing typical usage:

- **language-showcase.jcf** (412 lines) - Comprehensive language feature demonstration
- **web-server.jcf** (24 lines) - Real-world web server configuration example
- **functions-showcase.jcf** - Built-in functions demonstration

These samples help Linguist's classifier accurately detect `.jcf` files.

### 3. TextMate Grammar
The existing TextMate grammar is located at:
- `editors/vscode/syntaxes/jcl.tmLanguage.json`

This can be referenced by the Linguist team or added using:
```bash
script/add-grammar https://github.com/hemmer-io/jcl
```

## Submission Process

### Prerequisites
JCL meets GitHub Linguist's requirements:
- ✅ Real-world usage on GitHub
- ✅ Open source project (MIT OR Apache-2.0)
- ✅ Existing TextMate grammar for syntax highlighting
- ✅ Representative code samples
- ✅ Clear file extension with minimal conflicts

### Steps

1. **Fork github/linguist**
   ```bash
   gh repo fork github/linguist --clone
   cd linguist
   ```

2. **Add Language Definition**
   Add the entry from `languages.yml` to `lib/linguist/languages.yml`

3. **Add Grammar**
   ```bash
   script/add-grammar https://github.com/hemmer-io/jcl
   ```

4. **Add Samples**
   Copy files from `samples/` to `samples/JCL Configuration Language/`

5. **Generate Language ID**
   ```bash
   script/update-ids
   ```

6. **Test**
   ```bash
   bundle exec rake test
   ```

7. **Submit PR**
   ```bash
   git checkout -b add-jcl-configuration-language
   git add -A
   git commit -m "Add JCL Configuration Language"
   git push origin add-jcl-configuration-language
   gh pr create --title "Add JCL Configuration Language" --body "..."
   ```

## PR Description Template

```markdown
## Summary

Add support for JCL (Jack-of-All Configuration Language), a modern configuration language designed for human readability and type safety.

## Language Information

- **Name**: JCL Configuration Language
- **File Extension**: `.jcf`
- **Type**: Configuration/Data
- **Project**: https://github.com/hemmer-io/jcl
- **Documentation**: https://jcl.hemmer.io
- **License**: MIT OR Apache-2.0

## Why `.jcf` instead of `.jcl`?

The `.jcl` extension is already claimed by IBM mainframe Job Control Language (language_id: 316620079). JCL Configuration Language uses `.jcf` to avoid conflicts while maintaining clear association with the project name.

## Usage Examples

Search results showing `.jcf` usage:
- Primary repository: https://github.com/hemmer-io/jcl (27 `.jcf` files)
- Example configurations in docs and examples

## Sample Code

```jcf
# Web server configuration example
server = (
  host = "0.0.0.0",
  port = 443,
  ssl = true
)

database = (
  type = "postgres",
  host = "db.example.com",
  port = 5432
)

app = (
  name = "MyApp",
  server = server,
  database = database
)
```

## TextMate Grammar

Existing grammar available at: https://github.com/hemmer-io/jcl/tree/main/editors/vscode/syntaxes

## Checklist

- [x] Added language entry to `lib/linguist/languages.yml`
- [x] Added TextMate grammar
- [x] Added representative code samples to `samples/`
- [x] Generated language ID with `script/update-ids`
- [x] Tests pass (`bundle exec rake test`)
- [x] Sample code is licensed appropriately (MIT OR Apache-2.0)

## References

- Project repository: https://github.com/hemmer-io/jcl
- Language specification: https://jcl.hemmer.io/reference/language-spec/
- VSCode extension: Available in project repository
```

## Language Features (for reference)

### Syntax Highlights

**Keywords**: `if`, `then`, `else`, `when`, `for`, `in`, `fn`, `import`, `from`, `as`, `module`

**Literals**: `true`, `false`, `null`

**Operators**: `=>`, `??`, `?.`, `...`, `==`, `!=`, `<=`, `>=`, `and`, `or`, `not`

**Collections**:
- Lists: `[1, 2, 3]`
- Maps: `(key = value, ...)`

**Strings**:
- Double quotes with interpolation: `"Hello, ${name}!"`
- Triple quotes for multi-line: `"""..."""`

**Comments**: `#` to end of line

**Built-in Functions** (70+):
- String: `upper`, `lower`, `trim`, `split`, `join`, `format`
- Encoding: `jsonencode`, `yamlencode`, `base64encode`
- Collections: `map`, `filter`, `reduce`, `merge`, `keys`, `values`
- Numeric: `min`, `max`, `sum`, `avg`, `abs`, `ceil`, `floor`
- Hashing: `md5`, `sha256`, `sha512`
- File: `file`, `fileexists`, `template`

## Contact

For questions about this submission:
- GitHub: https://github.com/hemmer-io/jcl/issues
- Email: info@hemmer.io
