# kep

Cache any command output.

## Usage

```bash
kep curl google.com        # cached for 1h (default)
kep 7d curl google.com     # cached for 7 days
kep 30m echo hello         # cached for 30 minutes
```

Duration suffixes: `s` (seconds), `m` (minutes), `h` (hours), `d` (days)

## Install

```bash
cargo install kep
```
