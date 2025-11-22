# JCL Ruby Bindings

Ruby bindings for JCL (Jack-of-All Configuration Language) using Magnus.

## Installation

### From RubyGems (once published)

```bash
gem install jcl
```

Or add to your `Gemfile`:

```ruby
gem 'jcl'
```

Then run:

```bash
bundle install
```

### Building from Source

1. Ensure you have Rust installed
2. Build the gem:

```bash
cargo build --release --features ruby
```

3. The compiled extension will be in `target/release/`

## Usage

### Basic Example

```ruby
require 'jcl'

# Evaluate JCL code
jcl_code = <<~JCL
  version = "1.0.0"
  port = 8080
  debug = true
JCL

config = JCL.eval(jcl_code)

puts "Version: #{config['version']}"
puts "Port: #{config['port']}"
puts "Debug: #{config['debug']}"
```

### Parsing

```ruby
# Parse JCL code to check syntax
result = JCL.parse('version = "1.0.0"')
puts result  # "Parsed 1 statements"
```

### Evaluation

```ruby
# Evaluate JCL and get results as Hash
jcl_code = <<~JCL
  app = {
    name = "MyApp"
    port = 8080
    features = ["auth", "api", "admin"]
  }
JCL

config = JCL.eval(jcl_code)

# Access nested values
puts "App name: #{config['app']['name']}"
puts "Port: #{config['app']['port']}"
puts "Features: #{config['app']['features'].join(', ')}"

# Evaluate from file
file_config = JCL.eval_file('config.jcf')
```

### Formatting

```ruby
unformatted = 'version="1.0.0" port=8080'
formatted = JCL.format(unformatted)
puts formatted
# Output:
# version = "1.0.0"
# port = 8080
```

### Linting

```ruby
jcl_code = <<~JCL
  unused_var = 42
  result = 10
JCL

issues = JCL.lint(jcl_code)

issues.each do |issue|
  puts "#{issue['severity']}: #{issue['message']}"
  puts "  Rule: #{issue['rule']}"
  puts "  Suggestion: #{issue['suggestion']}" if issue['suggestion']
end
```

### Version Information

```ruby
version = JCL.version
puts "JCL version: #{version}"
```

## API Reference

### Module Methods

#### `JCL.parse(source)`
Parse JCL source code and return a status message.
- **Parameters:** `source` (String) - The JCL source code
- **Returns:** String - Status message with number of statements parsed
- **Raises:** `RuntimeError` on parse error

#### `JCL.eval(source)`
Evaluate JCL source code and return variables as a Hash.
- **Parameters:** `source` (String) - The JCL source code
- **Returns:** Hash - Evaluated variables
- **Raises:** `RuntimeError` on evaluation error

#### `JCL.eval_file(path)`
Evaluate JCL from a file and return variables as a Hash.
- **Parameters:** `path` (String) - Path to the JCL file
- **Returns:** Hash - Evaluated variables
- **Raises:** `RuntimeError` on file read or evaluation error

#### `JCL.format(source)`
Format JCL source code.
- **Parameters:** `source` (String) - The JCL source code
- **Returns:** String - Formatted JCL source code
- **Raises:** `RuntimeError` on format error

#### `JCL.lint(source)`
Lint JCL source code and return issues as an Array.
- **Parameters:** `source` (String) - The JCL source code
- **Returns:** Array<Hash> - Lint issues with keys: `rule`, `message`, `severity`, `suggestion` (optional)
- **Raises:** `RuntimeError` on lint error

#### `JCL.version`
Get the JCL version.
- **Returns:** String - Version string

## Type Conversions

JCL types are converted to Ruby types as follows:

| JCL Type | Ruby Type |
|----------|-----------|
| String | `String` |
| Int | `Integer` |
| Float | `Float` |
| Bool | `TrueClass` / `FalseClass` |
| Null | `NilClass` |
| List | `Array` |
| Map | `Hash` |
| Function | `String` ("<function>") |

## Use Cases

### Rails Configuration

```ruby
# config/initializers/jcl_config.rb
require 'jcl'

module MyApp
  class Application < Rails::Application
    jcl_config = JCL.eval_file(Rails.root.join('config', 'app.jcf'))

    config.app_name = jcl_config['app']['name']
    config.api_endpoint = jcl_config['api']['endpoint']
    config.feature_flags = jcl_config['features']
  end
end
```

### Chef Recipe

```ruby
# recipes/default.rb
require 'jcl'

# Load configuration from JCL
config = JCL.eval_file('/etc/app/config.jcf')

# Use configuration in Chef resources
template '/etc/nginx/nginx.conf' do
  source 'nginx.conf.erb'
  variables(
    port: config['server']['port'],
    worker_processes: config['server']['workers'],
    ssl_enabled: config['server']['ssl']
  )
  notifies :reload, 'service[nginx]'
end

service 'nginx' do
  action [:enable, :start]
end
```

### Puppet Module

```ruby
# lib/puppet/functions/jcl_eval.rb
Puppet::Functions.create_function(:jcl_eval) do
  dispatch :jcl_eval do
    param 'String', :path
    return_type 'Hash'
  end

  def jcl_eval(path)
    require 'jcl'
    JCL.eval_file(path)
  end
end
```

Then in your manifest:

```puppet
# manifests/config.pp
$config = jcl_eval('/etc/app/config.jcf')

file { '/etc/app/settings.json':
  ensure  => file,
  content => inline_template('<%= JSON.pretty_generate(@config) %>'),
}
```

### Rake Task

```ruby
# lib/tasks/jcl.rake
require 'jcl'

namespace :jcl do
  desc 'Validate JCL configuration files'
  task :validate do
    Dir.glob('config/**/*.jcf').each do |file|
      puts "Validating #{file}..."

      begin
        content = File.read(file)
        issues = JCL.lint(content)

        if issues.any? { |i| i['severity'] == 'error' }
          puts "  ✗ Errors found:"
          issues.each do |issue|
            puts "    [#{issue['severity']}] #{issue['message']}"
          end
          exit 1
        else
          puts "  ✓ Valid"
        end
      rescue StandardError => e
        puts "  ✗ Error: #{e.message}"
        exit 1
      end
    end
  end

  desc 'Format JCL configuration files'
  task :format do
    Dir.glob('config/**/*.jcf').each do |file|
      puts "Formatting #{file}..."

      begin
        content = File.read(file)
        formatted = JCL.format(content)
        File.write(file, formatted)
        puts "  ✓ Formatted"
      rescue StandardError => e
        puts "  ✗ Error: #{e.message}"
      end
    end
  end
end
```

### Sinatra Application

```ruby
require 'sinatra'
require 'jcl'

configure do
  config = JCL.eval_file('config/app.jcf')

  set :port, config['server']['port']
  set :environment, config['environment']
  set :api_key, config['api']['key']
end

get '/config' do
  content_type :json
  config = JCL.eval_file('config/app.jcf')
  config.to_json
end
```

### Configuration Loader Class

```ruby
# lib/config_loader.rb
require 'jcl'

class ConfigLoader
  def initialize(path)
    @path = path
    @config = nil
  end

  def load
    @config ||= JCL.eval_file(@path)
  end

  def reload
    @config = nil
    load
  end

  def get(key_path)
    keys = key_path.split('.')
    keys.reduce(load) { |hash, key| hash[key] }
  end

  def method_missing(method_name, *args)
    if args.empty? && load.key?(method_name.to_s)
      load[method_name.to_s]
    else
      super
    end
  end

  def respond_to_missing?(method_name, include_private = false)
    load.key?(method_name.to_s) || super
  end
end

# Usage
config = ConfigLoader.new('config/app.jcf')
puts config.version
puts config.get('app.name')
```

### RSpec Integration

```ruby
# spec/support/jcl_helper.rb
require 'jcl'

RSpec.configure do |config|
  config.before(:suite) do
    test_config = JCL.eval_file('spec/fixtures/test_config.jcf')
    ENV['TEST_CONFIG'] = test_config.to_json
  end
end

# spec/models/config_spec.rb
require 'rails_helper'

RSpec.describe 'Configuration' do
  let(:jcl_code) do
    <<~JCL
      app = {
        name = "TestApp"
        version = "1.0.0"
      }
    JCL
  end

  it 'parses JCL configuration' do
    config = JCL.eval(jcl_code)
    expect(config['app']['name']).to eq('TestApp')
    expect(config['app']['version']).to eq('1.0.0')
  end

  it 'validates JCL syntax' do
    invalid_jcl = 'invalid syntax here'
    expect { JCL.eval(invalid_jcl) }.to raise_error(RuntimeError)
  end

  it 'formats JCL code' do
    unformatted = 'app={name="Test"}'
    formatted = JCL.format(unformatted)
    expect(formatted).to include("app = {")
    expect(formatted).to include('name = "Test"')
  end
end
```

## Error Handling

All methods raise `RuntimeError` on errors. It's recommended to catch and handle these appropriately:

```ruby
begin
  config = JCL.eval_file('config.jcf')
rescue RuntimeError => e
  if e.message.include?('Parse error')
    puts "Invalid JCL syntax: #{e.message}"
  elsif e.message.include?('Failed to read file')
    puts "File not found: #{e.message}"
  else
    puts "Evaluation error: #{e.message}"
  end
end
```

### Using Rescue Blocks

```ruby
# Inline rescue
config = JCL.eval_file('config.jcf') rescue {}

# Method with rescue
def load_config(path)
  JCL.eval_file(path)
rescue RuntimeError => e
  Rails.logger.error("Failed to load config: #{e.message}")
  default_config
end
```

## Performance Considerations

- The native extension is loaded once per Ruby process
- Parsing and evaluation are performed in native Rust code for optimal performance
- For repeated evaluations, consider caching parsed results:

```ruby
class JCLCache
  def initialize
    @cache = {}
  end

  def eval_file(path)
    mtime = File.mtime(path)
    cache_key = "#{path}:#{mtime}"

    @cache[cache_key] ||= JCL.eval_file(path)
  end

  def clear
    @cache.clear
  end
end
```

## Thread Safety

The JCL Ruby bindings are thread-safe. Multiple threads can safely call JCL methods concurrently:

```ruby
threads = 10.times.map do |i|
  Thread.new do
    config = JCL.eval_file("config/env#{i}.jcf")
    # Process config...
  end
end

threads.each(&:join)
```

## Development

### Running Tests

```bash
# Build the extension
cargo build --release --features ruby

# Run Ruby tests
ruby test/jcl_test.rb
```

### Benchmarking

```ruby
require 'benchmark'
require 'jcl'

jcl_code = File.read('large_config.jcf')

Benchmark.bm do |x|
  x.report('parse:') { 100.times { JCL.parse(jcl_code) } }
  x.report('eval:') { 100.times { JCL.eval(jcl_code) } }
  x.report('format:') { 100.times { JCL.format(jcl_code) } }
  x.report('lint:') { 100.times { JCL.lint(jcl_code) } }
end
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT OR Apache-2.0
