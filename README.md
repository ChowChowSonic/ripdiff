# ripdiff

A terminal-based directory comparison tool built with Rust and Ratatui.

## Description

`ripdiff` is a fast, parallel file tree diffing tool that displays directory differences in an interactive terminal UI. It compares two directories side-by-side, allowing you to navigate file trees and view file contents directly in your terminal.

## Features

- **File or directory input** - Accepts both files and directories as arguments
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
ripdiff <old_directory|old_file> <new_directory|new_file>
```

### Keybindings

| Key | Action |
|-----|--------|
| `↑`/`↓` | Navigate file list |
| `Enter` | Open selected file/directory |
| `←`/`→` | Adjust filename display offset |
| `PageUp`/`↓` | Scroll file contents |
| `k`/`j` | Scroll file diff content |
| `Tab` | Toggle file sidebar |
| `Esc` | Exit |

## Architecture

```
src/
├── main.rs      # Entry point: parses args and runs the app
├── lib.rs       # Module declarations
├── app.rs       # CLI argument parsing (clap) and app orchestration
├── state.rs     # TuiState struct, file/navigation logic, unit tests
├── ui.rs        # Event loop and keyboard input handling
├── render.rs    # TUI rendering (file list, diff panes, layout)
├── diff.rs      # Diff computation engine (get_file_diff)
├── walker.rs    # Parallel directory walker (MultiVisitor)
└── config.rs    # Theme/color configuration
```

## Dependencies

- **ratatui** - Terminal UI framework
- **clap** - CLI argument parsing
- **anyhow** - Error handling
- **diffy** - Diff generation
- **ignore** - Fast file tree walking
- **rayon** - Parallel data processing
- **crossterm** - Cross-platform terminal manipulation
