# PROJET : Dashboard NMEA 0183 (TUI Rust)

## 1. Objectif du projet

Ce projet a deux finalités :

- **Apprentissage :** approfondir `ratatui`, `tokio`, et introduire la communication avec des équipements nautiques (port série, protocole NMEA 0183).
- **Outil pratique :** afficher en temps réel les instruments de bord (vitesse, cap, position) depuis un flux NMEA 0183, dans un dashboard entièrement terminal.

Objectif stratégique à long terme : se spécialiser dans le développement de systèmes embarqués nautiques.

## 2. Description Fonctionnelle

L'application lit un flux de trames NMEA 0183 ligne par ligne, parse les sentences reconnues, et met à jour un dashboard TUI en temps réel.

```
Source NMEA (fichier log ou port série)
    │
    └─> Worker async (lecture + parsing)
            │
            └─> Channel MPSC (structures de données propres)
                    │
                    └─> Thread UI (ratatui) ──> Dashboard terminal
```

**V1 :** source = fichier `.txt` / `.log` simulant une navigation (lecture ligne par ligne avec délai configurable).
**V2 :** source = port série réel (via `serialport`).

## 3. Fonctionnalités V1

- **Source de données :**
    - Lecture d'un fichier de log NMEA 0183 (chemin passé en argument CLI ou saisi dans la TUI).
    - Délai entre chaque ligne configurable (simulation du temps réel).

- **Parsing NMEA :**
    - `$GPRMC` — Position (lat/lon), vitesse fond (SOG), cap fond (COG), date/heure UTC.
    - `$GPGGA` — Position, altitude, qualité du fix GPS, nombre de satellites.

- **Dashboard TUI :**
    - **Vitesse sur le fond (SOG)** — valeur principale en grand affichage (nœuds).
    - **Cap fond (COG)** — en degrés.
    - **Latitude / Longitude** — format degrés-minutes décimales.
    - **Log défilant** — trames brutes reçues, avec horodatage local.
    - **Barre de statut** — source active, état du parsing (OK / erreur).

- **Configuration :**
    - Fichier `config.toml` local (non versionné) pour les paramètres par défaut (chemin fichier, délai de simulation).

- **Navigation clavier :**
    - `q` / `Ctrl-C` : quitter.
    - `p` : pause / reprise de la lecture.
    - `r` : relancer la lecture depuis le début du fichier.

## 4. Hors scope V1

- Connexion port série réelle (prévu V2 — voir section 6).
- Parsing de sentences autres que RMC et GGA.
- Affichage cartographique ou trace GPS.
- Export / enregistrement des données reçues.

## 5. Architecture et Stack Technique

- **Langage :** Rust
- **Interface Terminal (TUI) :** `ratatui` (rendu) + `crossterm` (backend terminal)
- **Asynchronisme :** `tokio` (runtime async, features full)
- **Parsing NMEA :** `nmea` (sentences RMC, GGA et autres)
- **Sérialisation / config :** `serde` + `toml`
- **Gestion d'erreurs :** `anyhow`

### Modèle de concurrence : MPSC

L'UI ne doit jamais bloquer sur des I/O. Architecture à deux acteurs :

- **Thread UI (principal) :** boucle d'événements `ratatui` + capture clavier via `crossterm`. Lit le channel à chaque tick et met à jour l'état affiché.
- **Worker task (arrière-plan) :** tâche `tokio` qui lit le fichier ligne par ligne, parse chaque trame via `nmea`, et envoie une structure `NmeaFrame` propre dans le channel MPSC vers l'UI.

Messages du worker vers l'UI (exemples) :
- `Event::Frame(NmeaFrame::Rmc { sog, cog, lat, lon, datetime })`
- `Event::Frame(NmeaFrame::Gga { lat, lon, altitude, satellites })`
- `Event::ParseError(String)` — trame mal formée, affichée dans le log.
- `Event::EndOfFile` — fin du fichier de simulation.

Message de l'UI vers le worker :
- `Command::Pause`
- `Command::Resume`
- `Command::Restart`

## 6. Structure de la Configuration (`config.toml`)

```toml
log_file_path = "data/sample_navigation.log"
simulation_delay_ms = 500
```

## 7. Pistes pour les versions futures

- **V2 — Port série réel :** intégrer la crate `serialport` pour lire depuis un récepteur GPS / traceur de carte branché en USB/série. Rendre la source interchangeable (fichier vs. port série) via un trait commun `NmeaSource`.
- **V3 — Sentences supplémentaires :** `$GPVTG` (cap/vitesse), `$GPGSV` (satellites en vue), `$IIDPT` (sonde).
- **V4 — Trace GPS :** affichage d'une trace de route dans un widget canvas `ratatui`.

## 8. Roadmap V1

- [ ] **Phase 1 : Squelette TUI**
    - Initialisation Cargo, dépendances.
    - Boucle d'événements `ratatui` avec `crossterm`.
    - Layout du dashboard (blocs : vitesse, cap, position, log).
    - Lecture / écriture `config.toml`.

- [ ] **Phase 2 : Worker NMEA**
    - Lecture du fichier log ligne par ligne avec délai.
    - Parsing des sentences RMC et GGA via la crate `nmea`.
    - Envoi des structures `NmeaFrame` via channel MPSC.
    - Gestion des commandes Pause / Resume / Restart depuis l'UI.

- [ ] **Phase 3 : Finalisation Dashboard**
    - Mise à jour réactive des widgets à chaque `Event::Frame`.
    - Log défilant des trames brutes avec horodatage.
    - Barre de statut (source, état, dernière erreur de parsing).
    - Gestion propre de `Event::EndOfFile` (message + arrêt propre).
