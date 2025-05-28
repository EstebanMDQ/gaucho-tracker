
# Gaucho Tracker Project Format

This document describes the file and folder structure for a Gaucho Tracker song project.

## Project Folder Structure

Each project is stored as a directory:

```
projects/
└── my-song/
    ├── gaucho.toml              # Project metadata
    ├── patterns/                # Step sequence data
    │   ├── 000.json             # Pattern steps
    │   └── 000.meta.json        # Tracker-specific FX
    ├── samples/                 # Sample files
    │   ├── kick.wav
    │   └── snare.wav
    ├── tracks.json              # Track/sample mapping
    └── notes.md                 # User notes or lyrics
```

## File Descriptions

### `gaucho.toml`

```toml
name = "My Song"
version = "1.0"
bpm = 120
swing = 0.0
author = "esteban"
created = "2025-05-27T14:00:00Z"
```

### `tracks.json`

```json
[
  { "name": "Kick", "sample": "samples/kick.wav", "volume": 1.0 },
  { "name": "Snare", "sample": "samples/snare.wav", "volume": 1.0 }
]
```

### `patterns/000.json`

```json
{
  "pattern_id": 0,
  "steps": [
    [true, false, false, false, true, false, false, false],
    [false, false, true, false, false, false, true, false]
  ]
}
```

### `patterns/000.meta.json`

```json
{
  "track_map": [
    { "channel": 1, "sample": "kick.wav" },
    { "channel": 2, "sample": "snare.wav" }
  ],
  "fx": {
    "1:3": { "retrigger": 2 },
    "2:7": { "reverse": true }
  }
}
```

### `notes.md`

```
# My Song
- Structure: intro / verse / chorus
- Try delay on snare in chorus
```
