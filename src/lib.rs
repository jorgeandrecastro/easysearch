// Copyright (C) 2026 Jorge Andre Castro
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! # EasySearch
//!
//! `easysearch` est une bibliothèque légère de recherche floue (fuzzy
//! search), conçue pour le projet Hodoe. Elle permet de trouver des
//! résultats pertinents même en présence de fautes de frappe, en combinant
//! la distance de Levenshtein (nombre minimal de modifications entre deux
//! chaînes) et un score de similarité normalisé.
//!
//! La comparaison utilise une similarité "partielle" : une requête courte
//! est comparée à la meilleure sous-fenêtre du candidat plutôt qu'au
//! candidat entier, ce qui permet de bien noter un terme court apparaissant
//! n'importe où dans un texte plus long (ex: "andr" dans "Andre Dubois").
//!
//! ## Exemple d'utilisation
//!
//! ```rust
//! use easysearch::{SearchConfig, search};
//!
//! let candidates = vec!["Jorge", "Andre", "Castro", "Jorge Andre"];
//! let config = SearchConfig::default();
//!
//! let results = search("Jroge", &candidates, &config);
//! assert_eq!(results[0].text, "Jorge");
//! ```

use std::cmp::min;

/// Configuration des seuils de recherche floue d'EasySearch.
///
/// Toutes les valeurs par défaut sont accessibles via [`SearchConfig::default`],
/// et peuvent être surchargées individuellement grâce aux méthodes `with_*`
/// (pattern builder), comme dans `ethosfeed::EthosConfig`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SearchConfig {
    /// Score de similarité minimal (0.0 à 1.0) en dessous duquel un candidat
    /// est exclu des résultats. `1.0` signifie correspondance exacte.
    pub min_similarity: f64,
    /// Nombre maximal de résultats retournés, triés par pertinence décroissante.
    pub max_results: usize,
    /// Si `true`, la comparaison ignore la casse (majuscules/minuscules).
    pub case_insensitive: bool,
    /// Bonus de score additionnel accordé quand le candidat commence
    /// exactement par la requête (utile pour prioriser l'auto-complétion).
    pub prefix_bonus: f64,
}

impl Default for SearchConfig {
    /// Initialise la configuration avec des seuils standards : tolère les
    /// fautes de frappe courantes (similarité minimale de 0.5), ignore la
    /// casse, et retourne jusqu'à 10 résultats.
    fn default() -> Self {
        Self {
            min_similarity: 0.5,
            max_results: 10,
            case_insensitive: true,
            prefix_bonus: 0.15,
        }
    }
}

impl SearchConfig {
    /// Retourne une copie de la configuration avec un nouveau seuil de similarité minimal.
    pub fn with_min_similarity(mut self, min_similarity: f64) -> Self {
        self.min_similarity = min_similarity.clamp(0.0, 1.0);
        self
    }

    /// Retourne une copie de la configuration avec un nouveau nombre maximal de résultats.
    pub fn with_max_results(mut self, max_results: usize) -> Self {
        self.max_results = max_results;
        self
    }

    /// Retourne une copie de la configuration avec la sensibilité à la casse activée/désactivée.
    pub fn with_case_insensitive(mut self, case_insensitive: bool) -> Self {
        self.case_insensitive = case_insensitive;
        self
    }

    /// Retourne une copie de la configuration avec un nouveau bonus de préfixe.
    pub fn with_prefix_bonus(mut self, prefix_bonus: f64) -> Self {
        self.prefix_bonus = prefix_bonus;
        self
    }
}

/// Résultat d'une recherche floue : le texte candidat retenu, accompagné de
/// son score de pertinence.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult<'a> {
    /// Le texte candidat original (référence vers l'entrée fournie à [`search`]).
    pub text: &'a str,
    /// Score de pertinence final, entre 0.0 et potentiellement un peu plus
    /// de 1.0 grâce au bonus de préfixe. Plus le score est élevé, plus le
    /// résultat est pertinent.
    pub score: f64,
}

/// Calcule la distance de Levenshtein entre deux chaînes : le nombre minimal
/// d'insertions, suppressions ou substitutions de caractères nécessaires
/// pour transformer l'une en l'autre.
///
/// La complexité est O(n*m) en temps et en mémoire, où n et m sont les
/// longueurs des deux chaînes (en nombre de caractères, pas d'octets).
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let (a_len, b_len) = (a_chars.len(), b_chars.len());

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut previous_row: Vec<usize> = (0..=b_len).collect();
    let mut current_row = vec![0usize; b_len + 1];

    for i in 1..=a_len {
        current_row[0] = i;
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            current_row[j] = min(
                min(current_row[j - 1] + 1, previous_row[j] + 1),
                previous_row[j - 1] + cost,
            );
        }
        std::mem::swap(&mut previous_row, &mut current_row);
    }

    previous_row[b_len]
}

/// Calcule un score de similarité normalisé entre 0.0 (complètement
/// différent) et 1.0 (identique), à partir de la distance de Levenshtein
/// relative à la longueur de la plus longue des deux chaînes.
///
/// Cette comparaison est "totale" : elle considère les deux chaînes dans
/// leur intégralité. Pour comparer une requête courte à l'intérieur d'un
/// texte plus long, préférez [`partial_similarity`].
pub fn similarity(a: &str, b: &str) -> f64 {
    let max_len = a.chars().count().max(b.chars().count());
    if max_len == 0 {
        return 1.0; // deux chaînes vides sont considérées identiques
    }

    let distance = levenshtein_distance(a, b);
    1.0 - (distance as f64 / max_len as f64)
}

/// Calcule un score de similarité "partiel" : compare `query` à la meilleure
/// sous-fenêtre de `text` de longueur égale à `query`, plutôt qu'à `text`
/// dans son intégralité.
///
/// Permet de bien noter un terme court apparaissant n'importe où dans un
/// texte plus long (ex: "andr" dans "andre dubois" obtient un score élevé,
/// alors qu'une comparaison directe des deux chaînes entières donnerait un
/// score faible à cause de la grande différence de longueur).
///
/// Si `query` est plus longue ou de même longueur que `text`, se comporte
/// comme [`similarity`] (comparaison directe des deux chaînes entières).
pub fn partial_similarity(query: &str, text: &str) -> f64 {
    let query_chars: Vec<char> = query.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();

    if query_chars.is_empty() {
        return if text_chars.is_empty() { 1.0 } else { 0.0 };
    }

    if query_chars.len() >= text_chars.len() {
        return similarity(query, text);
    }

    let window_len = query_chars.len();
    let mut best_score = 0.0f64;

    for start in 0..=(text_chars.len() - window_len) {
        let window: String = text_chars[start..start + window_len].iter().collect();
        let score = similarity(query, &window);
        if score > best_score {
            best_score = score;
        }
    }

    best_score
}

/// Recherche les candidats les plus similaires à `query` parmi `candidates`,
/// selon la configuration fournie.
///
/// Les résultats sont triés par score décroissant, filtrés selon
/// `config.min_similarity`, et limités à `config.max_results` entrées.
pub fn search<'a>(
    query: &str,
    candidates: &[&'a str],
    config: &SearchConfig,
) -> Vec<SearchResult<'a>> {
    let normalized_query = if config.case_insensitive {
        query.to_lowercase()
    } else {
        query.to_string()
    };

    let mut results: Vec<SearchResult<'a>> = candidates
        .iter()
        .filter_map(|&candidate| {
            let normalized_candidate = if config.case_insensitive {
                candidate.to_lowercase()
            } else {
                candidate.to_string()
            };

            let mut score = partial_similarity(&normalized_query, &normalized_candidate);

            if normalized_candidate.starts_with(&normalized_query) && !normalized_query.is_empty() {
                score += config.prefix_bonus;
            }

            if score >= config.min_similarity {
                Some(SearchResult {
                    text: candidate,
                    score,
                })
            } else {
                None
            }
        })
        .collect();

    results.sort_unstable_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(config.max_results);
    results
}

/// Variante de [`search`] qui applique la recherche floue à un champ
/// spécifique d'objets structurés, plutôt qu'à de simples chaînes.
///
/// Utile pour rechercher parmi des enregistrements complets (ex: posts,
/// profils) en indiquant comment en extraire le texte à comparer, tout en
/// retournant l'objet original entier dans le résultat.
///
/// # Exemple
///
/// ```rust
/// use easysearch::{search_by, SearchConfig};
///
/// struct User { name: String }
///
/// let users = vec![
///     User { name: "Jorge".to_string() },
///     User { name: "Andre".to_string() },
/// ];
///
/// let results = search_by("Jroge", &users, |u| &u.name, &SearchConfig::default());
/// assert_eq!(results[0].item.name, "Jorge");
/// ```
pub fn search_by<'a, T, F>(
    query: &str,
    items: &'a [T],
    extract_text: F,
    config: &SearchConfig,
) -> Vec<SearchByResult<'a, T>>
where
    F: Fn(&T) -> &str,
{
    let normalized_query = if config.case_insensitive {
        query.to_lowercase()
    } else {
        query.to_string()
    };

    let mut results: Vec<SearchByResult<'a, T>> = items
        .iter()
        .filter_map(|item| {
            let text = extract_text(item);
            let normalized_text = if config.case_insensitive {
                text.to_lowercase()
            } else {
                text.to_string()
            };

            let mut score = partial_similarity(&normalized_query, &normalized_text);

            if normalized_text.starts_with(&normalized_query) && !normalized_query.is_empty() {
                score += config.prefix_bonus;
            }

            if score >= config.min_similarity {
                Some(SearchByResult { item, score })
            } else {
                None
            }
        })
        .collect();

    results.sort_unstable_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(config.max_results);
    results
}

/// Résultat de [`search_by`] : l'élément original retenu, accompagné de son
/// score de pertinence.
#[derive(Debug, Clone)]
pub struct SearchByResult<'a, T> {
    /// L'élément original (référence vers l'entrée fournie à [`search_by`]).
    pub item: &'a T,
    /// Score de pertinence final, voir [`SearchResult::score`].
    pub score: f64,
}

// ============================================================================
// TESTS UNITAIRES
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_identical_strings() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshtein_empty_strings() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", "abc"), 3);
    }

    #[test]
    fn test_levenshtein_single_substitution() {
        assert_eq!(levenshtein_distance("cat", "bat"), 1);
    }

    #[test]
    fn test_levenshtein_insertion_and_deletion() {
        assert_eq!(levenshtein_distance("cat", "cats"), 1);
        assert_eq!(levenshtein_distance("cats", "cat"), 1);
    }

    #[test]
    fn test_levenshtein_handles_utf8_correctly() {
        assert_eq!(levenshtein_distance("café", "cafe"), 1);
    }

    #[test]
    fn test_similarity_identical_is_one() {
        assert_eq!(similarity("hello", "hello"), 1.0);
    }

    #[test]
    fn test_similarity_empty_strings_is_one() {
        assert_eq!(similarity("", ""), 1.0);
    }

    #[test]
    fn test_similarity_completely_different() {
        let score = similarity("abc", "xyz");
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_partial_similarity_finds_short_query_in_long_text() {
        let score = partial_similarity("andr", "andre dubois");
        assert!(score > 0.9, "score attendu > 0.9, obtenu {score}");
    }

    #[test]
    fn test_partial_similarity_falls_back_to_full_similarity_when_query_longer() {
        // Quand la requête est plus longue que le texte, partial_similarity
        // doit se comporter comme similarity classique.
        let partial = partial_similarity("hello world", "hello");
        let full = similarity("hello world", "hello");
        assert_eq!(partial, full);
    }

    #[test]
    fn test_search_finds_closest_match_despite_typo() {
        let candidates = vec!["Jorge", "Andre", "Castro", "Jorge Andre"];
        let config = SearchConfig::default();

        let results = search("Jroge", &candidates, &config);

        assert!(!results.is_empty());
        assert_eq!(results[0].text, "Jorge");
    }

    #[test]
    fn test_search_finds_short_query_inside_longer_candidate() {
        let candidates = vec!["Andre Dubois", "Marie Curie"];
        let config = SearchConfig::default();

        let results = search("andr", &candidates, &config);

        assert!(!results.is_empty());
        assert_eq!(results[0].text, "Andre Dubois");
    }

    #[test]
    fn test_search_respects_min_similarity() {
        let candidates = vec!["Jorge", "complètement différent"];
        let config = SearchConfig::default().with_min_similarity(0.8);

        let results = search("Jorge", &candidates, &config);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "Jorge");
    }

    #[test]
    fn test_search_respects_max_results() {
        let candidates = vec!["test1", "test2", "test3", "test4"];
        let config = SearchConfig::default()
            .with_min_similarity(0.0)
            .with_max_results(2);

        let results = search("test", &candidates, &config);

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_is_case_insensitive_by_default() {
        let candidates = vec!["JORGE"];
        let config = SearchConfig::default();

        let results = search("jorge", &candidates, &config);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "JORGE");
    }

    #[test]
    fn test_search_case_sensitive_when_configured() {
        let candidates = vec!["JORGE"];
        let config = SearchConfig::default()
            .with_case_insensitive(false)
            .with_min_similarity(0.9);

        let results = search("jorge", &candidates, &config);

        assert!(results.is_empty());
    }

    #[test]
    fn test_search_prefix_bonus_favors_prefix_matches() {
        let candidates = vec!["Jorge Andre", "Andre Jorge"];
        let config = SearchConfig::default().with_min_similarity(0.0);

        let results = search("Jorge", &candidates, &config);

        assert_eq!(results[0].text, "Jorge Andre");
    }

    #[test]
    fn test_search_by_extracts_field_and_returns_original_item() {
        struct User {
            name: String,
        }

        let users = vec![
            User { name: "Jorge".to_string() },
            User { name: "Andre".to_string() },
        ];

        let config = SearchConfig::default();
        let results = search_by("Jroge", &users, |u| &u.name, &config);

        assert_eq!(results[0].item.name, "Jorge");
    }

    #[test]
    fn test_builder_pattern_overrides_defaults() {
        let config = SearchConfig::default()
            .with_min_similarity(0.7)
            .with_max_results(5);

        assert_eq!(config.min_similarity, 0.7);
        assert_eq!(config.max_results, 5);
        assert_eq!(config.case_insensitive, SearchConfig::default().case_insensitive);
    }

    #[test]
    fn test_min_similarity_is_clamped() {
        let config = SearchConfig::default().with_min_similarity(1.5);
        assert_eq!(config.min_similarity, 1.0);

        let config = SearchConfig::default().with_min_similarity(-0.5);
        assert_eq!(config.min_similarity, 0.0);
    }
}