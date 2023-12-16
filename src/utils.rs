//! Utility functions used throughout the application

use rand::seq::SliceRandom;

static COLOURS: [&str; 8] = [
    "teal lighten-1",
    "orange accent-3",
    "blue-grey lighten-1",
    "deep-orange darken-2",
    "lime darken-1",
    "cyan lighten-1",
    "purple lighten-1",
    "pink darken-1",
];

/// Generates a random colour classes for the background (yes its bad but
/// funny)
pub fn random_colour() -> &'static str {
    COLOURS.choose(&mut rand::thread_rng()).unwrap()
}
