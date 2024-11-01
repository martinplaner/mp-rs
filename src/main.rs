use rand::{thread_rng, Rng};
use std::collections::HashMap;

fn main() {
    let dict: HashMap<char, Vec<String>> = HashMap::from([
        ('m', vec!["mittag".to_owned(), "monster".to_owned()]),
        ('p', vec!["pause".to_owned(), "problem".to_owned()]),
    ]);

    println!("{:#?}", generate_word("mp", &dict));
    println!("{:#?}", generate_word("mp", &dict));
    println!("{:#?}", generate_word("mp", &dict));
}

fn generate_word(term: &str, dict: &HashMap<char, Vec<String>>) -> String {
    term.chars()
        .map(|c| {
            let words = dict.get(&c).unwrap();
            let i = thread_rng().gen_range(0..words.len());
            words.get(i).unwrap().to_owned()
        })
        .collect::<Vec<String>>()
        .join("-")
}
