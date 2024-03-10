use std::collections::{HashSet, VecDeque};
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::time::Instant;

// Struktura do przechowywania statystyk
struct Stats {
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

/// Funkcja rozwiązująca, która znajduje rozwiązania problemu
///
/// # Argumenty
///
/// * `size` - Rozmiar problemu
/// * `alg` - Algorytm do użycia podczas rozwiązywania problemu
/// * `benchmark` - Czy chcę sprawdzać, czy w podanym stanie występują konflikty
///
/// # Zwraca
/// * `Stats` - Struktura przechowująca statystyki
fn solver(size: usize, alg: &str, benchmark: bool) -> Stats {
    // Uruchomienie timera
    let start: Instant = Instant::now();

    // Inicjalizacja zmiennych
    let mut open_max: usize = 0;
    let mut solutions: HashSet<Vec<Vec<usize>>> = HashSet::new();
    let mut closed: HashSet<Vec<Vec<usize>>> = HashSet::new();
    let mut open: VecDeque<Vec<Vec<usize>>> = VecDeque::from(initialize_board(size));
    let mut visited: HashSet<Vec<Vec<usize>>> = HashSet::new(); // Pomocniczy zbiór stanów do szybszego sprawdzania
    let mut state: Vec<Vec<usize>>;

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
            if !conflict_checker(&state) {
                // Dodanie rozwiązania do zbioru solutions
                solutions.insert(state);
            } else {
                // Dodanie stanu do zbioru closed
                closed.insert(state);
            }
        } else {
            // Generowanie dzieci stanów
            let childrens: Vec<Vec<Vec<usize>>> = generate_children(size, &state);
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
            print_board(size, solution);
            println!("{:?}\n", solution);
        }
    }
    if !benchmark {
        println!("Użyty algorytm: {}", alg);
        println!("Maksymalna długość listy open: {}", open_max);
        println!("Długość listy closed: {}", closed.len());
        println!("Liczba rozwiązań: {}", solutions.len());
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
fn solver_extra(size: usize, alg: &str, benchmark: bool) -> Stats {
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

/// Funkcja wykonująca benchmark dla n-hetmanów.
///
/// # Argumenty
///
/// * `size` - Wielkość planszy.
fn benchmark(size: usize) {
    // Sprawdź czy istnieją foldery do zapisu plików
    fs::create_dir_all("./benchmarks/zadanie_podstawowe").unwrap();

    // Uruchom solver używając algorytmu BFS i zbierz statystyki
    let stats_bfs: Stats = solver(size, "BFS", true);

    // Uruchom solver używając algorytmu DFS i zbierz statystyki
    let stats_dfs: Stats = solver(size, "DFS", true);

    // Wygeneruj ścieżkę do pliku z wynikami benchmarku
    let path: String = format!(
        "./benchmarks/zadanie_podstawowe/benchmark_{}_{}_{}.txt",
        size, stats_bfs.algo, stats_dfs.algo
    );

    // Wygeneruj tekst z wynikami benchmarku
    let text: String = format!("Benchmark dla {size}-hetmanów:\n\n{stats_bfs}\n\n{stats_dfs}");

    // Utwórz plik z wynikami benchmarku
    let mut file = File::create(path).unwrap();

    // Zapisz tekst z wynikami do pliku
    file.write_all(text.as_bytes()).unwrap();
}

/// Funkcja wykonujaca benchmark dla n-hetmanów (stany są wektorami).
///
/// # Argumenty
///
/// * `size` - Wielkość planszy
fn benchmark_extra(size: usize) {
    // Sprawdź czy istnieją foldery do zapisu plików
    fs::create_dir_all("./benchmarks/zadanie_dodatkowe").unwrap();

    // Uruchom solver używając algorytmu BFS i zbierz statystyki
    let stats_bfs: Stats = solver_extra(size, "BFS", true);

    // Uruchom solver używając algorytmu DFS i zbierz statystyki
    let stats_dfs: Stats = solver_extra(size, "DFS", true);

    // Wygeneruj ścieżkę do pliku z wynikami benchmarku
    let path: String = format!(
        "./benchmarks/zadanie_dodatkowe/benchmark_extra_{}_{}_{}.txt",
        size, stats_bfs.algo, stats_dfs.algo
    );

    // Wygeneruj tekst z wynikami benchmarku
    let text: String =
        format!("Benchmark extra dla {size}-hetmanów:\n\n{stats_bfs}\n\n{stats_dfs}");

    // Utwórz plik z wynikami benchmarku
    let mut file = File::create(path).unwrap();

    // Zapisz tekst z wynikami do pliku
    file.write_all(text.as_bytes()).unwrap();
}

/// Sprawdza, czy w podanym stanie występują konflikty.
///
/// # Argumenty
///
/// * `state` - Stan reprezentowany jako wektor wektorów wartości typu usize.
///
/// # Zwraca
///
/// * `bool` - `true`, jeśli występują konflikty, `false` w przeciwnym przypadku.
fn conflict_checker(state: &Vec<Vec<usize>>) -> bool {
    // Iteruje po wszystkich parach hetmanów
    for i in 0..state.len() - 1 {
        for j in i + 1..state.len() {
            // Oblicza różnice w współrzędnych x i y
            let x_diff: isize = (state[i][0] as isize - state[j][0] as isize).abs();
            let y_diff: isize = (state[i][1] as isize - state[j][1] as isize).abs();

            // Sprawdza, czy występują konflikty
            if state[i][0] == state[j][0] || state[i][1] == state[j][1] || x_diff == y_diff {
                return true;
            }
        }
    }
    false
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

/// Inicjalizuje planszę o podanym rozmiarze.
///
/// # Argumenty
///
/// * `size` - Rozmiar planszy.
///
/// # Zwraca
///
/// * `Vec<Vec<Vec<usize>>>` - Wektor wektorów wektorów reprezentujący zainicjalizowaną planszę.
fn initialize_board(size: usize) -> Vec<Vec<Vec<usize>>> {
    let mut to_return: Vec<Vec<Vec<usize>>> = Vec::with_capacity(size * size);

    for i in 0..size {
        for j in 0..size {
            to_return.push([[i, j].to_vec()].to_vec());
        }
    }
    to_return
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

/// Generuje wszystkie możliwe stany potomne poprzez dodanie pojedynczego elementu do podanego stanu.
///
/// # Argumenty
///
/// * `size` - Rozmiar planszy.
/// * `state` - Aktualny stan.
///
/// # Zwraca
///
/// * `Vec<Vec<Vec<usize>>>` - Wektor wektorów, gdzie każdy wektor reprezentuje stan potomny.
fn generate_children(size: usize, state: &[Vec<usize>]) -> Vec<Vec<Vec<usize>>> {
    // Inicjalizuj pusty wektor do przechowywania stanów potomnych.
    let mut to_return: Vec<Vec<Vec<usize>>> = Vec::new();

    // Iteruj po każdym polu planszy.
    for i in 0..size {
        for j in 0..size {
            // Pomijaj, jeśli bieżąca pozycja już istnieje w stanie.
            if state.contains(&[i, j].to_vec()) {
                continue;
            }

            // Utwórz tymczasowy wektor poprzez sklonowanie bieżącego stanu.
            let mut temp_vec: Vec<Vec<usize>> = state.to_vec();

            // Dodaj bieżącą pozycję do tymczasowego wektora.
            temp_vec.push([i, j].to_vec());

            // Posortuj tymczasowy wektor w kolejności rosnącej.
            temp_vec.sort_unstable();

            // Dodaj tymczasowy wektor do wektora stanów potomnych.
            to_return.push(temp_vec);
        }
    }

    // Zwróć wektor stanów potomnych.
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

/// Wyświetla planszę gry.
///
/// # Argumenty
///
/// * `size` - Rozmiar planszy gry.
/// * `state` - Stan planszy gry.
fn print_board(size: usize, state: &[Vec<usize>]) {
    // Tworzenie ciągu znaków reprezentującego obramowanie.
    let border: String = "|".to_string() + &"___|".repeat(size);
    // Tworzenie ciągu znaków reprezentującego obramowanie górnej krawędzi.
    let border_up: String = ".".to_string() + &"___.".repeat(size);

    // Iteracja przez każdy wiersz planszy.
    for i in 0..size {
        // Wyświetlanie obramowania.
        if i == 0 {
            println!("{}", border_up);
        } else {
            println!("{}", border);
        }

        // Tworzenie ciągu znaków dla aktualnego wiersza.
        let content: String = (0..size)
            .map(|j: usize| {
                if state.contains(&[i, j].to_vec()) {
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

/// Pobiera od użytkownika liczbę hetmanów na szachownicy.
///
/// # Zwraca
/// * `usize` - Liczba hetmanów
fn change_size() -> usize {
    loop {
        println!("Ile hetmanów (Powyżej 6 działa wolno dla normalnego zadania): ");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let trimmed_input = input.trim();

        // Sprawdź, czy podany argument jest poprawną liczbą
        if let Ok(size) = trimmed_input.parse::<usize>() {
            // Sprawdź, czy rozmiar planszy nie jest 2 lub 3
            if size == 2 || size == 3 {
                println!("Dla planszy o rozmiarach 2x2 i 3x3 nie ma rozwiązania\n");
            } else {
                return size;
            }
        } else {
            println!("Argument musi być liczbą\n");
        }
    }
}

fn main() {
    let mut input: String;
    let mut size: usize = change_size();
    let mut extra: bool = false;

    // Główna pętla
    loop {
        input = "".to_string();
        println!("\n\nAlgorytm ({} hetmanów, zadanie dodatkowe = {}):\n1. BFS\n2. DFS\n3. Benchmark\n4. Przełącz zadanie dodatkowe\n\n9. Zmień ilość hetmanów\n0. Koniec\nWybierz: ", size, extra);
        std::io::stdin().read_line(&mut input).unwrap();

        // Parsowanie wprowadzonych danych przez użytkownika
        match input.trim().parse() {
            Ok(1) => {
                // Wywołanie funkcji solver z algorytmem BFS
                _ = if !extra {
                    solver(size, "BFS", false)
                } else {
                    solver_extra(size, "BFS", false)
                }
            }
            Ok(2) => {
                // Wywołanie funkcji solver z algorytmem DFS
                _ = if !extra {
                    solver(size, "DFS", false)
                } else {
                    solver_extra(size, "DFS", false)
                }
            }
            Ok(3) => {
                // Wywołanie funkcji benchmark
                if !extra {
                    benchmark(size)
                } else {
                    benchmark_extra(size)
                }
            }
            Ok(4) => {
                extra = !extra; // Przełączenie zadania dodatkowego
            }
            Ok(9) => {
                size = change_size(); // Wywołanie funkcji change_size do aktualizacji rozmiaru
            }
            Ok(0) => break, // Zakończenie pętli w przypadku wprowadzenia przez użytkownika wartości 0
            _ => continue,  // Kontynuacja pętli dla nieprawidłowych danych wejściowych
        }
    }
}
