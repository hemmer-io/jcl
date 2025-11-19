# JCL Development Scripts

This directory contains scripts to help with JCL development.

## Git Hooks

### Pre-commit Hook

The pre-commit hook automatically runs code quality checks before each commit to ensure that all committed code meets the project's standards.

#### Installation

```bash
./scripts/install-hooks.sh
```

#### What it does

The pre-commit hook runs three checks in sequence:

1. **Code Formatting** (`cargo fmt`)
   - Automatically formats code if needed
   - Ensures consistent code style across the project

2. **Linting** (`cargo clippy`)
   - Checks for common mistakes and code smells
   - Fails if any warnings are found
   - Uses project's clippy configuration

3. **Tests** (`cargo test`)
   - Runs all unit and integration tests
   - Ensures changes don't break existing functionality
   - Fails if any tests fail

#### Bypassing the Hook

If you need to commit without running the checks (not recommended), use:

```bash
git commit --no-verify
```

**Warning:** Bypassing the hook may cause CI failures. Only use this when absolutely necessary.

#### Uninstalling

To remove the pre-commit hook:

```bash
rm .git/hooks/pre-commit
```

#### Troubleshooting

**Hook fails with "cargo: command not found"**
- Ensure Rust and Cargo are installed and in your PATH
- Run `cargo --version` to verify

**Hook takes too long**
- The hook runs all tests, which may take time on large changes
- Consider committing smaller, focused changes
- Use `git commit --no-verify` sparingly for work-in-progress commits

**Hook fails but I want to commit anyway**
- Fix the failing checks first (recommended)
- Or use `--no-verify` flag (not recommended)

## Adding New Scripts

When adding new development scripts:

1. Place them in the `scripts/` directory
2. Make them executable: `chmod +x scripts/your-script.sh`
3. Add documentation to this README
4. Use clear error messages and colored output for better UX
5. Test on both macOS and Linux if possible
