use plotly::{
    {Layout, Plot, Scatter},
    common::Mode,
    layout::Axis,
};
use rand::Rng;
use rayon::prelude::*;
use std::fmt;

struct Params {
    n: usize,       // rozmiar szachownicy i liczba hetmanów
    pop: usize,     // wielkość populacji
    gen_max: usize, // maksymalna liczba generacji
    p_c: f64,       // prawdopodobienstwo krzyzowania
    p_m: f64,       // prawdopodobienstwo mutacji
    tournament_size: usize, // wielkość turnieju
}

fn init_params(n: usize, pop: usize, gen_max: usize, p_c: f64, p_m: f64, tournament_size: usize) -> Params {
    Params {
        n,
        pop,
        gen_max,
        p_c,
        p_m,
        tournament_size,
    }
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "n={}, pop={}, gen_max={}, p_c={}, p_m={}",
            self.n, self.pop, self.gen_max, self.p_c, self.p_m
        )
    }
}

/// Uruchamia algorytm ewolucyjny do rozwiązania problemu N-hetmanów.
///
/// # Argumenty
///
/// * `params` - Parametry dla algorytmu.
///
/// # Zwraca
///
/// * `solution` - Kolejny pokolenie.
/// * `evaluations` - Liczba bić w pokoleniu.
/// * `best_evaluations` - Najlepsza liczba bić dla każdego pokolenia.
/// * `avg_evaluations` - Średnia z sumy bić dla każdego pokolenia.
fn algo(params: &Params) -> (Vec<usize>, usize, Vec<usize>, Vec<f64>) {
    // Inicjalizacja populacji i ocen
    let mut population = generate_population(params.n, params.pop);
    let mut evaluations = evaluate(&population);

    // Inicjalizacja wektorów do przechowywania najlepszej oceny za każde pokolenie oraz średniej oceny za każde pokolenie
    let mut best_evaluation_per_gen = Vec::with_capacity(params.gen_max);
    let mut avg_evaluation_per_gen = Vec::with_capacity(params.gen_max);

    let mut gen = 0;
    let mut best_individual = evaluations
        .iter()
        .enumerate()
        .min_by_key(|(_, attacks)| *attacks)
        .map(|(i, _)| i)
        .unwrap_or_default();

    // Zapisanie początkowej najlepszej i średniej oceny
    best_evaluation_per_gen.push(evaluations[best_individual]);
    avg_evaluation_per_gen.push(evaluations.iter().sum::<usize>() as f64 / params.pop as f64);

    // Uruchomienie algorytmu aż do osiągnięcia maksymalnej liczby pokoleń lub znalezienia rozwiązania
    while gen < params.gen_max && evaluations[best_individual] > 0 {
        // Wykonaj selekcję, krzyżowanie i mutację aby stworzyć nową populację
        let mut new_population = selection(&population, &evaluations, params.tournament_size);
        crossover(&mut new_population, params.p_c);
        mutation(&mut new_population, params.p_m);

        // Oceń nową populację
        evaluations = evaluate(&new_population);

        // Zaktualizuj najlepszego osobnika
        best_individual = evaluations
            .iter()
            .enumerate()
            .min_by_key(|(_, attacks)| *attacks)
            .map(|(i, _)| i)
            .unwrap_or_default();

        // Zaktualizuj populację
        population = new_population;

        // Zapisz najlepszą ocenę i średnią ocenę dla tego pokolenia
        best_evaluation_per_gen.push(evaluations[best_individual]);
        avg_evaluation_per_gen.push(evaluations.iter().sum::<usize>() as f64 / params.pop as f64);

        gen += 1;
    }

    // Zwróć końcowe wyniki
    (
        population[best_individual].clone(),
        evaluations[best_individual],
        best_evaluation_per_gen,
        avg_evaluation_per_gen,
    )
}

/// Generuje populację osobników.
///
/// # Argumenty
///
/// * `size` - Rozmiar każdego osobnika.
/// * `count` - Liczba osobników w populacji.
///
/// # Zwraca
///
/// * `population` - Populacja osobników.
fn generate_population(size: usize, count: usize) -> Vec<Vec<usize>> {
    // Tworzy pusty wektor do przechowywania populacji.
    let mut population = Vec::with_capacity(count);
    // Tworzy generator liczb losowych.
    let mut rng = rand::thread_rng();

    // Generuje każdego osobnika w populacji.
    for _ in 0..count {
        // Tworzy pusty wektor do przechowywania osobnika.
        let mut individual = Vec::with_capacity(size);
        // Generuje pozycję dla każdego hetmana w osobniku.
        for _ in 0..size {
            // Generuje losową pozycję i dodaje ją do osobnika.
            individual.push(rng.gen_range(0..size));
        }
        // Dodaje osobnika do populacji.
        population.push(individual);
    }

    // Zwraca populację.
    population
}

/// Ocenia sprawność poszczególnych osobników w populacji na podstawie liczby ataków królowych.
///
/// # Argumenty
///
/// * `population` - Wycinek wektorów reprezentujących populację osobników.
///
/// # Zwraca
///
/// * `evaluations` - Liczba ataków jako wektor wartości typu usize.
fn evaluate(population: &[Vec<usize>]) -> Vec<usize> {
    population
        .par_iter() // Parallel iterator over the population
        .map(|individual| {
            let n = individual.len();
            let mut attacks = 0;

            for i in 0..n - 1 {
                let mut j = i + 1;
                let mut x_diff: isize;
                let mut y_diff: isize;

                while j < n {
                    x_diff = (i as isize - j as isize).abs();
                    y_diff = (individual[i] as isize - individual[j] as isize).abs();

                    // Sprawdzenie ataków
                    if i == j || x_diff == y_diff || individual[i] == individual[j] {
                        attacks += 1;
                    }

                    j += 1;
                }
            }

            attacks
        })
        .collect()
}

/// Przeprowadza selekcję na populacji w oparciu o ich oceny.
///
/// # Argumenty
///
/// * `pop` - Populacja jako wektor wektorów typu usize.
/// * `evals` - Oceny jako wektor wartości typu usize.
///
/// # Zwraca
///
/// * `new_population` - Nowa populacja.
fn selection(pop: &[Vec<usize>], evals: &[usize], tournament_size: usize) -> Vec<Vec<usize>> {
    // Inicjalizacja generatora liczb losowych.
    let mut rng = rand::thread_rng();
    // Nowa populacja, która będzie zwrócona.
    let mut new_population = Vec::with_capacity(pop.len());

    // Pętla wykonuje się, dopóki nowa populacja nie będzie miała takiej samej wielkości jak oryginalna.
    while new_population.len() < pop.len() {
        // Wektor turnieju, który będzie przechowywał pary (osobnik, ocena).
        let mut tournament = Vec::with_capacity(tournament_size);

        // Losowanie osobników do turnieju.
        for _ in 0..tournament_size {
            let individual_index = rng.gen_range(0..pop.len());
            tournament.push((&pop[individual_index], evals[individual_index]));
        }

        // Sortowanie turnieju względem liczby ataków (oceny).
        tournament.sort_unstable_by_key(|&(_, attacks)| attacks);

        // Dodanie najlepszego osobnika z turnieju do nowej populacji.
        new_population.push(tournament[0].0.clone());
    }

    // Zwrócenie nowej populacji.
    new_population
}

/// Przeprowadza krzyżowanie na populacji.
///
/// # Argumenty
///
/// * `population` - Wektor wektorów reprezentujących populację.
/// * `p_c` - Prawdopodobieństwo krzyżowania.
fn crossover(population: &mut [Vec<usize>], p_c: f64) {
    let pop = population.len();

    // Iteracja przez pary w populacji
    for i in (0..pop - 2).step_by(2) {
        // Wykonaj krzyżowanie z prawdopodobieństwem p_c
        if random() <= p_c {
            let (a, b) = population.split_at_mut(i + 2);
            cross(&mut a[i], &mut b[0]);
        }
    }
}

/// Zamienia elementy między `slice_a` a `slice_b` w losowo wygenerowanych indeksach.
///
/// # Argumenty
///
/// * `slice_a` - Zmienny fragment elementów.
/// * `slice_b` - Zmienny fragment elementów.
fn cross(slice_a: &mut [usize], slice_b: &mut [usize]) {
    // Generuje generator liczb losowych
    let mut rng = rand::thread_rng();

    // Generuje losowe indeksy w zakresie fragmentów
    let mut index_a = rng.gen_range(0..slice_a.len());
    let mut index_b = rng.gen_range(0..slice_b.len());

    // Zamienia indeksy miejscami, jeśli index_a jest większy niż index_b
    if index_a > index_b {
        std::mem::swap(&mut index_a, &mut index_b);
    }

    // Zamienia elementy między wygenerowanymi indeksami
    for index in index_a..index_b {
        std::mem::swap(&mut slice_a[index], &mut slice_b[index]);
    }
}

/// Wykonuje mutację na populacji osobników.
///
/// # Argumenty
///
/// * `population` - Wskaźnik mutowalny do populacji osobników.
/// * `p_m` - Prawdopodobieństwo mutacji.
fn mutation(population: &mut [Vec<usize>], p_m: f64) {
    // Iteruje po każdym osobniku w populacji
    population.par_iter_mut().for_each(|individual| {
        // Sprawdza, czy na podstawie prawdopodobieństwa powinna nastąpić mutacja
        if random() <= p_m {
            // Przeprowadza mutację na osobniku
            mutate(individual);
        }
    });
}

/// Mutuje jednostkę poprzez losową zmianę jednego z jej elementów.
///
/// # Argumenty
///
/// * `individual` - Mutowalna referencja do wektora elementów typu `usize`, reprezentująca jednostkę.
fn mutate(individual: &mut Vec<usize>) {
    // Generuje losowy indeks w zakresie wektora reprezentującego jednostkę
    let index = rand::thread_rng().gen_range(0..individual.len());

    // Generuje losową nową pozycję w zakresie wektora reprezentującego jednostkę
    let new_position = rand::thread_rng().gen_range(0..individual.len());

    // Aktualizuje element na losowo wygenerowanym indeksie nową pozycją
    individual[index] = new_position;
}

/// Generuje losową liczbę zmiennoprzecinkową między 0 a 1.
fn random() -> f64 {
    rand::thread_rng().gen::<f64>()
}

/// Tworzy wykresy dla najlepszych i średnich ocen w kolejnych pokoleniach.
///
/// # Argumenty
///
/// * `best_evas` - Wycinek zawierający najlepsze oceny dla każdego pokolenia.
/// * `avg_evas` - Wycinek zawierający średnie oceny dla każdego pokolenia.
fn plots(params: &Params, best_evas: &[usize], avg_evas: &[f64]) {
    // Tworzy wektor pokoleń od 0 do długości `best_evas`.
    let generations: Vec<usize> = (0..best_evas.len()).collect();

    // Tworzy ślad liniowy dla najlepszych ocen.
    let best_trace = Scatter::new(generations.clone(), best_evas.to_vec())
        .name("Najlepsze oceny")
        .mode(Mode::Lines);

    // Tworzy ślad liniowy dla średnich ocen.
    let avg_trace = Scatter::new(generations, avg_evas.to_vec())
        .name("Średnie oceny")
        .mode(Mode::Lines);

    // Tworzy nowy wykres.
    let mut chart = Plot::new();
    // Dodaje ślad najlepszych ocen do wykresu.
    chart.add_trace(best_trace);
    // Dodaje ślad średnich ocen do wykresu.
    chart.add_trace(avg_trace);

    let title = format!("Problem N-hetmanów - Algorytm ewolucyjny\nParametry: n={}, pop={}, gen_max={}, p_c={}, p_m={}",
                        params.n, params.pop, params.gen_max, params.p_c, params.p_m);

    // Tworzy układ wykresu.
    let chart_layout = Layout::new()
        .title(title.as_str().into())
        .x_axis(Axis::new().title("Pokolenia".into()))
        .y_axis(Axis::new().title("Oceny".into()));

    // Ustawia układ wykresu.
    chart.set_layout(chart_layout);
    // Wyświetla wykres.
    chart.show();
}

fn experiment() {
    let aval_pop = [10, 100];
    let aval_p_m = [0.2, 0.5];

    aval_pop.par_iter().for_each(|&pop| {
        aval_p_m.par_iter().for_each(|&p_m| {
            let params: Params = init_params(8, pop, 1000, 0.7, p_m, pop/2);
            let (solution, evaluations, best_evaluations, avg_evaluations) = algo(&params);
            println!("Testing: {}", params);
            println!("Solution: {:?}\nEvaluation: {}", solution, evaluations);
            println!("Best evaluations: {:?}", best_evaluations);
            plots(&params, &best_evaluations, &avg_evaluations);
        });
    });
}
// Większa wartość p_m powoduje większą różnicę między najlepszą oceną a śrenią dla danej generacji. Przy mniejszych wartościach p_m obie te wartości są bliżej siebie.
// Większa liczba populacji powoduje dojście do lepszej oceny w mniejszą ilość generacji.

fn main() {
    experiment();
    // let params: Params = init_params(10, 60, 1000, 1., 0.6, 3);
    // let (solution, evaluations, best_evaluations, avg_evaluations) = algo(&params);

    // println!("{}", params);
    // println!("Solution: {:?}\nEvaluation: {}", solution, evaluations);
    // println!("Best evaluations: {:?}", best_evaluations);

    // plots(&params, &best_evaluations, &avg_evaluations);
}
