import com.google.gson.Gson;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.reflect.TypeToken;
import java.lang.reflect.Type;
import java.util.Map;
import java.util.List;

/**
 * JCL - Jack-of-All Configuration Language
 *
 * Java bindings for the JCL configuration language.
 * Provides methods to parse, evaluate, format, and lint JCL code.
 */
public class JCL {
    private static final Gson gson = new Gson();

    static {
        // Load the native library
        System.loadLibrary("jcl");
    }

    /**
     * Parse JCL source code.
     *
     * @param source The JCL source code to parse
     * @return A status message indicating the number of statements parsed
     * @throws RuntimeException if parsing fails
     */
    public static native String parse(String source);

    /**
     * Evaluate JCL source code and return variables as a JSON string.
     *
     * @param source The JCL source code to evaluate
     * @return JSON string containing the evaluated variables
     * @throws RuntimeException if evaluation fails
     */
    private static native String eval(String source);

    /**
     * Evaluate JCL from a file and return variables as a JSON string.
     *
     * @param path Path to the JCL file
     * @return JSON string containing the evaluated variables
     * @throws RuntimeException if evaluation fails
     */
    private static native String evalFile(String path);

    /**
     * Format JCL source code.
     *
     * @param source The JCL source code to format
     * @return The formatted JCL source code
     * @throws RuntimeException if formatting fails
     */
    public static native String format(String source);

    /**
     * Lint JCL source code and return issues as a JSON string.
     *
     * @param source The JCL source code to lint
     * @return JSON array string containing lint issues
     * @throws RuntimeException if linting fails
     */
    private static native String lint(String source);

    /**
     * Get the JCL version.
     *
     * @return The version string
     */
    public static native String version();

    /**
     * Evaluate JCL source code and return variables as a Map.
     *
     * @param source The JCL source code to evaluate
     * @return Map containing the evaluated variables
     * @throws RuntimeException if evaluation fails
     */
    public static Map<String, Object> evalToMap(String source) {
        String json = eval(source);
        Type type = new TypeToken<Map<String, Object>>(){}.getType();
        return gson.fromJson(json, type);
    }

    /**
     * Evaluate JCL from a file and return variables as a Map.
     *
     * @param path Path to the JCL file
     * @return Map containing the evaluated variables
     * @throws RuntimeException if evaluation fails
     */
    public static Map<String, Object> evalFileToMap(String path) {
        String json = evalFile(path);
        Type type = new TypeToken<Map<String, Object>>(){}.getType();
        return gson.fromJson(json, type);
    }

    /**
     * Evaluate JCL source code and return variables as a JsonObject.
     *
     * @param source The JCL source code to evaluate
     * @return JsonObject containing the evaluated variables
     * @throws RuntimeException if evaluation fails
     */
    public static JsonObject evalToJson(String source) {
        String json = eval(source);
        return gson.fromJson(json, JsonObject.class);
    }

    /**
     * Evaluate JCL from a file and return variables as a JsonObject.
     *
     * @param path Path to the JCL file
     * @return JsonObject containing the evaluated variables
     * @throws RuntimeException if evaluation fails
     */
    public static JsonObject evalFileToJson(String path) {
        String json = evalFile(path);
        return gson.fromJson(json, JsonObject.class);
    }

    /**
     * Lint JCL source code and return issues as a List.
     *
     * @param source The JCL source code to lint
     * @return List of LintIssue objects
     * @throws RuntimeException if linting fails
     */
    public static List<LintIssue> lintToList(String source) {
        String json = lint(source);
        Type type = new TypeToken<List<LintIssue>>(){}.getType();
        return gson.fromJson(json, type);
    }

    /**
     * Lint JCL source code and return issues as a JsonArray.
     *
     * @param source The JCL source code to lint
     * @return JsonArray containing lint issues
     * @throws RuntimeException if linting fails
     */
    public static JsonArray lintToJson(String source) {
        String json = lint(source);
        return gson.fromJson(json, JsonArray.class);
    }

    /**
     * Represents a linting issue found in JCL code.
     */
    public static class LintIssue {
        private String rule;
        private String message;
        private String severity;
        private String suggestion;

        public String getRule() {
            return rule;
        }

        public void setRule(String rule) {
            this.rule = rule;
        }

        public String getMessage() {
            return message;
        }

        public void setMessage(String message) {
            this.message = message;
        }

        public String getSeverity() {
            return severity;
        }

        public void setSeverity(String severity) {
            this.severity = severity;
        }

        public String getSuggestion() {
            return suggestion;
        }

        public void setSuggestion(String suggestion) {
            this.suggestion = suggestion;
        }

        @Override
        public String toString() {
            return String.format("[%s] %s: %s%s",
                severity,
                rule,
                message,
                suggestion != null ? " (Suggestion: " + suggestion + ")" : "");
        }
    }
}
