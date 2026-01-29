# Fire Channel

Tummo (inner fire) meditation trainer for the [COSMIC](https://system76.com/cosmic) desktop.

Guides you through the traditional Tibetan practice of nine-round breathing followed by vase breathing. A body silhouette visualization shows the three energy channels (roma, kyangma, uma) and tracks breath flow through each nostril across the nine rounds.

## Features

- **Nine-round breathing** — Three rounds per nostril pattern (left, right, both), nine rounds total
- **Vase breathing** — Timed breath retention phase after the nine rounds complete
- **Body visualization** — Silhouette with animated energy channels, inner fire, and nostril indicators
- **Configurable timing** — Adjustable inhale (default 6s), exhale (default 8s), and vase hold (default 15s) durations
- **Audio** — Brown noise, binaural beats, and phase transition tones (independently togglable)
- **Session timer** — 5/10/15/20/30 minutes or infinite
- **Statistics** — Tracks total practice time, sessions completed, and daily practice
- **Persistent settings** — Configuration and stats saved to `~/.config/firechannel/firechannel/settings.json`

## Install

### Nix flake

```nix
# flake input
firechannel.url = "github:daytimeforninja/firechannel";

# add to packages
inputs.firechannel.packages.${system}.default
```

### From source

```sh
nix develop  # or install deps manually: wayland, libxkbcommon, libGL, fontconfig, alsa-lib, pkg-config
cargo run
```

## License

[Hippocratic License 3.0](LICENSE.md)
