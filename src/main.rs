use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};
use std::{
    collections::{HashMap, HashSet},
    fs::{read_to_string, File},
    io::{self, BufRead},
    ops::Range,
    thread::{self},
};

fn generate_bigram_map(corpus: &str) -> HashMap<String, usize> {
    let mut bigram_map = HashMap::new();
    let chars: Vec<char> = corpus.chars().collect();

    for window in chars.windows(2) {
        if let [a, b] = window {
            let bigram = format!("{}{}", a, b);
            *bigram_map.entry(bigram).or_insert(0) += 1;
        }
    }

    bigram_map
}

fn generate_trigram_map(corpus: &str) -> HashMap<String, usize> {
    let mut trigram_map = HashMap::new();
    let chars: Vec<char> = corpus.chars().collect();

    for window in chars.windows(3) {
        if let [a, b, c] = window {
            let trigram = format!("{}{}{}", a, b, c);
            *trigram_map.entry(trigram).or_insert(0) += 1;
        }
    }

    trigram_map
}

fn main() {
    let common_first_alphabet = vec![
        'e', 'i', 'a', 'n', 'o', 's', 'r', 't', 'l', 'c', 'u', 'd', 'p', 'm', 'h', 'g', 'y', 'b',
        'f', 'v', 'k', 'w', 'z', 'x', 'q', 'j',
    ];

    let bible = read_to_string("bible.txt").unwrap();

    println!("Generating bigram from bible.");
    let bigram_map = generate_bigram_map(
        bible
            .chars()
            .filter(|c| c.is_alphabetic()) // Keep only alphabetic characters
            .collect::<String>()
            .to_lowercase()
            .as_str(),
    );

    println!("Generating trigram from bible.");
    let trigram_map = generate_trigram_map(
        bible
            .chars()
            .filter(|c| c.is_alphabetic()) // Keep only alphabetic characters
            .collect::<String>()
            .to_lowercase()
            .as_str(),
    );

    let words_file_path = "words.txt";
    let names_file_path = "first-names.txt";

    let file = File::open(words_file_path).unwrap();
    let reader = io::BufReader::new(file);

    let mut hashset = HashSet::new();
    println!("Loading words");
    for line in reader.lines() {
        let word = line.unwrap().trim().to_string();
        hashset.insert(word);
    }
    println!("Loaded {} words", hashset.len());
    let mut names = Vec::new();
    let file = File::open(names_file_path).unwrap();
    let reader = io::BufReader::new(file);
    for line in reader.lines() {
        let name = line.unwrap().trim().to_string();
        names.push(name);
    }
    names.shuffle(&mut rand::thread_rng());
    let mut names = names.iter();

    // let hashset = HashSet::from([String::from("among us")]);

    let mut search_results = Vec::new();
    let monkey_amount = 10;

    for strategy in [
        TypingStrategy::Random,
        TypingStrategy::LinearCommon,
        TypingStrategy::LogCommon,
        TypingStrategy::Bigram,
        TypingStrategy::Trigram,
    ] {
        for _ in 0..monkey_amount {
            let letters_to_use = common_first_alphabet.clone();
            let search_set = hashset.clone();
            let mut monkey = Monkey::new(strategy.prefix() + names.next().unwrap());
            let strategy = strategy.clone();
            let bigram_map = bigram_map.clone();
            let trigram_map = trigram_map.clone();

            let monkey_handle = thread::spawn(move || {
                monkey.type_and_count(
                    letters_to_use,
                    100000,
                    4..9,
                    search_set,
                    strategy,
                    Some(&bigram_map),
                    Some(&trigram_map),
                )
            });
            search_results.push(monkey_handle);
        }
    }

    let mut total_word_occurrences = HashMap::<String, usize>::new();
    let mut monkey_leaderboard = Vec::<(String, HashMap<String, usize>)>::new();
    for search_result in search_results {
        let (monkey_name, result) = search_result.join().unwrap();
        monkey_leaderboard.push((monkey_name, result.clone()));
        for (occurrence, count) in result {
            total_word_occurrences
                .entry(occurrence)
                .and_modify(|e| *e += count)
                .or_insert(count);
        }
    }
    let mut occurrences_vec: Vec<(&String, &usize)> = total_word_occurrences.iter().collect();

    monkey_leaderboard.sort_by(|a, b| {
        b.1.iter()
            .map(|w| w.1)
            .sum::<usize>()
            .cmp(&a.1.iter().map(|w| w.1).sum::<usize>())
    });

    occurrences_vec.sort_by(|a, b| b.1.cmp(a.1));
    let top_occurrences: Vec<_> = occurrences_vec.into_iter().take(10).collect();
    let top_monkeys: Vec<_> = monkey_leaderboard.into_iter().take(10).collect();

    for (occurrence, count) in top_occurrences {
        println!("{occurrence} was typed {count} times");
    }
    for (placement, (monkey, words)) in top_monkeys.iter().enumerate() {
        println!(
            "{}. {monkey} typed {} words",
            placement + 1,
            words.iter().map(|w| w.1).sum::<usize>()
        );
    }
}

#[derive(Clone)]
enum TypingStrategy {
    Random,
    LinearCommon,
    LogCommon,
    Bigram,
    Trigram,
}

impl TypingStrategy {
    fn prefix(&self) -> String {
        match self {
            TypingStrategy::Random => String::from("random_"),
            TypingStrategy::LinearCommon => String::from("linear_"),
            TypingStrategy::LogCommon => String::from("log_"),
            TypingStrategy::Bigram => String::from("bigram"),
            TypingStrategy::Trigram => String::from("trigram"),
        }
    }
}

struct Monkey {
    name: String,
}

impl Monkey {
    fn new(name: String) -> Self {
        Self { name }
    }

    fn type_and_count(
        &mut self,
        typewriter: Vec<char>,
        letters_to_type: usize,
        word_search_window: Range<usize>,
        search_set: HashSet<String>,
        strategy: TypingStrategy,
        bigram_map: Option<&HashMap<String, usize>>,
        trigram_map: Option<&HashMap<String, usize>>,
    ) -> (String, HashMap<String, usize>) {
        let mut r = rand::thread_rng();
        let mut page = String::new();
        for _ in 0..letters_to_type {
            match strategy {
                TypingStrategy::Random => {
                    page.push(*typewriter.get(r.gen_range(0..typewriter.len())).unwrap())
                }
                TypingStrategy::LinearCommon => {
                    page.push(get_weighted_random_letter(typewriter.as_slice(), &mut r))
                }
                TypingStrategy::LogCommon => page.push(get_weighted_random_letter_logarithmic(
                    typewriter.as_slice(),
                    &mut r,
                )),
                TypingStrategy::Bigram => {
                    if let Some(bigram_map) = bigram_map {
                        if page.is_empty() {
                            // Start with a random character
                            page.push(*typewriter.get(r.gen_range(0..typewriter.len())).unwrap());
                        } else {
                            // Use the last character to select the next
                            let last_char = page.chars().last().unwrap();
                            let candidates: Vec<_> = bigram_map
                                .iter()
                                .filter(|(k, _)| k.starts_with(last_char))
                                .collect();

                            if let Some((next_bigram, _)) = candidates.choose(&mut r) {
                                page.push(next_bigram.chars().nth(1).unwrap());
                            } else {
                                // Fallback to random if no match
                                page.push(
                                    *typewriter.get(r.gen_range(0..typewriter.len())).unwrap(),
                                );
                            }
                        }
                    }
                }
                TypingStrategy::Trigram => {
                    if let Some(trigram_map) = trigram_map {
                        if page.len() < 2 {
                            // Start with random characters if not enough context
                            page.push(*typewriter.get(r.gen_range(0..typewriter.len())).unwrap());
                        } else {
                            // Use the last two characters to select the next
                            let last_two_chars: String = page[page.len() - 2..].chars().collect();
                            let candidates: Vec<_> = trigram_map
                                .iter()
                                .filter(|(k, _)| k.starts_with(&last_two_chars))
                                .collect();

                            if let Some((next_trigram, _)) = candidates.choose(&mut r) {
                                page.push(next_trigram.chars().nth(2).unwrap());
                            } else {
                                // Fallback to random if no match
                                page.push(
                                    *typewriter.get(r.gen_range(0..typewriter.len())).unwrap(),
                                );
                            }
                        }
                    }
                }
            }
        }
        println!(
            "{} finished writing {letters_to_type} characters",
            self.name
        );

        let mut search_occurrences = HashMap::new();
        for word_len in word_search_window {
            page.as_bytes().windows(word_len).for_each(|w| {
                if let Ok(word) = String::from_utf8(w.to_vec()) {
                    if search_set.contains(&word) {
                        search_occurrences
                            .entry(word)
                            .and_modify(|e| *e += 1)
                            .or_insert(1);
                    }
                }
            });
        }
        println!(
            "{} finished counting and found {} occurrences",
            self.name,
            search_occurrences.len()
        );

        (self.name.clone(), search_occurrences)
    }
}

fn get_weighted_random_letter(common_first_alphabet: &[char], thread_rng: &mut ThreadRng) -> char {
    let total_elements = common_first_alphabet.len();
    let weights: Vec<usize> = (0..total_elements).rev().collect(); // [n-1, n-2, ..., 1, 0]
    let total_weight: usize = weights.iter().sum();
    let random_weight: usize = thread_rng.gen_range(0..total_weight);
    let mut cumulative_weight = 0;
    for (index, &weight) in weights.iter().enumerate() {
        cumulative_weight += weight;
        if random_weight < cumulative_weight {
            return common_first_alphabet.get(index).copied().unwrap();
        }
    }
    common_first_alphabet.first().copied().unwrap()
}

fn get_weighted_random_letter_logarithmic(
    common_first_alphabet: &[char],
    thread_rng: &mut ThreadRng,
) -> char {
    let weights: Vec<f64> = (1..=common_first_alphabet.len())
        .map(|i| (common_first_alphabet.len() as f64 - i as f64 + 1.0).ln())
        .collect();
    let total_weight: f64 = weights.iter().sum();
    let random_weight: f64 = thread_rng.gen_range(0.0..total_weight);
    let mut cumulative_weight = 0.0;
    for (index, &weight) in weights.iter().enumerate() {
        cumulative_weight += weight;
        if random_weight < cumulative_weight {
            return common_first_alphabet.get(index).copied().unwrap();
        }
    }

    common_first_alphabet.first().copied().unwrap()
}
