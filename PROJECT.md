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
**V2:** source = real serial port (via `serialport`).

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

- Real serial port connection (planned for V2 — see section 6).
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
- `Event::ParseError(String)` — malformed frame, displayed in the log.
- `Event::EndOfFile` — end of the simulation file.

UI-to-worker messages:
- `Command::Pause`
- `Command::Resume`
- `Command::Restart`

## 6. Configuration structure (`config.toml`)

```toml
log_file_path = "data/sample_navigation.log"
simulation_delay_ms = 500
```

Section 7 — replaces the V2 track with:

- **V2 — FFI binding to a C parser:** replace the `nmea` crate with an NMEA parser written in C (custom stub or existing library), integrated via Rust/C FFI with explicit `unsafe` handling. Goal: demonstrate the ability to wrap legacy embedded code in a safe Rust layer — the central scenario of the B2B value proposition.
- **V3 — Real serial port:** integrate `serialport` to read from a USB GPS receiver/plotter. Make the source interchangeable via a common trait `NmeaSource` (file / C FFI / serial port).

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
