use rand::{thread_rng, Rng};
use std::collections::HashMap;

fn main() {
    let dict: HashMap<char, Vec<&str>> = HashMap::from([
        ('m', vec!["mittag", "monster"]),
        ('p', vec!["pause", "problem"]),
    ]);

    println!("{:#?}", generate_word("mp", &dict));
    println!("{:#?}", generate_word("mp", &dict));
    println!("{:#?}", generate_word("mp", &dict));
    println!("{:#?}", generate_word("abc", &dict));
}

fn generate_word(term: &str, dict: &HashMap<char, Vec<&str>>) -> Option<String> {
    let mut rng = thread_rng();

    term.chars()
        .map(|c| {
            dict.get(&c).and_then(|words| {
                let i = rng.gen_range(0..words.len());
                words.get(i).map(|&s| s)
            })
        })
        .collect::<Option<Vec<&str>>>()
        .map(|words| words.join("-"))
}
