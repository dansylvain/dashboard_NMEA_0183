# Marine Dashboard — NMEA 0183 TUI

A terminal-based (TUI) dashboard written in Rust for real-time visualization of marine navigation instruments — speed over ground, course over ground, position, and raw sentence logging — via the NMEA 0183 protocol.

> **Status:** Active development — V1 (file-based simulation). See [Roadmap](#roadmap).

---

## Motivation

This project sits at the intersection of two goals:

- **Domain expertise:** Work hands-on with the standard protocols of marine electronics (NMEA 0183, with NMEA 2000 / Signal K on the horizon), from raw sentence parsing to structured telemetry data.
- **Systems language transition:** Bring embedded/systems engineering experience from C/C++ into Rust — applying ownership, borrowing, and structured concurrency where manual memory management used to live.

The result is a tool that is genuinely useful on a boat, built with the rigor expected of embedded software.

---

## Architecture

The application is designed around one constraint: **the display thread never blocks on I/O.**

```
NMEA source (log file / serial port)
        │
        ▼
  Worker task (tokio)
  ├── reads raw lines
  ├── delegates to parser module (nmea crate / nom)
  └── sends typed events over MPSC channel
        │
        ▼
  UI thread (ratatui + crossterm)
  ├── reads channel on each tick
  └── updates instrument widgets
```

**Stack:**

| Concern          | Crate                        |
|------------------|------------------------------|
| TUI rendering    | `ratatui` + `crossterm`      |
| Async runtime    | `tokio` (full features)      |
| NMEA parsing     | `nmea` (built on `nom`)      |
| Config           | `serde` + `toml`             |
| Error handling   | `anyhow`                     |

**Concurrency model — MPSC:**

- The UI sends commands to the worker: `Command::StartStream`, `Command::Pause`, `Command::Resume`, `Command::Restart`
- The worker sends events back to the UI: `Event::Frame(NmeaFrame)`, `Event::RawLine(String)`, `Event::ParseError(String)` (non-fatal), `Event::SourceError(String)` (fatal, → `Error` state), `Event::EndOfFile`

---

## Features (V1)

V1 is currently in active development.

---

## Roadmap

**V2 — FFI binding to a CVE-documented C parser**
Wrap the NMEA0183 driver from `gpsd` (pre-3.9) via Rust/C FFI, replacing the `nmea` crate as the active `SentenceParser`. The driver carries a documented buffer-overflow/denial-of-service vulnerability on malformed `$GPGGA` sentences ([CVE-2013-2038](https://www.cvedetails.com/cve/CVE-2013-2038/)) — the safe Rust boundary is expected to contain it. Central demonstration of the project's B2B positioning: securing legacy embedded C code.

**V3 — Live serial port**
Add a `serialport`-backed `NmeaSource`, alongside the existing file source, to read from a real USB GPS receiver/plotter.

**Beyond V3** *(ideas, not yet scoped in PROJECT.md)*
- Additional sentences: `$GPVTG` (course/speed), `$GPGSV` (satellites in view), `$IIDPT` (depth sounder).
- GPS track rendering via the `ratatui` canvas widget.

---

## Getting Started

### Prerequisites

- Rust toolchain (`rustup` recommended) — stable channel
- A NMEA 0183 log file (see [Test Data](#test-data) below)

### Configuration

Create a `config.toml` file at the project root (not versioned):

```toml
log_file_path = "data/sample_navigation.log"
simulation_delay_ms = 500
```

### Build and run

```bash
git clone https://github.com/dansylvain/dashboard_nmea_0183.git
cd dashboard_nmea_0183

cargo check        # fast syntax/type check
cargo clippy       # linter — no warnings accepted
cargo run          # launch the dashboard
```

### Keyboard shortcuts

| Key        | Action                        |
|------------|-------------------------------|
| `q` / `Ctrl-C` | Quit                      |
| `p`        | Pause / resume stream         |
| `r`        | Restart stream from beginning |

---

## Test Data

The application reads any plain-text file containing standard NMEA 0183 sentences, one per line. Example:

```
$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A
$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47
$GPRMC,123520,A,4807.056,N,01131.012,E,022.6,084.5,230394,003.1,W*6B
```

Sample log files with real navigation data are available from public sources such as the [GPSd project test suite](https://gitlab.com/gpsd/gpsd/-/tree/master/test) or can be generated from any chartplotter with NMEA logging enabled.

---

## Project Structure

```
src/
├── main.rs           # Entry point — tokio init, TUI launch
├── app.rs            # Application state machine (Idle / Streaming / Paused / Error)
├── ui/
│   ├── mod.rs        # ratatui draw() entry point
│   └── widgets.rs    # Instrument widgets, status bar, raw log panel
├── worker.rs         # Background task — stream reading, parser delegation, MPSC dispatch
├── parser/
│   ├── mod.rs        # Public parser facade
│   └── nmea.rs       # Sentence parsing logic (RMC, GGA) via the nmea crate
└── config.rs         # config.toml loading
```

---

## Development Environment

100% terminal workflow: Neovim / tmux / WSL2.

---

## License

MIT — see [LICENSE](LICENSE).
