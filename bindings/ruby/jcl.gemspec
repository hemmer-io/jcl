Gem::Specification.new do |spec|
  spec.name          = "jcl-lang"
  spec.version       = "1.0.0"
  spec.authors       = ["Hemmer IO"]
  spec.email         = ["info@hemmer.io"]

  spec.summary       = "Ruby bindings for JCL (Jack-of-All Configuration Language)"
  spec.description   = "JCL is a general-purpose configuration language with powerful built-in functions, static type inference, and multi-language bindings."
  spec.homepage      = "https://hemmer-io.github.io/jcl/"
  spec.license       = "MIT OR Apache-2.0"
  spec.required_ruby_version = ">= 2.7.0"

  spec.metadata = {
    "homepage_uri"      => spec.homepage,
    "source_code_uri"   => "https://github.com/hemmer-io/jcl",
    "bug_tracker_uri"   => "https://github.com/hemmer-io/jcl/issues",
    "documentation_uri" => "https://hemmer-io.github.io/jcl/",
    "changelog_uri"     => "https://github.com/hemmer-io/jcl/blob/main/CHANGELOG.md"
  }

  spec.files = Dir[
    "lib/**/*.rb",
    "ext/**/*.{rs,toml}",
    "Cargo.toml",
    "README.md",
    "LICENSE-MIT",
    "LICENSE-APACHE"
  ]

  spec.require_paths = ["lib"]
  spec.extensions    = ["ext/jcl/extconf.rb"]

  # Development dependencies
  spec.add_development_dependency "rake", "~> 13.0"
  spec.add_development_dependency "rake-compiler", "~> 1.2"
  spec.add_development_dependency "rspec", "~> 3.12"

  # Runtime dependency for native extension compilation
  spec.add_dependency "rb_sys", "~> 0.9"
end
