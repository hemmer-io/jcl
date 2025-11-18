# JCL Java Bindings

Java bindings for JCL (Jack-of-All Configuration Language) using JNI (Java Native Interface).

## Installation

### Maven

Add the following dependency to your `pom.xml`:

```xml
<dependency>
    <groupId>io.hemmer</groupId>
    <artifactId>jcl</artifactId>
    <version>0.1.0</version>
</dependency>

<!-- Also add Gson for JSON handling -->
<dependency>
    <groupId>com.google.gson</groupId>
    <artifactId>gson</artifactId>
    <version>2.10.1</version>
</dependency>
```

### Gradle

Add the following to your `build.gradle`:

```gradle
dependencies {
    implementation 'io.hemmer:jcl:0.1.0'
    implementation 'com.google.gson:gson:2.10.1'
}
```

### Building from Source

1. Build the native library:
```bash
cargo build --release --features java
```

2. The native library will be in `target/release/libjcl.so` (Linux), `libjcl.dylib` (macOS), or `jcl.dll` (Windows)

3. Ensure the native library is in your Java library path or use `-Djava.library.path=/path/to/lib`

## Usage

### Basic Example

```java
import java.util.Map;

public class Example {
    public static void main(String[] args) {
        // Evaluate JCL code
        String jclCode = """
            version = "1.0.0"
            port = 8080
            debug = true
            """;

        Map<String, Object> config = JCL.evalToMap(jclCode);

        System.out.println("Version: " + config.get("version"));
        System.out.println("Port: " + config.get("port"));
        System.out.println("Debug: " + config.get("debug"));
    }
}
```

### Parsing

```java
// Parse JCL code to check syntax
String result = JCL.parse("version = \"1.0.0\"");
System.out.println(result); // "Parsed 1 statements"
```

### Evaluation

```java
// Evaluate JCL and get results as Map
String jclCode = """
    app = {
        name = "MyApp"
        port = 8080
        features = ["auth", "api", "admin"]
    }
    """;

Map<String, Object> config = JCL.evalToMap(jclCode);

// Access nested values
Map<String, Object> app = (Map<String, Object>) config.get("app");
System.out.println("App name: " + app.get("name"));

// Evaluate from file
Map<String, Object> fileConfig = JCL.evalFileToMap("config.jcl");
```

### Using JsonObject

```java
import com.google.gson.JsonObject;
import com.google.gson.JsonArray;

// Get results as Gson JsonObject
JsonObject config = JCL.evalToJson(jclCode);

// Access values with type safety
String name = config.getAsJsonObject("app").get("name").getAsString();
int port = config.getAsJsonObject("app").get("port").getAsInt();
JsonArray features = config.getAsJsonObject("app").getAsJsonArray("features");
```

### Formatting

```java
String unformatted = "version=\"1.0.0\" port=8080";
String formatted = JCL.format(unformatted);
System.out.println(formatted);
// Output:
// version = "1.0.0"
// port = 8080
```

### Linting

```java
import java.util.List;

String jclCode = """
    unused_var = 42
    result = 10
    """;

// Get lint issues as List
List<JCL.LintIssue> issues = JCL.lintToList(jclCode);

for (JCL.LintIssue issue : issues) {
    System.out.println(issue.getSeverity() + ": " + issue.getMessage());
    if (issue.getSuggestion() != null) {
        System.out.println("  Suggestion: " + issue.getSuggestion());
    }
}

// Or get as JsonArray
JsonArray issuesJson = JCL.lintToJson(jclCode);
```

### Version Information

```java
String version = JCL.version();
System.out.println("JCL version: " + version);
```

## API Reference

### Static Methods

#### `String parse(String source)`
Parse JCL source code and return a status message.
- **Parameters:** `source` - The JCL source code
- **Returns:** Status message with number of statements parsed
- **Throws:** `RuntimeException` on parse error

#### `Map<String, Object> evalToMap(String source)`
Evaluate JCL source code and return variables as a Map.
- **Parameters:** `source` - The JCL source code
- **Returns:** Map containing evaluated variables
- **Throws:** `RuntimeException` on evaluation error

#### `Map<String, Object> evalFileToMap(String path)`
Evaluate JCL from a file and return variables as a Map.
- **Parameters:** `path` - Path to the JCL file
- **Returns:** Map containing evaluated variables
- **Throws:** `RuntimeException` on file read or evaluation error

#### `JsonObject evalToJson(String source)`
Evaluate JCL source code and return variables as a JsonObject.
- **Parameters:** `source` - The JCL source code
- **Returns:** JsonObject containing evaluated variables
- **Throws:** `RuntimeException` on evaluation error

#### `JsonObject evalFileToJson(String path)`
Evaluate JCL from a file and return variables as a JsonObject.
- **Parameters:** `path` - Path to the JCL file
- **Returns:** JsonObject containing evaluated variables
- **Throws:** `RuntimeException` on file read or evaluation error

#### `String format(String source)`
Format JCL source code.
- **Parameters:** `source` - The JCL source code
- **Returns:** Formatted JCL source code
- **Throws:** `RuntimeException` on format error

#### `List<LintIssue> lintToList(String source)`
Lint JCL source code and return issues as a List.
- **Parameters:** `source` - The JCL source code
- **Returns:** List of LintIssue objects
- **Throws:** `RuntimeException` on lint error

#### `JsonArray lintToJson(String source)`
Lint JCL source code and return issues as a JsonArray.
- **Parameters:** `source` - The JCL source code
- **Returns:** JsonArray containing lint issues
- **Throws:** `RuntimeException` on lint error

#### `String version()`
Get the JCL version.
- **Returns:** Version string

### LintIssue Class

Represents a linting issue found in JCL code.

#### Properties
- `String rule` - The rule that was violated
- `String message` - Description of the issue
- `String severity` - Severity level: "error", "warning", or "info"
- `String suggestion` - Optional suggestion for fixing the issue

## Type Conversions

JCL types are converted to Java types as follows:

| JCL Type | Java Type |
|----------|-----------|
| String | `String` |
| Int | `Long` or `Integer` |
| Float | `Double` |
| Bool | `Boolean` |
| Null | `null` |
| List | `List<Object>` |
| Map | `Map<String, Object>` |
| Function | `String` ("<function>") |

When using `evalToJson()`, types are converted to Gson's `JsonElement` types:
- `JsonPrimitive` for strings, numbers, and booleans
- `JsonNull` for null values
- `JsonArray` for lists
- `JsonObject` for maps

## Use Cases

### Spring Boot Configuration

```java
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Bean;

@Configuration
public class JclConfig {

    @Bean
    public AppConfiguration loadConfig() {
        Map<String, Object> config = JCL.evalFileToMap("application.jcl");

        AppConfiguration appConfig = new AppConfiguration();
        appConfig.setPort((Integer) config.get("port"));
        appConfig.setDebug((Boolean) config.get("debug"));

        return appConfig;
    }
}
```

### Android Application

```java
import android.app.Application;

public class MyApplication extends Application {
    private Map<String, Object> config;

    @Override
    public void onCreate() {
        super.onCreate();

        // Load config from assets
        try {
            InputStream is = getAssets().open("config.jcl");
            String jclCode = new String(is.readAllBytes());
            config = JCL.evalToMap(jclCode);
        } catch (IOException e) {
            e.printStackTrace();
        }
    }

    public Map<String, Object> getConfig() {
        return config;
    }
}
```

### Maven Plugin

```java
import org.apache.maven.plugin.AbstractMojo;
import org.apache.maven.plugin.MojoExecutionException;
import org.apache.maven.plugins.annotations.Mojo;
import org.apache.maven.plugins.annotations.Parameter;

@Mojo(name = "validate-jcl")
public class JclValidateMojo extends AbstractMojo {

    @Parameter(property = "jcl.source", required = true)
    private String source;

    public void execute() throws MojoExecutionException {
        try {
            List<JCL.LintIssue> issues = JCL.lintToList(
                new String(Files.readAllBytes(Paths.get(source)))
            );

            for (JCL.LintIssue issue : issues) {
                if ("error".equals(issue.getSeverity())) {
                    throw new MojoExecutionException("JCL validation failed: " + issue.getMessage());
                } else {
                    getLog().warn(issue.getMessage());
                }
            }
        } catch (IOException e) {
            throw new MojoExecutionException("Failed to read JCL file", e);
        }
    }
}
```

### Gradle Task

```java
import org.gradle.api.DefaultTask;
import org.gradle.api.tasks.TaskAction;

public class JclFormatTask extends DefaultTask {

    @TaskAction
    public void format() {
        getProject().fileTree("src/main/resources")
            .include("**/*.jcl")
            .forEach(file -> {
                try {
                    String content = new String(Files.readAllBytes(file.toPath()));
                    String formatted = JCL.format(content);
                    Files.write(file.toPath(), formatted.getBytes());
                    System.out.println("Formatted: " + file.getName());
                } catch (IOException e) {
                    e.printStackTrace();
                }
            });
    }
}
```

## Error Handling

All methods throw `RuntimeException` on errors. It's recommended to catch and handle these appropriately:

```java
try {
    Map<String, Object> config = JCL.evalFileToMap("config.jcl");
} catch (RuntimeException e) {
    if (e.getMessage().contains("Parse error")) {
        System.err.println("Invalid JCL syntax: " + e.getMessage());
    } else if (e.getMessage().contains("Failed to read file")) {
        System.err.println("File not found: " + e.getMessage());
    } else {
        System.err.println("Evaluation error: " + e.getMessage());
    }
}
```

## Performance Considerations

- The native library is loaded once per JVM process
- Parsing and evaluation are performed in native code for optimal performance
- For repeated evaluations, consider caching parsed results
- Large configurations benefit from streaming when possible

## Thread Safety

The JCL Java bindings are thread-safe. Multiple threads can safely call JCL methods concurrently.

## License

MIT OR Apache-2.0
