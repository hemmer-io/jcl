" Vim syntax file
" Language: JCL (Jack-of-All Configuration Language)
" Maintainer: JCL Project
" Latest Revision: 2025

if exists("b:current_syntax")
  finish
endif

" Keywords
syn keyword jclKeyword fn if then else when for in import return break continue mut
syn keyword jclConstant null
syn keyword jclBoolean true false

" Types
syn keyword jclType string int float bool list map any

" Operators
syn keyword jclOperator and or not
syn match jclOperator "\v\=\>"
syn match jclOperator "\v\?\?"
syn match jclOperator "\v\?\."
syn match jclOperator "\v\=\="
syn match jclOperator "\v\!\="
syn match jclOperator "\v\<\="
syn match jclOperator "\v\>\="
syn match jclOperator "\v\*\*"
syn match jclOperator "\v\+\+"
syn match jclOperator "\v[\+\-\*/%<>=!?:|]"

" Built-in Functions
syn keyword jclFunction upper lower trim trimprefix trimsuffix replace split join
syn keyword jclFunction format substr strlen base64encode base64decode
syn keyword jclFunction jsonencode jsondecode yamlencode yamldecode
syn keyword jclFunction tomlencode tomldecode urlencode urldecode
syn keyword jclFunction length len contains keys values merge lookup
syn keyword jclFunction reverse sort slice distinct flatten compact
syn keyword jclFunction min max sum avg abs ceil floor round
syn keyword jclFunction tostring str tonumber int float tobool tolist tomap
syn keyword jclFunction md5 sha1 sha256 sha512 hash
syn keyword jclFunction timestamp formatdate timeadd
syn keyword jclFunction file fileexists dirname basename abspath
syn keyword jclFunction template templatefile
syn keyword jclFunction range zipmap coalesce try
syn keyword jclFunction cartesian combinations permutations product
syn keyword jclFunction map filter reduce

" Numbers
syn match jclNumber "\v<\d+>"
syn match jclNumber "\v<\d+\.\d+>"
syn match jclNumber "\v<\d+\.?\d*e[+-]?\d+>"
syn match jclNumber "\v<0x[0-9a-fA-F]+>"

" Strings
syn region jclString start='"' end='"' contains=jclStringInterpolation,jclEscape
syn region jclString start="'" end="'" contains=jclEscape
syn region jclString start='"""' end='"""' contains=jclStringInterpolation

" String interpolation
syn region jclStringInterpolation matchgroup=jclInterpolationDelim start="\${" end="}" contained contains=jclKeyword,jclFunction,jclOperator,jclNumber,jclBoolean

" Escape sequences
syn match jclEscape "\\[\"'\\bfnrt]" contained
syn match jclEscape "\\u[0-9a-fA-F]\{4}" contained

" Comments
syn match jclComment "#.*$"

" Type annotations
syn match jclTypeAnnotation "\v:\s*(string|int|float|bool|list|map|any)\s*\="

" Function definitions
syn match jclFunctionDef "\v<fn\s+\w+" contains=jclKeyword

" Variables and identifiers
syn match jclIdentifier "\v<[a-zA-Z_][a-zA-Z0-9_]*>"

" Highlight groups
hi def link jclKeyword Keyword
hi def link jclConstant Constant
hi def link jclBoolean Boolean
hi def link jclType Type
hi def link jclOperator Operator
hi def link jclFunction Function
hi def link jclNumber Number
hi def link jclString String
hi def link jclStringInterpolation Special
hi def link jclInterpolationDelim Delimiter
hi def link jclEscape SpecialChar
hi def link jclComment Comment
hi def link jclTypeAnnotation Type
hi def link jclFunctionDef Function
hi def link jclIdentifier Identifier

let b:current_syntax = "jcl"
