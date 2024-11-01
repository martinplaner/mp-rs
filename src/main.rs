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
    println!("{:#?}", generate_word("abc", &dict));
}

fn generate_word(term: &str, dict: &HashMap<char, Vec<String>>) -> Option<String> {
    let mut rng = thread_rng();

    term.chars()
        .map(|c| {
            dict.get(&c).and_then(|words| {
                let i = rng.gen_range(0..words.len());
                words.get(i).map(|s| s.as_str())
            })
        })
        .collect::<Option<Vec<&str>>>()
        .map(|words| words.join("-"))
}
