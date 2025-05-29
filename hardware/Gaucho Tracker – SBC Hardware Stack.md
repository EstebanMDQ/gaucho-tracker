# Gaucho Tracker – SBC Hardware Stack

## Overview

This document defines the hardware and software stack for the main computing unit of Gaucho Tracker. The goal is to create a platform that is reliable, easy to replicate, and friendly for community contributions.

The Raspberry Pi was chosen as the primary SBC due to its wide availability, excellent support for USB peripherals, audio, and GPIO, and its extensive documentation.

---

## Core Platform

### **Primary SBC**
- **Raspberry Pi 4** (recommended for best performance)
- **Raspberry Pi Zero 2 W** (for minimal/portable builds)

> The project is designed to run on either platform. Pi 4 is ideal for development or real-time sampling; Pi Zero 2 W is great for compact builds.

---

## Audio Interface

### **Audio Input / Output**
- **Initial Plan:** Use USB audio interfaces for both input and output

> Most **USB audio sound cards** should work out-of-the-box with Raspberry Pi OS via ALSA. These include:
- UGREEN USB sound adapters
- AudioInjector USB or HAT cards
- Behringer UMC22, UCA202, etc.
- Any class-compliant USB DAC/ADC

No kernel modifications or drivers are typically needed. This makes it easier for the community to adapt different soundcards.

---

## Display

### **Target Display**
- **3.5" TFT Display (SPI, Waveshare Pi-compatible)**
  - Initial dev model: **Waveshare 3.5” TFT for Pi (A-type)**

> If display compatibility becomes a blocker, alternative options (HDMI or DSI displays) may be considered. The aim is to maintain a functional and minimal TUI interface that fits within 320×480 resolution.

---

## USB Peripherals

- **Custom keyboard module** (Raspberry Pi Pico HID)
- **Rotary encoders** handled by the Pico and exposed as key or scroll events
- Optional: USB MIDI input devices or hubs

---

## Software Stack

- **OS**: Raspberry Pi OS Lite (headless CLI, no GUI)
- **Audio**: ALSA (with optional `arecord`, `sox`, or custom Rust bindings)
- **Programming Language**: Rust
- **UI**: Text-based interface via `ratatui` or similar

---

## Goals for Community Use

- Work across Raspberry Pi models
- Plug-and-play USB audio
- Minimal dependencies
- Modular design (Pico handles input, SBC handles core)
- Open-source, remixable, minimal barriers to entry

---

## Next Steps

- [ ] Confirm USB audio sampling via CLI tools (`arecord`, `alsa-utils`)
- [ ] Prototype full stack: keyboard + soundcard + screen
- [ ] Benchmark performance on Pi 4 vs Zero 2 W
- [ ] Document tested audio interfaces