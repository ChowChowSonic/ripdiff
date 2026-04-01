# ripdiff

A terminal-based directory comparison tool built with Rust and Ratatui.

## Description

`ripdiff` is a fast, parallel file tree diffing tool that displays directory differences in an interactive terminal UI. It compares two directories side-by-side, allowing you to navigate file trees and view file contents directly in your terminal.

## Features

- **Parallel file discovery** - Uses rayon and ignore for fast directory traversal
- **Side-by-side comparison** - View file contents from both directories simultaneously
- **Interactive TUI** - Navigate file trees with keyboard controls
- **Real-time file viewing** - Open and scroll through file contents
- **Expandable directory tree** - Navigate nested folder structures on-the-fly

## Installation

```bash
cargo build --release
```

## Usage

```bash
ripdiff <old_directory> <new_directory>
```

### Keybindings

| Key | Action |
|-----|--------|
| `↑`/`↓` | Navigate file list |
| `Enter` | Open selected file/directory |
| `←`/`→` | Adjust filename display offset |
| `PageUp`/`PageDown` | Scroll file contents |
| `Esc` | Exit |

## Architecture

```
src/
├── main.rs          # Entry point and parallel directory loading
├── multivisitor.rs  # Parallel visitor pattern for file tree walking
└── tui.rs           # Terminal UI rendering and input handling
```

## Dependencies

- **ratatui** - Terminal UI framework
- **diffy** - Diff generation
- **ignore** - Fast file tree walking
- **rayon** - Parallel data processing
- **crossterm** - Cross-platform terminal manipulation
