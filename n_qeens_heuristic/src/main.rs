use priority_queue::DoublePriorityQueue;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::time::Instant;

mod n_hetman_dfs_bfs;

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

/// Funkcja rozwiązująca, który znajduje rozwiązania problemu (stany są wektorami)
///
/// # Argumenty
///
/// * `size` - Rozmiar problemu.
/// * `benchmark` - Flaga wskazująca, czy algorytm ma być uruchamiany w trybie benchmark.
/// * `heurystyka` - Funkcja do sprawdzania heurystyki
/// * `func_name` - Nazwa funkcji sprawdzającej heurystyki
///
/// # Zwraca
/// * `Stats` - Struktura przechowująca statystyki
fn solver(
    size: usize,
    benchmark: bool,
    heurystyka: fn(&[usize], usize) -> usize,
    func_name: &str,
) -> Stats {
    // Zaczynamy mierzyć czas wykonania
    let start: Instant = Instant::now();

    // Inicjalizujemy zmienne
    let mut open_max: usize = 0; // Maksymalna długość listy open
    let mut solutions: HashSet<Vec<usize>> = HashSet::new(); // Zbiór rozwiązań
    let mut closed: HashSet<Vec<usize>> = HashSet::new(); // Zbiór stanów, które zostały już przetworzone
    let mut open: DoublePriorityQueue<Vec<usize>, usize> =
        DoublePriorityQueue::from(initialize_board(size, func_name)); // Lista stanów do sprawdzenia
    let mut visited: HashSet<Vec<usize>> = HashSet::new(); // Zbiór stanów, które zostały juz dodane do listy open
    let mut state: Vec<usize>; // Bieżący stan

    // Pętla do wykonania algorytmu
    while let Some(elem) = open.pop_min() {
        state = elem.0;

        // Sprawdzamy, czy stan jest rozwiązaniem
        if state.len() == size {
            if !conflict_checker(&state) {
                // Jeżeli stan jest rozwiązaniem, dodajemy go do listy rozwiązan i kończymy dalsze sprawdzanie
                solutions.insert(state);
                break;
            } else {
                closed.insert(state);
            }
        } else {
            // Generujemy potomków bieżącego stanu
            let childrens: Vec<(Vec<usize>, usize)> =
                generate_children(size, &state, heurystyka, func_name);
            for elem in &childrens {
                if !closed.contains(&elem.0) && !visited.contains(&elem.0) {
                    open.push(elem.0.clone(), elem.1);
                    visited.insert(elem.0.clone());
                }
            }
            closed.insert(state);
        }

        // Aktualizujemy długość listy open
        if open.len() > open_max {
            open_max = open.len();
        }

        // Wyświetlamy informacje o postępie
        print!("{}[2J", 27 as char); // Czyszczenie ekranu
        println!("Długość listy open: {}", open.len());
        println!("Długość listy closed: {}", closed.len());
    }

    // Mierzymy czas wykonania
    let duration: std::time::Duration = start.elapsed();

    // Wyświetlamy rozwiązania
    println!("\n\nRozwiązania: {:?}\nlen={}", solutions, solutions.len());
    if !solutions.is_empty() {
        for solution in solutions.iter() {
            print_board(size, solution);
            println!("{:?}\n", solution);
        }
    }

    // Wyświetlamy statystyki jeżeli nie jest w trybie benchmark
    if !benchmark {
        println!("Użyty algorytm: {}", func_name);
        println!("Maksymalna długość listy open: {}", open_max);
        println!("Długość listy closed: {}", closed.len());
        println!("Liczba rozwiązań: {}", solutions.len());
        println!("Czas wykonania: {:?}", duration);

        println!("\nNaciśnij Enter, aby kontynuować...");
        let stdin = io::stdin();
        let _ = stdin.lock().lines().next();
    }

    // Tworzymy obiekt statystyk
    let stats: Stats = Stats {
        algo: func_name.to_string(),
        open_max,
        closed: closed.len(),
        solutions: solutions.len(),
        duration,
    };

    // Zwracamy obiekt statystyk
    stats
}

/// Oblicza wartość heurystyczną dla danego stanu.
///
/// # Argumenty
///
/// * `state` - Stan reprezentowany jako wektor liczb całkowitych.
/// * `_` - Symbol zastępczy dla drugiego argumentu, który nie jest używany w tej funkcji.
///
/// # Zwraca
///
/// * `usize` - Wartość heurystyczną jako liczba całkowita.
fn heuristic1(state: &[usize], _: usize) -> usize {
    let mut heuristic: usize = 0;
    // Iteruje po wszystkich hetmanów
    for i in 0..state.len() - 1 {
        for j in i + 1..state.len() {
            // Oblicza współrzędne hetmanów
            let x_diff: isize = (i as isize - j as isize).abs();
            let y_diff: isize = (state[i] as isize - state[j] as isize).abs();

            // Sprawdza, czy występują konflikty
            if i == j || x_diff == y_diff || state[i] == state[j] {
                heuristic += 1;
            }
        }
    }
    heuristic
}

/// Oblicza wartość heurystyczną dla danego stanu.
/// Funkcja wykorzystuje wynik z funkcji heuristic1 i niektórych dodatkowych reguł.
///
/// # Argumenty
///
/// * `state` - Stan reprezentowany jako wektor liczb całkowitych.
/// * `size` - Rozmiar stanu
///
/// # Zwraca
///
/// * `usize` - Wartość heurystyczną jako liczba całkowita.
fn heuristic2(state: &[usize], size: usize) -> usize {
    // Inicjalizacja wartości heurystyki na 0
    let mut heuristic: usize = 0;

    // Obliczenie wartości heurystyki1 dla danego stanu i rozmiaru
    let h1_value: usize = heuristic1(state, size);

    // Sprawdzenie, czy długość stanu jest równa rozmiarowi
    if state.len() == size {
        // Sprawdzenie, czy pierwszy i ostatni element stanu są równie 0 lub rozmiar - 1
        heuristic = if (state[0] == 0 || state[0] == size - 1)
            && (state[size - 1] == 0 || state[size - 1] == size - 1)
        {
            1
        } else {
            0
        };
    } else if state[0] == 0 || state[0] == size - 1 {
        // Jeżeli pierwszy element stanu jest równy 0 lub rozmiar - 1, ustaw wartość heurystyki na 1
        heuristic = 1;
    }

    // Jeżeli wartość heurystyki1 wynosi 0, zwróć wartość heurystyki
    // W przeciwnym razie, dodaj 1 do wartości heurystyki i zwróć ją
    if h1_value == 0 {
        heuristic
    } else {
        heuristic + 1
    }
}

/// Oblicza wartość heurystyczną dla danego stanu licząc odległość Manhattan.
///
/// # Argumenty
///
/// * `state` - Stan reprezentowany jako wektor liczb całkowitych.
/// * `size` - Rozmiar stanu
///
/// # Zwraca
///
/// * `usize` - Wartość heurystyczną jako liczba całkowita.
// TODO: znaleźć sposób na przyspieszenie
fn heuristic3(state: &[usize], size: usize) -> usize {
    let mut manhattan_total: usize = 0;
    let mut steps: usize = 0;

    for i in 0..state.len() - 1 {
        for j in i + 1..i + 4 {
            if j > state.len() - 1 {
                break;
            }
            steps += 1;

            // Obliczanie odległości Manhattan
            let manhattan_dist: isize =
                ((i as isize) - j as isize).abs() + ((state[i] as isize) - state[j] as isize).abs();

            if manhattan_dist == 3 {
                manhattan_total += 1;
            }
        }
    }
    (steps + (size - state.len())) - manhattan_total
}

/// Funkcja do testowania wydajności funkcji solvera dla różnych heurystyk.
/// Tworzy katalog, w którym będą przechowywane wyniki testów, uruchamia funkcję solvera dla różnych heurystyk,
/// a następnie zapisuje wyniki do pliku tekstowego.
///
/// # Argumenty
///
/// * `size` - Rozmiar problemu do rozwiązywania.
fn benchmark(size: usize, benchmark_dfs_bfs: bool) {
    // Tworzenie katalogu do przechowywania wyników testów
    let directory: &str = if !benchmark_dfs_bfs {
        "./benchmarks"
    } else {
        "./benchmarks/with_dfs_bfs"
    };
    fs::create_dir_all(directory).unwrap();

    // Uruchamianie funkcji solvera dla różnych heurystyk
    let stats_h1: Stats = solver(size, true, heuristic1, "H1");
    let stats_h2: Stats = solver(size, true, heuristic2, "H2");
    let stats_h3: Stats = solver(size, true, heuristic3, "H3");

    if !benchmark_dfs_bfs {
        // Tworzenie ścieżki do pliku tekstowego
        let path: String = format!("{}/benchmark_{}.txt", directory, size);

        // Tworzenie tekstu do zapisania do pliku
        let text: String = format!(
            "Test wydajności dla {size}-hetmana:\n\n{stats_h1}\n\n{stats_h2}\n\n{stats_h3}"
        );

        // Tworzenie pliku na podanej ścieżce
        let mut file: File = File::create(path).unwrap();

        // Zapisywanie tekstu do pliku
        file.write_all(text.as_bytes()).unwrap();
    } else {
        let stats_bfs: n_hetman_dfs_bfs::Stats = n_hetman_dfs_bfs::solver_extra(size, "BFS", true);
        let stats_dfs: n_hetman_dfs_bfs::Stats = n_hetman_dfs_bfs::solver_extra(size, "DFS", true);

        // Tworzenie ścieżki do pliku tekstowego
        let path: String = format!("{}/benchmark_{}_with_dfs_bfs.txt", directory, size);

        // Tworzenie tekstu do zapisania do pliku
        let text: String = format!(
            "Test wydajności dla {size}-hetmanów:\n\n{stats_h1}\n\n{stats_h2}\n\n{stats_h3}\n\n{stats_bfs}\n\n{stats_dfs}"
        );

        // Tworzenie pliku na podanej ścieżce
        let mut file: File = File::create(path).unwrap();

        // Zapisywanie tekstu do pliku
        file.write_all(text.as_bytes()).unwrap();
    }
}

/// Sprawdza, czy w podanym stanie występują konflikty. (Stany są wektorami)
///
/// # Argumenty
/// * `state` - Stan reprezentowany jako wektor wartości typu usize.
///
/// # Zwraca
/// * `bool` - `true`, jeżeli występują konflikty, `false` w przeciwnym przypadku.
fn conflict_checker(state: &Vec<usize>) -> bool {
    // Iteruje po wszystkich hetmanach
    for i in 0..state.len() - 1 {
        for j in i + 1..state.len() {
            // Oblicza współrzędne hetmanów
            let x_diff: isize = (i as isize - j as isize).abs();
            let y_diff: isize = (state[i] as isize - state[j] as isize).abs();

            // Sprawdza, czy występują konflikty
            if x_diff == y_diff || state[i] == state[j] {
                return true;
            }
        }
    }
    false
}

/// Funkcja inicjuje planszę o podanej rozmiarze.
///
/// # Argumenty
///
/// * `size` - Rozmiar planszy do zainicjowania.
/// * `func_name` - Nazwa funkcji.
///
/// # Zwraca
///
/// * `Vec<(Vec<usize>, usize)>` - Wektor, w którym każdy element to para: wektor reprezentujący wiersz planszy oraz wagę.
fn initialize_board(size: usize, func_name: &str) -> Vec<(Vec<usize>, usize)> {
    // Inicjalizacja pustego wektora do przechowywania planszy
    let mut to_return = Vec::new();

    // Pętla iterująca po każdym indeksie na planszy
    for i in 0..size {
        // Jeżeli nazwa funkcji to "H2"
        if func_name == "H2" {
            // Rozszerzanie wektora to_return o pary wartosci
            to_return.extend((0..size).map(|_| ([size, i].to_vec(), usize::MAX)));
        } else {
            // Rozszerzanie wektora to_return o pary wartosci
            to_return.extend((0..size).map(|_| ([i].to_vec(), usize::MAX)));
        }
    }
    // Zwracanie zainicjowanej planszy
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
fn generate_children(
    size: usize,
    state: &Vec<usize>,
    heurystyka: fn(&[usize], usize) -> usize,
    func_name: &str,
) -> Vec<(Vec<usize>, usize)> {
    // Inicjalizuj pusty wektor do przechowywania stanów potomnych.
    let mut to_return: Vec<(Vec<usize>, usize)> = Vec::new();

    for i in 0..size {
        if state.contains(&i) {
            continue;
        }

        // Utwórz tymczasowy wektor poprzez sklonowanie bieżącego stanu.
        let mut temp_vec: Vec<usize> = state.clone();

        // Dodaj bieżącą pozycję do tymczasowego wektora.
        if func_name == "H2" && state.len() == size - 1 && state[0] == size {
            temp_vec[0] = i;
        } else {
            temp_vec.push(i);
        }

        let h: usize = heurystyka(&temp_vec, size);

        to_return.push((temp_vec, h));
    }

    // Zwróć wektor stanów potomnych.
    to_return
}

/// Wyświetla plansę gry. (Stany są wektorami)
///
/// # Argumenty
/// * `size` - Rozmiar planszy gry.
/// * `state` - Stan planszy gry.
fn print_board(size: usize, state: &[usize]) {
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
    let mut benchamrk_dfs_bfs: bool = false;
    let mut input: String;
    let mut size: usize = change_size();

    // Główna pętla
    loop {
        input = "".to_string();
        println!("\n\nAlgorytm ({} hetmanów, benchmark z DFS i BFS = {}):\n1. Heurystyka H1\n2. Heurystyka H2\n3. Heurystyka H3\n4. Benchmark\n5. DFS\n6. BFS\n\n8. Zmień tryb benchmark\n9. Zmień ilość hetmanów\n0. Koniec\nWybierz: ", size, benchamrk_dfs_bfs);
        std::io::stdin().read_line(&mut input).unwrap();

        // Parsowanie wprowadzonych danych przez użytkownika
        match input.trim().parse() {
            Ok(1) => {
                // Wywołanie funkcji solver z Heurystyka H1
                _ = solver(size, false, heuristic1, "H1");
            }
            Ok(2) => {
                // Wywołanie funkcji solver z Heurystyka H2
                _ = solver(size, false, heuristic2, "H2");
            }
            Ok(3) => {
                // Wywołanie funkcji solver z Heurystyka H3
                _ = solver(size, false, heuristic3, "H3");
            }
            Ok(4) => {
                // Wywołanie funkcji benchmark
                benchmark(size, benchamrk_dfs_bfs);
            }
            Ok(5) => {
                // Wywołanie funkcji solver z algorytmem DFS
                _ = n_hetman_dfs_bfs::solver_extra(size, "DFS", false);
            }
            Ok(6) => {
                // Wywołanie funkcji solver z algorytmem BFS
                _ = n_hetman_dfs_bfs::solver_extra(size, "BFS", false);
            }
            Ok(8) => {
                benchamrk_dfs_bfs = !benchamrk_dfs_bfs;
            }
            Ok(9) => {
                size = change_size(); // Wywołanie funkcji change_size do aktualizacji rozmiaru
            }
            Ok(0) => break, // Zakończenie pętli w przypadku wprowadzenia przez użytkownika wartości 0
            _ => continue,  // Kontynuacja pętli dla nieprawidłowych danych wejściowych
        }
    }
}
