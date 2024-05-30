# Gitdl

The Gitdl is a command-line tool that allows you to selectively download files or directories from a GitHub repository, without having to clone the entire repository.

## Features

- Download specific files or directories from a GitHub repository
- Supports file patterns to filter downloads
- Maintains the original directory structure
- Configurable proxy settings
- Specify a reference (branch, tag, or commit) to download from

## Usage

```
gitdl.exe [OPTIONS] <url> [pattern]
```

**Arguments:**
- `<url>`: The URL of the GitHub repository
- `[pattern]`: The file pattern to match (default is `*` to download all files)

**Options:**
- `-p <proxy>`: Set a proxy server (default is empty)
- `-r <ref>`: Specify a reference (branch, tag, or commit) to download from (default is `HEAD`)
- `-h, --help`: Print the help message
- `-V, --version`: Print the version information

## Example

Download all files from the `docs` directory of a GitHub repository:

```
gitdl.exe https://github.com/example/my-repo docs/*
```

Download a specific file from a GitHub repository:

```
gitdl.exe https://github.com/example/my-repo README.md
```

Download files from a specific branch or tag:

```
gitdl.exe -r develop https://github.com/example/my-repo *.js
```

This README.md file provides a clear and concise overview of your GitHub File Downloader tool, covering the main features, usage instructions, and examples. Feel free to further customize and expand it as needed.