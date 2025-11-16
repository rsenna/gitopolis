> [!NOTE]
> Current objectives
> - Create brew tap for vaquera
> - "Add all" functionality: from a given folder, add all git repos recursively
> - Convert .gitmodules to .vaquera.toml

# Vaquera

TODO
An agnostic CLI tool to manage workspaces.
Here, "workspace" means any set of files with a common purpose, e.g. a code project, a collection of related projects,
a set of configuration files, documents etc.

Once a workspace is defined and know, vaquera can
- Move it between paths
- Run ad-hoc shell commands in a specific workspace, or many workspaces at once
- Register *custom-commands* to run in a workspace, or in a group of workspaces
- Quickly `cd` into a workspace from anywhere
- Associate metadata, like tags, descriptions etc
- Import/export workspaces through zip/tar/gzip/git or other ways through *custom integrations*

A common use case is managing multiple git repositories in your machine.

Note: though git is supported by default, other VCS/DVCS systems can easily be integrated through *custom-commands*.

## Installation

### Homebrew (macOS / Linux)

TODO

### Download binary release from github

TODO

### From crates.io

TODO

## Built in help

Vaquera has a fully documented command system, so use `-h` to get help for each command:

```sh
vaquera -h
vaquera clone -h
```

## Initial setup

TODO

## Usage

TODO
