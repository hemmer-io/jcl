fn main() {
    // Tell cargo to rerun the build script if the grammar file changes
    println!("cargo:rerun-if-changed=src/grammar.pest");
}
