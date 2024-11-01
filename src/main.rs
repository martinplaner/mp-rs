use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn main() {
    let dict = load_dict("words_de.txt").expect("Error loading dictionary");

    println!("{:#?}", generate_word(&dict, "mp", "-"));
    println!("{:#?}", generate_word(&dict, "mp", "-"));
    println!("{:#?}", generate_word(&dict, "mp", "-"));
    println!("{:#?}", generate_word(&dict, "abc", " "));
}

fn load_dict(path: &str) -> io::Result<HashMap<char, Vec<String>>> {
    let file = File::open(Path::new(path))?;
    let reader = BufReader::new(file);

    let mut dict: HashMap<char, Vec<String>> = HashMap::new();
    for line in reader.lines() {
        let line = line?;

        if line.starts_with('#') {
            continue;
        }

        if let Some(mut first_char) = line.chars().next() {
            first_char = first_char.to_uppercase().next().unwrap_or(first_char);
            dict.entry(first_char).or_default().push(line);
        }
    }

    Ok(dict)
}

fn generate_word(dict: &HashMap<char, Vec<String>>, term: &str, delimiter: &str) -> Option<String> {
    let mut rng = thread_rng();

    term.to_uppercase()
        .chars()
        .map(|c| {
            dict.get(&c).and_then(|words| {
                let i = rng.gen_range(0..words.len());
                words.get(i).map(|s| s.to_owned())
            })
        })
        .collect::<Option<Vec<String>>>()
        .map(|words| words.join(delimiter))
}
