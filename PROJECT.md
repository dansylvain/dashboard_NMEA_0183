# PROJET : ElevenLabs TUI (Nom à définir)

## 1. Objectif du projet

Ce projet a deux finalités :

- **Apprentissage :** explorer la création d'interfaces terminal (TUI) en Rust via la bibliothèque `ratatui`.
- **Outil pratique :** générer des fichiers audio à partir d'une chaîne de caractères, en s'appuyant sur l'API ElevenLabs pour la synthèse vocale.

## 2. Description Fonctionnelle

L'utilisateur saisit une chaîne de caractères (ou désigne un fichier qui en contient une). Il précise la **langue source** (langue dans laquelle le texte est écrit) et la **langue cible** (langue que doit utiliser ElevenLabs pour le prononcer).

Le pipeline se déroule comme suit :

```
Texte source
    │
    ├─ [si langue source ≠ langue cible] ──> API de traduction ──> Texte traduit
    │
    └─ [dans tous les cas] ──> API ElevenLabs (voix choisie, langue cible) ──> Fichier audio
```

L'utilisateur choisit ensuite le nom et l'emplacement du fichier audio généré via une fenêtre de sauvegarde.

## 3. Fonctionnalités V1

- **Entrée du texte** (via la TUI) :
    - Saisie/collage direct d'une chaîne de caractères dans l'interface.
    - Désignation d'un fichier local contenant la chaîne.
- **Paramètres de traitement** (via la TUI) :
    - Sélection de la langue source.
    - Sélection de la langue cible.
    - Sélection de la voix ElevenLabs à utiliser.
- **Pipeline conditionnel :**
    - Si langue source = langue cible → envoi direct à ElevenLabs.
    - Si langue source ≠ langue cible → traduction préalable, puis envoi à ElevenLabs.
- **Sauvegarde du fichier audio :**
    - Widget de saisie de texte dans la TUI pour entrer le chemin et le nom du fichier de sortie (ex: `/mnt/c/Users/.../output.mp3`).
    - Approche "Pure TUI", style Vim — pas de dépendance à un dialogue natif OS, compatible WSL sans configuration supplémentaire.
- **Configuration :**
    - Clés API (ElevenLabs, service de traduction) stockées dans un fichier de config local.
    - Service de traduction : à déterminer (DeepL, OpenAI, etc.).

## 4. Hors scope V1

- Lecture de fichiers multi-blocs ou longs documents.
- Gestion de plus de 5-6 langues (couverture pratique restreinte).

## 5. Pistes pour les versions futures

- **V2 — Mode CLI pur :** ajouter `clap` pour permettre d'utiliser l'outil directement en ligne de commande, sans passer par la TUI (ex: `app "Bonjour" --from fr --to en --voice ma_voix`). Utile pour scripter ou automatiser des générations en masse.

## 5. Architecture et Stack Technique

- **Langage :** Rust
- **Interface Terminal (TUI) :** `ratatui` (rendu) + `crossterm` (backend terminal)
- **Asynchronisme & Réseau :** `tokio` (runtime) + `reqwest` (requêtes HTTP)
- **Sérialisation :** `serde` + `serde_json` / `toml` (config locale et parsing des réponses API)

### Modèle de concurrence : MPSC

Pour éviter tout gel de l'interface pendant les appels réseau, l'application adopte une architecture à deux acteurs communicant via des canaux MPSC (`tokio::sync::mpsc`) :

- **Thread UI (principal) :** gère le rendu `ratatui` et la capture des événements clavier. Quand l'utilisateur lance le traitement, il envoie un message dans le canal et passe son état à `Processing`.
- **Worker task (arrière-plan) :** tâche `tokio` qui exécute les appels `reqwest` (traduction conditionnelle → ElevenLabs) et renvoie des messages de progression au thread UI.

Exemples de messages retournés par le worker : `TranslationDone`, `AudioDone`, `Error`.

## 6. Structure de la Configuration (`config.toml`)

```toml
elevenlabs_api_key = ""
translation_api_key = ""
default_voice_id = ""
```

## 7. Roadmap V1

- [ ] **Phase 1 : Squelette TUI**
    - Initialisation du projet Cargo avec les dépendances.
    - Mise en place de la boucle d'événements `ratatui`.
    - Écran principal avec les champs de saisie (texte, langues, voix).
    - Lecture/écriture du fichier de configuration.
- [ ] **Phase 2 : Intégration API**
    - Requêtes asynchrones vers l'API de traduction (avec branchement conditionnel).
    - Requêtes asynchrones vers l'API ElevenLabs.
    - Sauvegarde du flux audio reçu sur le disque.
- [ ] **Phase 3 : Finalisation TUI**
    - Gestion des états (chargement, succès, erreur) dans l'interface.
    - Intégration de la fenêtre de sauvegarde native (investigation WSL/PowerShell).
    - Affichage du résultat et confirmation à l'utilisateur.
