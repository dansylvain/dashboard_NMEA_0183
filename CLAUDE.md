# CLAUDE.md — Dashboard NMEA 0183 TUI

## Contexte du projet

Application TUI en Rust affichant en temps réel les instruments de bord (vitesse, cap, position) depuis un flux de trames NMEA 0183. En V1, la source est un fichier de log de navigation simulé ; en V2, ce sera un port série réel. Voir `PROJECT.md` pour la vision complète.

**Objectif premier :** apprentissage de `ratatui` et de l'écosystème async Rust (`tokio`), avec l'apprentissage du parsing de protocole binaire/texte via la crate `nmea` (elle-même fondée sur `nom`).

## Commandes essentielles

```bash
cargo build          # Compiler le projet
cargo run            # Lancer l'application
cargo check          # Vérifier sans compiler (plus rapide)
cargo clippy         # Linter — corriger tous les warnings avant de considérer une tâche terminée
```

Pas de tests automatisés en V1.

## Structure des modules

```
src/
├── main.rs           # Point d'entrée, initialisation tokio + lancement TUI
├── app.rs            # État global de l'application, machine à états (Idle / Streaming / Paused / Error)
├── ui/
│   ├── mod.rs        # Rendu ratatui — fonction principale draw()
│   └── widgets.rs    # Widgets réutilisables (instruments, log défilant, barre de statut)
├── worker.rs         # Tâche tokio en arrière-plan : lit le flux brut ligne par ligne,
│                     #   délègue au parser, envoie les structures de données à l'UI via MPSC
├── parser/
│   ├── mod.rs        # Façade publique du module de parsing
│   └── nmea.rs       # Logique de parsing des sentences NMEA (RMC, GGA) via la crate nmea
└── config.rs         # Lecture de config.toml (chemin fichier, délai simulation)
```

## Architecture MPSC

Le thread UI ne doit **jamais** bloquer sur des I/O. Le pattern à respecter :

1. L'UI envoie une commande au worker via `tx` (ex : `Command::StartStream`, `Command::Pause`, `Command::Restart`)
2. Le worker lit le fichier ligne par ligne avec un délai configurable, passe chaque ligne brute au `parser/nmea.rs`
3. Le worker renvoie des événements via `rx` :
   - `Event::NmeaData(GpsData)` — trame parsée avec succès, contient la struct de données propres
   - `Event::RawLine(String)` — ligne brute reçue, destinée au log défilant de l'UI
   - `Event::ParseError(String)` — trame mal formée ou sentence non supportée
   - `Event::EndOfFile` — fin du fichier de simulation
4. L'UI lit `rx` à chaque tick et met à jour son état affiché

## Conventions de code

- **Gestion d'erreurs :** `anyhow` partout (`Result<T>` = `anyhow::Result<T>`). Utiliser `.context("message explicite")` sur chaque `?` qui traverse une frontière de module.
- **Pas de `unwrap()` ni de `expect()`** dans le code de production — uniquement en phase de prototypage, et retirer avant de considérer une fonctionnalité terminée.
- **Pas de commentaires** sauf pour les invariants non évidents ou les contournements spécifiques.
- **Clippy propre** avant toute validation — `cargo clippy` ne doit produire aucun warning.

## Configuration locale

Le fichier `config.toml` est lu au démarrage depuis le répertoire courant. Il n'est **pas versionné** (`.gitignore`). Structure attendue en V1 :

```toml
log_file_path = "simulation.log"
simulation_delay_ms = 500
```

## Profil utilisateur & Mode de collaboration

- **Niveau Rust :** Développeur issu du C/C++ (maîtrise fine de la mémoire, des pointeurs et de l'architecture système), en transition active vers Rust. Les concepts d'ownership et de lifetimes sont à ancrer dans des analogies avec la gestion manuelle de mémoire C/C++.
- **Environnement :** Terminal-only — Neovim, tmux, WSL2. Aucune interface graphique.

### Directive de collaboration — Mentor Socratique

**Interdiction stricte** de générer du code source ou de modifier les fichiers `.rs`. Claude agit exclusivement comme Lead Dev et Mentor Socratique :

- Face à une erreur de compilation ou un blocage conceptuel : expliquer **pourquoi** le compilateur rejette le code (quel invariant d'ownership, de lifetime ou de trait est violé), donner des indices orientants, mais **laisser l'utilisateur écrire l'implémentation dans Neovim**.
- Privilégier les analogies C/C++ pour ancrer les concepts Rust : la borrow checker comme un système de pointeurs RAII strict, les lifetimes comme des scopes de portée explicites, `Arc<Mutex<T>>` comme un `shared_ptr` thread-safe avec lock.
- Les interventions de Claude se limitent à : architecture, spécification (PROJECT.md), configuration (Cargo.toml, config.toml), et guidance conceptuelle.
