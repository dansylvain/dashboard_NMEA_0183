# CLAUDE.md — ElevenLabs TUI

## Contexte du projet

Application TUI en Rust permettant de générer des fichiers audio à partir d'une chaîne de caractères via l'API ElevenLabs. Une traduction préalable (via API tierce) est effectuée si la langue source diffère de la langue cible. Voir `PROJECT.md` pour la vision complète.

**Objectif premier :** apprentissage de `ratatui` et de l'écosystème async Rust (`tokio`, `reqwest`).

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
├── app.rs            # État global de l'application, machine à états (Idle / Processing / Done / Error)
├── ui/
│   ├── mod.rs        # Rendu ratatui — fonction principale draw()
│   └── widgets.rs    # Widgets réutilisables (champs de saisie, barre de statut, etc.)
├── worker.rs         # Tâche tokio en arrière-plan : appels API, messages de progression via MPSC
├── api/
│   ├── translation.rs  # Client pour l'API de traduction
│   └── elevenlabs.rs   # Client pour l'API ElevenLabs
└── config.rs         # Lecture/écriture de config.toml (clés API, voix par défaut)
```

## Architecture MPSC

Le thread UI ne doit **jamais** bloquer sur des appels réseau. Le pattern à respecter :

1. L'UI envoie un message au worker via `tx` (ex: `Command::Generate { text, from, to, voice }`)
2. Le worker exécute les appels `reqwest` en tâche de fond
3. Le worker renvoie des messages de progression via `rx` (ex: `Event::TranslationDone`, `Event::AudioDone`, `Event::Error`)
4. L'UI lit `rx` à chaque tick et met à jour son état

## Conventions de code

- **Gestion d'erreurs :** `anyhow` partout (`Result<T>` = `anyhow::Result<T>`). Utiliser `.context("message explicite")` sur chaque `?` qui traverse une frontière de module.
- **Pas de `unwrap()` ni de `expect()`** dans le code de production — uniquement en phase de prototypage, et retirer avant de considérer une fonctionnalité terminée.
- **Pas de commentaires** sauf pour les invariants non évidents ou les contournements spécifiques.
- **Clippy propre** avant toute validation — `cargo clippy` ne doit produire aucun warning.

## Configuration locale

Le fichier `config.toml` est lu au démarrage depuis le répertoire courant. Il n'est **pas versionné** (`.gitignore`). Structure attendue :

```toml
elevenlabs_api_key = ""
translation_api_key = ""
default_voice_id = ""
```

Ne jamais écrire de clés API en dur dans le code.

## Profil utilisateur

- Niveau Rust : **intermédiaire** (ownership, borrowing, traits acquis — apprentissage de ratatui/tokio en cours)
- Environnement : **WSL2** sous Windows
- Préférer des explications orientées "pourquoi" plutôt que "quoi" pour les concepts ratatui/tokio
