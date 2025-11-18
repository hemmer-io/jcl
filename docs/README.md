# JCL Documentation

This directory contains the JCL documentation site built with Jekyll.

## Local Development

To run the documentation site locally:

```bash
# Install dependencies
cd docs
bundle install

# Run Jekyll server
bundle exec jekyll serve

# Open http://localhost:4000 in your browser
```

## Publishing to GitHub Pages

The documentation is automatically published to GitHub Pages via GitHub Actions whenever changes are pushed to the `main` or `master` branch.

### One-Time Setup (via GitHub Web UI)

1. Go to your repository settings on GitHub
2. Navigate to **Settings** → **Pages**
3. Under **Source**, select **GitHub Actions**
4. The site will be published automatically on the next push

### Manual Deployment

You can also trigger a manual deployment:

1. Go to **Actions** tab on GitHub
2. Select **Deploy Jekyll Documentation to GitHub Pages**
3. Click **Run workflow**

## Site Structure

- `index.md` - Homepage
- `getting-started/` - Installation and tutorials
- `reference/` - Language specification, functions, and CLI tools
- `guides/` - Comparison guides and advanced topics

## Adding New Pages

1. Create a new Markdown file in the appropriate directory
2. Add frontmatter with `layout` and `permalink`
3. Update `_config.yml` navigation if needed

Example:

```yaml
---
layout: page
title: My New Page
permalink: /guides/my-page/
---

# My New Page

Content goes here...
```
