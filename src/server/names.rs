use rand::seq::SliceRandom;

const ADJECTIVES: &[&str] = &[
    "sensible", "funny", "cool", "fast", "brave", "bright", "calm", "clever", "gentle", "happy",
    "kind", "lucky", "mighty", "nice", "proud", "quick", "sharp", "smart", "strong", "wise",
];

const ANIMALS: &[&str] = &[
    "pangolin", "badger", "cat", "dog", "eagle", "fox", "goose", "hawk", "ibis", "jackal", "koala",
    "lemur", "mouse", "newt", "owl", "panda", "quail", "rabbit", "seal", "tiger",
];

pub fn generate_random_name() -> String {
    let mut rng = rand::thread_rng();
    let adj = ADJECTIVES.choose(&mut rng).unwrap_or(&"cool");
    let anim = ANIMALS.choose(&mut rng).unwrap_or(&"panda");
    format!("{}.{}", adj, anim)
}
