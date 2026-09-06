# easysearch

[![Crates.io](https://img.shields.io/crates/v/easysearch.svg)](https://crates.io/crates/easysearch)
[![Downloads](https://img.shields.io/crates/d/easysearch.svg)](https://crates.io/crates/easysearch)
[![docs.rs](https://docs.rs/easysearch/badge.svg)](https://docs.rs/easysearch)
[![License](https://img.shields.io/crates/l/easysearch.svg)](LICENSE)

Bibliothèque légère de recherche floue (fuzzy search), conçue pour le projet **Hodoe**. Trouve des résultats pertinents même en présence de fautes de frappe, en combinant la distance de Levenshtein et un score de similarité normalisé.

## Algorithme

[Distance de Levenshtein](https://fr.wikipedia.org/wiki/Distance_de_Levenshtein) : nombre minimal d'insertions, suppressions ou substitutions pour transformer une chaîne en une autre. Convertie en score de similarité entre 0.0 et 1.0.

La comparaison utilise une similarité **partielle** : une requête courte est comparée à la meilleure sous-fenêtre du candidat plutôt qu'au candidat entier — "andr" trouve correctement "Andre Dubois", même si les deux chaînes ont des longueurs très différentes.

## Installation

```toml
[dependencies]
easysearch = "0.1"
```

Aucune dépendance externe : la bibliothèque repose uniquement sur `std`.

## Démarrage rapide

```rust
use easysearch::{search, SearchConfig};

let candidates = vec!["Jorge", "Andre", "Castro"];
let config = SearchConfig::default();

let results = search("Jroge", &candidates, &config);
// results[0].text == "Jorge", malgré la faute de frappe
```

## Rechercher un terme court dans un texte plus long

```rust
use easysearch::{search, SearchConfig};

let candidates = vec!["Andre Dubois", "Marie Curie"];
let config = SearchConfig::default();

let results = search("andr", &candidates, &config);
// results[0].text == "Andre Dubois"
```

## Rechercher dans des objets structurés

`search_by` permet de rechercher dans des structures complètes en indiquant quel champ comparer, tout en retournant l'objet original entier :

```rust
use easysearch::{search_by, SearchConfig};

struct User { name: String }

let users = vec![
    User { name: "Jorge".to_string() },
    User { name: "Andre".to_string() },
];

let results = search_by("Jroge", &users, |u| &u.name, &SearchConfig::default());
// results[0].item.name == "Jorge"
```

## Personnaliser la configuration

```rust
use easysearch::SearchConfig;

let config = SearchConfig::default()
    .with_min_similarity(0.6)
    .with_max_results(5)
    .with_case_insensitive(true)
    .with_prefix_bonus(0.2);
```

| Paramètre | Rôle | Par défaut |
|---|---|---|
| `min_similarity` | Score minimal (0.0-1.0) pour qu'un résultat soit retenu | `0.5` |
| `max_results` | Nombre maximal de résultats retournés | `10` |
| `case_insensitive` | Ignore la casse dans la comparaison | `true` |
| `prefix_bonus` | Bonus de score si le candidat commence par la requête | `0.15` |

## Exemple complet

```bash
cargo run --example search_demo
```

## Note de performance

La distance de Levenshtein a une complexité O(n×m) par comparaison, et `partial_similarity` répète cette comparaison pour chaque position possible dans le candidat quand la requête est plus courte — donc plus coûteuse qu'une comparaison directe. Pour de très grandes listes de candidats (plusieurs dizaines de milliers), envisagez un filtrage préalable (ex: recherche par préfixe en base de données) avant d'appliquer `easysearch` sur un sous-ensemble réduit.

## Historique des versions

- **0.1.0** — Version initiale avec `search`, `search_by`, `levenshtein_distance`, `similarity` et `partial_similarity` pour la recherche de termes courts dans des textes plus longs.

## License

GPL-2.0-or-later
Copyright (C) 2026 Jorge Andre Castro