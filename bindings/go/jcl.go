// Package jcl provides Go bindings for the Jack-of-All Configuration Language (JCL).
//
// JCL is a general-purpose configuration language with powerful built-in functions
// that prioritizes safety, ease of use, and flexibility.
package jcl

/*
#cgo LDFLAGS: -L./target/release -ljcl
#include <stdlib.h>
#include "./src/jcl.h"
*/
import "C"
import (
	"encoding/json"
	"errors"
	"unsafe"
)

// Parse parses JCL source code and returns a summary.
func Parse(source string) (string, error) {
	cSource := C.CString(source)
	defer C.free(unsafe.Pointer(cSource))

	cResult := C.jcl_parse(cSource)
	defer C.jcl_free_string(cResult)

	if cResult == nil {
		return "", errors.New("parse failed")
	}

	return C.GoString(cResult), nil
}

// Eval evaluates JCL source code and returns the result as a map.
func Eval(source string) (map[string]interface{}, error) {
	cSource := C.CString(source)
	defer C.free(unsafe.Pointer(cSource))

	cResult := C.jcl_eval(cSource)
	defer C.jcl_free_string(cResult)

	if cResult == nil {
		return nil, errors.New("evaluation failed")
	}

	jsonStr := C.GoString(cResult)

	var result map[string]interface{}
	err := json.Unmarshal([]byte(jsonStr), &result)
	if err != nil {
		return nil, err
	}

	return result, nil
}

// EvalFile loads and evaluates a JCL file.
func EvalFile(path string) (map[string]interface{}, error) {
	cPath := C.CString(path)
	defer C.free(unsafe.Pointer(cPath))

	cResult := C.jcl_eval_file(cPath)
	defer C.jcl_free_string(cResult)

	if cResult == nil {
		return nil, errors.New("evaluation failed")
	}

	jsonStr := C.GoString(cResult)

	var result map[string]interface{}
	err := json.Unmarshal([]byte(jsonStr), &result)
	if err != nil {
		return nil, err
	}

	return result, nil
}

// Format formats JCL source code.
func Format(source string) (string, error) {
	cSource := C.CString(source)
	defer C.free(unsafe.Pointer(cSource))

	cResult := C.jcl_format(cSource)
	defer C.jcl_free_string(cResult)

	if cResult == nil {
		return "", errors.New("format failed")
	}

	return C.GoString(cResult), nil
}

// LintIssue represents a linting issue found in JCL code.
type LintIssue struct {
	Rule       string `json:"rule"`
	Message    string `json:"message"`
	Severity   string `json:"severity"`
	Suggestion string `json:"suggestion,omitempty"`
}

// Lint lints JCL source code and returns any issues found.
func Lint(source string) ([]LintIssue, error) {
	cSource := C.CString(source)
	defer C.free(unsafe.Pointer(cSource))

	cResult := C.jcl_lint(cSource)
	defer C.jcl_free_string(cResult)

	if cResult == nil {
		return nil, errors.New("lint failed")
	}

	jsonStr := C.GoString(cResult)

	var issues []LintIssue
	err := json.Unmarshal([]byte(jsonStr), &issues)
	if err != nil {
		return nil, err
	}

	return issues, nil
}

// Version returns the JCL version.
func Version() string {
	cVersion := C.jcl_version()
	defer C.jcl_free_string(cVersion)
	return C.GoString(cVersion)
}
