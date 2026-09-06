// Copyright (C) 2026 Jorge Andre Castro
// License: GPL-2.0-or-later
//
// Exemple : recherche floue dans une liste de noms, avec des fautes de
// frappe volontaires pour illustrer la tolérance de l'algorithme.
//
// Lancer avec : cargo run --example search_demo

use easysearch::{search, SearchConfig};

fn main() {
    let candidates = vec![
        "Jorge Andre Castro",
        "Andre Dubois",
        "Marie Curie",
        "Jorge Luis Borges",
        "Sophie Marceau",
    ];

    let config = SearchConfig::default();

    let queries = ["Jroge", "andr", "borgess", "xyz"];

    println!("=========================================================");
    println!("   EXEMPLE : Recherche floue avec EasySearch                ");
    println!("=========================================================\n");

    for query in queries {
        println!("Recherche : \"{query}\"");
        let results = search(query, &candidates, &config);

        if results.is_empty() {
            println!("  (aucun résultat suffisamment pertinent)\n");
            continue;
        }

        for result in &results {
            println!("  {:.2} -> {}", result.score, result.text);
        }
        println!();
    }
}