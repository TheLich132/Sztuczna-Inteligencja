use std::collections::{HashSet, VecDeque};
use std::io::{self, BufRead};
use std::time::Instant;

// Struktura do przechowywania statystyk
pub struct Stats {
    /// Nazwa algorytmu.
    algo: String,

    /// Maksymalna liczba otwartych elementów.
    open_max: usize,

    /// Liczba zamkniętych elementów.
    closed: usize,

    /// Liczba znalezionych rozwiązań.
    solutions: usize,

    /// Czas trwania algorytmu.
    duration: std::time::Duration,
}

// Formatowanie struktury `Stats` do wyświetlania
impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "Algorytm: {}", self.algo)?;
        writeln!(f, "Maksymalna długość listy open: {}", self.open_max)?;
        writeln!(f, "Maksymalna długość listy closed: {}", self.closed)?;
        writeln!(f, "Liczba rozwiązanń: {}", self.solutions)?;
        writeln!(f, "Czas: {:?}", self.duration)?;
        Ok(())
    }
}

/// Funkcja rozwiązująca, który znajduje rozwiązania problemu (stany są wektorami (zadanie dodatkowe))
///
/// # Argumenty
///
/// * `size` - Rozmiar problemu
/// * `alg` - Algorytm do użycia podczas rozwiązywania problemu
/// * `benchmark` - Czy chcemetery sprawdzać, czy w podanym stanie występują konflikty
///
/// # Zwraca
/// * `Stats` - Struktura przechowująca statystyki
pub fn solver_extra(size: usize, alg: &str, benchmark: bool) -> Stats {
    // Uruchomienie timera
    let start: Instant = Instant::now();

    // Inicjalizacja zmiennych
    let mut open_max: usize = 0;
    let mut solutions: HashSet<Vec<usize>> = HashSet::new();
    let mut closed: HashSet<Vec<usize>> = HashSet::new();
    let mut open: VecDeque<Vec<usize>> = VecDeque::from(initialize_board_extra(size));
    let mut visited: HashSet<Vec<usize>> = HashSet::new(); // Pomocniczy zbiór stanów do szybszego sprawdzania
    let mut state: Vec<usize>;

    // Pętla do momentu, aż lista open jest pusta
    while !open.is_empty() {
        // Pobranie kolejnego stanu z listy open na podstawie algorytmu
        if alg == "BFS" {
            state = open.pop_front().unwrap();
        } else {
            state = open.pop_back().unwrap();
        }

        // Sprawdzenie, czy stan jest rozwiązaniem
        if state.len() == size {
            if !conflict_checker_extra(&state) {
                // Dodanie rozwiązania do zbioru solutions
                solutions.insert(state);
                break;
            } else {
                // Dodanie stanu do zbioru closed
                closed.insert(state);
            }
        } else {
            // Generowanie dzieci stanów
            let childrens: Vec<Vec<usize>> = generate_children_extra(size, &state);
            for elem in &childrens {
                if !closed.contains(elem) && !visited.contains(elem) {
                    // Dodanie dziecka do listy open
                    open.push_back(elem.clone());
                    // Dodanie dziecka do zbioru visited
                    visited.insert(elem.clone());
                }
            }
            // Dodanie bieżącego stanu do zbioru closed
            closed.insert(state);
        }
        // Aktualizacja maksymalnej długości listy open
        if open.len() > open_max {
            open_max = open.len();
        }
        print!("{}[2J", 27 as char);
        println!("Długość listy open: {}", open.len());
        println!("Długość listy closed: {}", closed.len());
    }

    // Obliczenie czasu trwania funkcji solver
    let duration: std::time::Duration = start.elapsed();

    // Wyświetlenie rozwiązań i innych informacji
    println!("\n\nRozwiązania: {:?}\nlen={}", solutions, solutions.len());
    if !solutions.is_empty() {
        for solution in solutions.iter() {
            print_board_extra(size, solution);
            println!("{:?}\n", solution);
        }
    }
    if !benchmark {
        println!("Użyty algorytm: {}", alg);
        println!("Maksymalna długość listy open: {}", open_max);
        println!("Długość listy closed: {}", closed.len());
        println!("Liczba rozwiązowań: {}", solutions.len());
        println!("Czas wykonania: {:?}", duration);

        println!("\nNaciśnij Enter, aby kontynuować...");
        let stdin = io::stdin();
        let _ = stdin.lock().lines().next(); // Poczekaj na wciśnięcie Enter
    }

    let stats: Stats = Stats {
        algo: alg.to_string(),
        open_max,
        closed: closed.len(),
        solutions: solutions.len(),
        duration,
    };

    stats
}

/// Sprawdza, czy w podanym stanie występują konflikty. (Stany są wektorami)
///
/// # Argumenty
/// * `state` - Stan reprezentowany jako wektor wartości typu usize.
///
/// # Zwraca
/// * `bool` - `true`, jeżeli występują konflikty, `false` w przeciwnym przypadku.
fn conflict_checker_extra(state: &Vec<usize>) -> bool {
    // Iteruje po wszystkich hetmanów
    for i in 0..state.len() - 1 {
        for j in i + 1..state.len() {
            // Oblicza współrzędne hetmanów
            let x_diff: isize = (i as isize - j as isize).abs();
            let y_diff: isize = (state[i] as isize - state[j] as isize).abs();

            // Sprawdza, czy występują konflikty
            if i == j || x_diff == y_diff || state[i] == state[j] {
                return true;
            }
        }
    }
    false
}

/// Inicjalizuje planszę o podanym rozmiarze. (Stany są wektorami)
///
/// # Argumenty
/// * `size` - Rozmiar planszy.
///
/// # Zwraca
/// * `Vec<Vec<usize>>` - Wektor wektorów reprezentujacy zainicjalizowane plansze.
fn initialize_board_extra(size: usize) -> Vec<Vec<usize>> {
    let mut to_return: Vec<Vec<usize>> = Vec::with_capacity(size * size);

    for i in 0..size {
        to_return.push([i].to_vec());
    }
    to_return
}

/// Generuje wszystkie możliwe stany potomne poprzez dodanie pojedynczego elementu do podanego stanu. (Stany są wektorami)
///
/// # Argumenty
/// * `size` - Rozmiar planszy.
/// * `state` - Aktualny stan.
///
/// # Zwraca
/// * `Vec<Vec<usize>>` - Wektor wektorów, gdzie każdy wektor reprezentuje stan potomny.
fn generate_children_extra(size: usize, state: &[usize]) -> Vec<Vec<usize>> {
    // Inicjalizuj pusty wektor do przechowywania stanów potomnych.
    let mut to_return: Vec<Vec<usize>> = Vec::new();

    for i in 0..size {
        if state.contains(&i) {
            continue;
        }

        // Utwórz tymczasowy wektor poprzez sklonowanie bieżącego stanu.
        let mut temp_vec: Vec<usize> = state.to_vec();

        // Dodaj bieżącą pozycję do tymczasowego wektora.
        temp_vec.push(i);

        to_return.push(temp_vec);
    }

    // Zwróć wektor stanów potomnych.
    to_return
}

/// Wyświetla plansę gry. (Stany są wektorami)
///
/// # Argumenty
/// * `size` - Rozmiar planszy gry.
/// * `state` - Stan planszy gry.
fn print_board_extra(size: usize, state: &[usize]) {
    // Tworzenie ciągu znaków reprezentującego obramowanie.
    let border: String = "|".to_string() + &"___|".repeat(size);
    // Tworzenie ciągu znaków reprezentującego obramowanie górnej krawędzi.
    let border_up: String = ".".to_string() + &"___.".repeat(size);

    // Iteracja przez każdy wiersz planszy i każdy stan.
    for (i, current_state) in state.iter().enumerate().take(size) {
        // Wyświetlanie obramowania.
        if i == 0 {
            println!("{}", border_up);
        } else {
            println!("{}", border);
        }

        // Tworzenie ciągu znaków dla aktualnego wiersza.
        let content: String = (0..size)
            .map(|j: usize| {
                if &j == current_state {
                    if (i + j) % 2 == 0 {
                        "_H_".to_string()
                    } else {
                        " H ".to_string()
                    }
                } else if (i + j) % 2 == 0 {
                    "___".to_string()
                } else {
                    "   ".to_string()
                }
            })
            .collect::<Vec<String>>()
            .join("|");

        // Wyświetlanie zawartości dla aktualnego wiersza.
        println!("|{}|", content);
    }

    // Wyświetlanie końcowego obramowania.
    println!("{}", border);
}
