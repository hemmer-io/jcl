# JCL Syntax Highlighting for Vim/Neovim

Syntax highlighting for JCL (Jack-of-All Configuration Language) in Vim and Neovim.

## Features

- Full syntax highlighting for JCL
- Keyword highlighting
- Built-in function highlighting
- String interpolation support
- Comment highlighting
- Type annotation highlighting
- Operator highlighting

## Installation

### Using vim-plug

Add this to your `.vimrc` or `init.vim`:

```vim
Plug 'turner-hemmer/jcl', {'rtp': 'editors/vim'}
```

Then run `:PlugInstall`

### Using Vundle

Add this to your `.vimrc`:

```vim
Plugin 'turner-hemmer/jcl', {'rtp': 'editors/vim'}
```

Then run `:PluginInstall`

### Using Pathogen

```bash
cd ~/.vim/bundle
git clone https://github.com/turner-hemmer/jcl.git
ln -s ~/.vim/bundle/jcl/editors/vim ~/.vim/bundle/jcl-vim
```

### Manual Installation

#### Vim

```bash
mkdir -p ~/.vim/{syntax,ftdetect}
cp syntax/jcl.vim ~/.vim/syntax/
cp ftdetect/jcl.vim ~/.vim/ftdetect/
```

#### Neovim

```bash
mkdir -p ~/.config/nvim/{syntax,ftdetect}
cp syntax/jcl.vim ~/.config/nvim/syntax/
cp ftdetect/jcl.vim ~/.config/nvim/ftdetect/
```

## Usage

Syntax highlighting will automatically activate for `.jcl` files.

## Syntax Elements

The syntax file highlights:

- **Keywords**: `fn`, `if`, `then`, `else`, `when`, `for`, `in`, `import`, `mut`
- **Types**: `string`, `int`, `float`, `bool`, `list`, `map`, `any`
- **Built-in Functions**: All 56+ JCL functions
- **Operators**: `=`, `==`, `!=`, `+`, `-`, `*`, `/`, `and`, `or`, `=>`, `??`, `?.`
- **Literals**: Strings, numbers, booleans, `null`
- **Comments**: Lines starting with `#`
- **String Interpolation**: `${...}` expressions in strings

## Example

```jcl
# Configuration
name: string = "MyApp"
version: int = 2

# Lambda function
double = x => x * 2

# Higher-order functions
numbers = [1, 2, 3, 4, 5]
doubled = map(double, numbers)

# String interpolation
message = "Hello, ${name}!"
```

## Customization

You can customize the colors by adding highlight commands to your `.vimrc`:

```vim
" Example: Make keywords bold and blue
hi jclKeyword ctermfg=blue cterm=bold guifg=#0000FF gui=bold

" Example: Make strings green
hi jclString ctermfg=green guifg=#00FF00
```

## About JCL

JCL (Jack-of-All Configuration Language) is a modern, safe, and flexible general-purpose configuration language.

Learn more at: https://github.com/turner-hemmer/jcl

## License

MIT OR Apache-2.0
