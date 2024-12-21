# Infinite Monkey Theorem Rust CLI Application

## Overview

This Rust command-line application simulates the Infinite Monkey Theorem by creating a group of virtual "monkeys," each employing different text generation strategies. The goal is to see which strategy produces the most valid words. At the end of each run, a leaderboard will display the results, ranking the monkeys by the number of valid words they managed to generate.

---

## Features

- **Multiple Monkey Strategies:**


  - **Random:** Purely random character generation.
  - **LinearCommon:** Characters are chosen based on their linear frequency in common texts.
  - **LogCommon:** Characters are chosen with probabilities following a logarithmic distribution based on common frequency.
  - **Bigram:** Generates text based on the probabilities of bigrams (two-character sequences).
  - **Trigram:** Generates text based on the probabilities of trigrams (three-character sequences).

- **Word Count Analysis:** Monkeys analyze their generated text to count valid words.

- **Leaderboard:** Displays the ranking of monkeys by the number of correct words generated.

---

## Usage


Run the application with:

```bash
cargo run
```
