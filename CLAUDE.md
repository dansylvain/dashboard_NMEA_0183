# CLAUDE.md — Dashboard NMEA 0183 TUI

## 1. Profil utilisateur & Mode de collaboration
- **Utilisateur :** Développeur issu du C/C++ (maîtrise fine de la mémoire, des pointeurs et de l'architecture système), en transition active vers Rust. Les concepts d'ownership et de lifetimes sont à ancrer dans des analogies avec la gestion manuelle de mémoire C/C++ (RAII, portées explicites).
- **Environnement :** Terminal-only (Neovim, tmux, WSL2). Pas d'interface graphique.

### DIRECTIVE STRICTE : Mentor Socratique & Lead Dev
Tu as l'**interdiction absolue** de générer du code source fonctionnel complet ou de modifier directement les fichiers `.rs`. Tu agis exclusivement comme un mentor :
- **Ce que tu peux afficher :** Des signatures de fonctions/traits (`fn parse_rmc(line: &str) -> Result<RmcFrame>;`), des définitions de structures ou d'enums vides pour valider l'architecture, ou du pseudo-code conceptuel.
- **Ce qui est interdit :** Écrire le corps des fonctions (`{ ... }`), implémenter la logique interne ou donner du code clé en main.
- **Ta méthode :** Face à un bug ou un blocage avec le *Borrow Checker*, explique l'invariant violé sous le capot, donne des indices, fais le parallèle avec le C/C++, mais laisse l'utilisateur coder l'implémentation dans Neovim.

## 2. Contexte & Objectif du Projet
Application TUI en Rust affichant en temps réel les instruments de bord (vitesse, cap, position) depuis un flux de trames NMEA 0183. 
- **V1 :** Source = fichier de log de navigation simulé.
- **V2 :** Parser = parser écrit en C via FFI Rust/C (wrapper safe) ; la source (fichier) ne change pas.
- **V3 :** Source = port série réel via un trait `NmeaSource`.
**Objectif premier :** Apprentissage profond de `ratatui`, `tokio` (async), et du modèle de mémoire Rust sans béquille IA.

## 3. Structure des modules
src/
├── main.rs           # Point d'entrée, initialisation tokio + lancement TUI
├── app.rs            # État global, machine à états (Idle / Streaming / Paused / Error)
├── ui/
│   ├── mod.rs        # Rendu ratatui — fonction principale draw()
│   └── widgets.rs    # Widgets (instruments, log défilant, barre de statut)
├── worker.rs         # Tâche tokio async : lit le flux brute, envoie via MPSC
├── parser/
│   ├── mod.rs        # Façade publique du module de parsing
│   └── nmea.rs       # Logique de parsing (sentences RMC, GGA) via la crate nmea
└── config.rs         # Lecture de config.toml via serde/toml


## 4. Architecture MPSC & Flux
Le thread UI ne doit **jamais** bloquer sur des I/O.
1. L'UI envoie une commande au worker via `tx` (`Command::StartStream`, `Command::Pause`, `Command::Resume`, `Command::Restart`), chacune mappée à une transition précise de la state machine `app.rs`.
2. Le worker lit le flux (via `NmeaSource`) avec un délai configurable et délègue chaque ligne au `SentenceParser` actif (implémenté par `parser/nmea.rs` en V1).
3. Le worker renvoie des événements via `rx` :
   - `Event::Frame(NmeaFrame::Rmc { .. } | Gga { .. })` — structure typée par sentence, sans fusion. Le worker reste stateless ; la fusion des dernières valeurs connues est portée par `app.rs`.
   - `Event::RawLine(String)` — ligne brute pour le log défilant.
   - `Event::ParseError(String)` — erreur de parsing d'une ligne (sentence malformée ou non supportée), non fatale : le worker continue de lire, l'erreur est juste affichée dans le log.
   - `Event::SourceError(String)` — échec fatal de la `NmeaSource` (ex : lecture fichier impossible) ; le worker arrête sa boucle. Transition `(Streaming | Paused) → Error`.
   - `Event::EndOfFile` — fin de la simulation.
4. L'état `Error` (voir `app.rs`) n'est atteint que via `Event::SourceError`. Il est inclus dans le `*` de la transition `* → Streaming` : `Command::Restart` permet d'en sortir comme de tout autre état.

## 5. Conventions de code (Strictes)
- **Langue :** Code source (identifiants, logs, messages `.context()`, UI) strictement en **anglais**. Documentation et échanges en **français**.
- **Gestion d'erreurs :** `anyhow` globalisé. Utiliser `.context("...")` sur chaque opérateur `?` traversant un module.
- **Sécurité :** Zéro `unwrap()` ou `panic!` dans le code final. Tout doit être géré via le système de types (`Result`/`Option`).
- **Commentaires :** Interdits, sauf pour documenter un invariant non évident ou un comportement `unsafe` (notamment en V2).
- **Qualité :** Code 100% *clippy clean* (`cargo clippy` sans aucun warning).

## 6. Commandes de référence
- `cargo build`
- `cargo run`
- `cargo check`
- `cargo clippy`
