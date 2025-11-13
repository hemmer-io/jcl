fn main() {
    // Tell cargo to rerun the build script if the grammar file changes
    // Note: The actual grammar file used is ./grammar.pest (root), not src/grammar.pest
    println!("cargo:rerun-if-changed=grammar.pest");
}
