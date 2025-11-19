# frozen_string_literal: true

require_relative "jcl/jcl"

module JCL
  class Error < StandardError; end
  class SyntaxError < Error; end
  class RuntimeError < Error; end
  class TypeError < Error; end

  VERSION = "1.0.0"
end
