# 🏀 rust-tui-ball-bounce

A terminal-based bouncing ball simulation built with Rust, featuring real-time physics, ball-to-ball elastic collisions, and live telemetry graphs — all rendered in your terminal using [Ratatui](https://github.com/ratatui/ratatui) and [Crossterm](https://github.com/crossterm-rs/crossterm).

![Rust](https://img.shields.io/badge/Rust-2021_Edition-orange)

## Features

- **Ball Arena** — Watch balls bounce around a bordered arena rendered directly in your terminal
- **Elastic Collisions** — Balls collide with each other using physically accurate elastic collision resolution
- **Multiple Balls** — Add or remove balls on the fly, each with a unique color and symbol (●, ◉, ○, ◎, ◆, ■, ▲, ★)
- **Live Telemetry Graphs** — Four real-time charts display X position, Y position, X velocity, and Y velocity over time using Braille-dot rendering
- **Speed Control** — Adjust the simulation speed from 0.25× to 5.0×
- **Pause/Resume** — Freeze and unfreeze the simulation at any time
- **~60 FPS** — Smooth animation at approximately 60 frames per second

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (2021 edition or later)

### Build & Run

```bash
# Build and run
cargo run 
```

## Controls

| Key              | Action           |
|------------------|------------------|
| `Space` / `P`   | Pause / Resume   |
| `+` / `=` / `A` | Add a ball       |
| `-` / `_` / `R` | Remove a ball    |
| `↑`              | Speed up         |
| `↓`              | Speed down       |
| `Q` / `Esc`     | Quit             |

## Layout

The TUI is divided into three rows:

| Section | Contents |
|---------|----------|
| **Top** | Ball arena (left) and status/controls panel (right) |
| **Middle** | X Position graph (left) and Y Position graph (right) |
| **Bottom** | X Velocity graph (left) and Y Velocity graph (right) |

Each graph tracks up to 300 ticks of history per ball, with all balls plotted simultaneously in their respective colors.

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| [ratatui](https://crates.io/crates/ratatui) | 0.29 | Terminal UI framework (widgets, layout, charts) |
| [crossterm](https://crates.io/crates/crossterm) | 0.28 | Cross-platform terminal manipulation (input, raw mode) |

## License

This project is open source. Feel free to use, modify, and distribute it as you see fit.
