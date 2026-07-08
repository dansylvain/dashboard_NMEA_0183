# PROJECT: Dashboard NMEA 0183 (Rust TUI)

## 1. Project Goals

This project has two purposes:

- **Learning:** deepen knowledge of `ratatui`, `tokio`, and introduce communication with nautical equipment (serial port, NMEA 0183 protocol).
- **Practical tool:** display real-time onboard instruments (speed, heading, position) from an NMEA 0183 data stream, inside a fully terminal-based dashboard.

### Strategic positioning

This project is part of a precise B2B trajectory: **securing and modernizing legacy code (C/C++) from maritime embedded systems** to meet the requirements of the European *Cyber Resilience Act*, without performance loss.

V1 lays the Rust foundations in the target domain (NMEA parsing, async, TUI). V2 switches to an NMEA parser written in C, integrated via Rust/C FFI — directly demonstrating the key differentiating skill: interfacing and securing legacy embedded code inside a safe Rust wrapper. This project is the first building block of the **Digital Trust Stack** portfolio, targeting actors in the sailing tech space (Madintec, Pixel sur Mer, IMOCA/Ultim teams).

## 2. Functional Description

The application reads an NMEA 0183 data stream line by line, parses recognized sentences, and updates a TUI dashboard in real time.

```
NMEA Source (log file or serial port)
    │
    └─> Async worker (reading + parsing)
            │
            └─> MPSC channel (clean data structures)
                    │
                    └─> UI thread (ratatui) ──> Terminal dashboard
```

**V1:** source = `.txt` / `.log` file simulating a navigation session (line-by-line reading with a configurable delay).
**V2:** same file source; the `nmea` crate is replaced by a C parser via FFI (see section 7).
**V3:** source = real serial port (via `serialport`).

## 3. V1 Features

- **Data source:**
    - Reading an NMEA 0183 log file (path passed as a CLI argument or entered in the TUI).
    - Configurable delay between each line (real-time simulation).

- **NMEA parsing:**
    - `$GPRMC` — Position (lat/lon), speed over ground (SOG), course over ground (COG), UTC date/time.
    - `$GPGGA` — Position, altitude, GPS fix quality, number of satellites.

- **TUI dashboard:**
    - **Speed over ground (SOG)** — primary value in large display (knots).
    - **Course over ground (COG)** — in degrees.
    - **Latitude / Longitude** — decimal degrees-minutes format.
    - **Scrolling log** — raw received frames with local timestamp.
    - **Status bar** — active source, parsing state (OK / error).

- **Configuration:**
    - Local `config.toml` file (not versioned) for default parameters (file path, simulation delay).

- **Keyboard navigation:**
    - `q` / `Ctrl-C`: quit.
    - `p`: pause / resume reading.
    - `r`: restart reading from the beginning of the file.

## 4. Out of scope for V1

- Real serial port connection (planned for V3 — see section 7).
- Parsing sentences other than RMC and GGA.
- Cartographic display or GPS track.
- Export / recording of received data.

## 5. Architecture and Technical Stack

- **Language:** Rust
- **Terminal Interface (TUI):** `ratatui` (rendering) + `crossterm` (terminal backend)
- **Async runtime:** `tokio` (full features)
- **NMEA parsing:** `nmea` (RMC, GGA, and other sentences)
- **Serialization / config:** `serde` + `toml`
- **Error handling:** `anyhow`

### Concurrency model: MPSC

The UI must never block on I/O. Two-actor architecture:

- **UI thread (main):** `ratatui` event loop + keyboard capture via `crossterm`. Reads the channel on each tick and updates the displayed state.
- **Worker task (background):** `tokio` task that reads the file line by line, parses each frame via `nmea`, and sends a clean `NmeaFrame` struct into the MPSC channel toward the UI.

Worker-to-UI messages (examples):
- `Event::Frame(NmeaFrame::Rmc { sog, cog, lat, lon, datetime })`
- `Event::Frame(NmeaFrame::Gga { lat, lon, altitude, satellites })`
- `Event::RawLine(String)` — raw sentence text, for the scrolling raw-frame log.
- `Event::ParseError(String)` — malformed frame, non-fatal: displayed in the log, the worker keeps reading.
- `Event::SourceError(String)` — fatal `NmeaSource` failure (e.g. unreadable file); the worker stops its loop.
- `Event::EndOfFile` — end of the simulation file.

UI-to-worker messages:
- `Command::StartStream`
- `Command::Pause`
- `Command::Resume`
- `Command::Restart`

Each command maps to exactly one transition of the `app.rs` state machine (`Idle → Streaming`, `Streaming → Paused`, `Paused → Streaming`, `* → Streaming` from the beginning). One additional, worker-driven transition exists outside the command set: `(Streaming | Paused) → Error` on `Event::SourceError`. `Error` is included in the `*` of `* → Streaming`, so `Command::Restart` recovers from it exactly like from any other state. The worker stays stateless — it never merges fields across sentences. Merging the latest known SOG/COG/lat/lon/altitude/satellites into a single displayable snapshot is `app.rs`'s responsibility, consistent with its role as global state holder.

**Runtime flavor:** `tokio::main(flavor = "current_thread")`. V1 has exactly one worker task — a task always runs on a single thread at a time regardless of pool size, so the default `multi_thread` runtime (one OS thread per CPU core) would leave most of its pool idle for no benefit here. `current_thread` keeps the thread count honest: one plain OS thread for the UI loop (never `.await`s — only non-blocking/timeout polls on the keyboard and the channel), one tokio-managed thread for the worker task.

This is a separate concern from `tokio::task::spawn_blocking`, which V2 will likely need if the wrapped `gpsd` C function is synchronous/blocking — `spawn_blocking` runs on its own dedicated blocking-thread pool regardless of the main runtime's flavor.

Revisit `current_thread` if the scope grows to genuinely concurrent worker tasks (e.g. aggregating several `NmeaSource` instances at once) — that is the point where `multi_thread` starts paying for itself.

### Source and parser abstractions

Two independent traits, not one — they vary independently across versions:

- **`NmeaSource`** — where raw lines come from. `fn next_line(&mut self) -> Result<Option<String>, SourceError>;`. Implementations: file reader (V1), serial port reader (V3).
- **`SentenceParser`** — how a raw line becomes a typed frame. `fn parse(&self, line: &str) -> Result<NmeaFrame, ParseError>;`. Implementations: `nmea` crate wrapper (V1), `gpsd` FFI wrapper (V2, the `unsafe` boundary lives here).

The worker owns one of each and composes them; e.g. V2 keeps the V1 file `NmeaSource` unchanged and only swaps the `SentenceParser`.

## 6. Configuration structure (`config.toml`)

```toml
log_file_path = "data/sample_navigation.log"
simulation_delay_ms = 500
```

## 7. Roadmap — V2 / V3

- **V2 — FFI binding to a real, CVE-documented C parser:** replace the `nmea` crate with the NMEA0183 driver from `gpsd` (pre-3.9), which contains a documented buffer-overflow/denial-of-service vulnerability triggered by a malformed `$GPGGA` sentence missing fields and a terminator ([CVE-2013-2038](https://www.cvedetails.com/cve/CVE-2013-2038/)). Integrate it via Rust/C FFI with explicit `unsafe` handling. Goal: demonstrate the ability to wrap real, CVE-documented legacy embedded code in a safe Rust layer — the central scenario of the B2B value proposition. The vulnerable pre-fix version is the wrapped target; the safe Rust boundary is expected to contain the class of bug the CVE describes.
- **V3 — Real serial port:** integrate `serialport` to read from a USB GPS receiver/plotter as a new `NmeaSource` implementation (see section 5), alongside the existing file source. The `SentenceParser` in use (`nmea` crate or `gpsd` FFI) is independent of this choice.

## 8. V1 Roadmap

- [ ] **Phase 1: TUI skeleton**
    - Cargo initialization, dependencies.
    - `ratatui` event loop with `crossterm`.
    - Dashboard layout (blocks: speed, heading, position, log).
    - `config.toml` read / write.

- [ ] **Phase 2: NMEA worker**
    - Line-by-line file reading with delay.
    - Parsing of RMC and GGA sentences via the `nmea` crate.
    - Sending `NmeaFrame` structs via MPSC channel.
    - Handling Pause / Resume / Restart commands from the UI.

- [ ] **Phase 3: Dashboard finalization**
    - Reactive widget updates on each `Event::Frame`.
    - Scrolling log of raw frames with timestamp.
    - Status bar (source, state, last parsing error).
    - Clean handling of `Event::EndOfFile` (message + graceful shutdown).
