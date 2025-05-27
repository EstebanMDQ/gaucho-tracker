# Gaucho Tracker

**Gaucho Tracker** is an open-source, modular, DAWless music tracker built in Rust, designed for single-board computers like the Raspberry Pi. It offers a minimal Linux environment that boots directly into a terminal-based tracker interface with a built-in sequencer and sampler.

## Objectives
- Portable, standalone music production station
- Modular codebase supporting sequencer, sampler, MIDI I/O, plugin hosting
- Runs on a minimal Linux system with a 3.5" TFT screen
- Written in Rust with a TUI interface and low-latency audio engine
- USB and MIDI hardware support

## Repository Structure

```
gaucho-tracker/
├── Cargo.toml                # Workspace definition
├── crates/
│   ├── core/                 # Timing, transport, scheduling
│   ├── sampler/              # WAV loading and sample playback
│   ├── sequencer/            # Step sequencer and pattern logic
│   ├── audio/                # Audio backend abstraction
│   └── tui/                  # Terminal UI
└── tests/                    # Integration tests
```

## Getting Started

### Prerequisites
- Rust (latest stable, install via [rustup](https://rustup.rs))

### Install dependencies
```sh
cargo build
```

### Run the app (TUI frontend)
```sh
cargo run -p tui
```

### Run all tests
```sh
cargo test --workspace
```

### Linting and Formatting
This project uses [rustfmt](https://github.com/rust-lang/rustfmt) and [clippy](https://github.com/rust-lang/rust-clippy) to ensure code consistency.

To format your code:
```sh
cargo fmt
```

To run Clippy for lint checks:
```sh
cargo clippy --all-targets --all-features -- -D warnings
```

Install both tools with:
```sh
rustup component add rustfmt clippy
```

## Future Features
- MIDI input/output/thru support
- Live audio recording and resampling
- Synth module
- Plugin hosting (LV2 or custom format)
- Save/load sessions and patterns

## License
MIT

---
For development guidance, issue tracking, and contributions, please open a PR or start a discussion!