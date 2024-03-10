#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::upper_case_acronyms)]

use csv::Writer;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Normal};
use std::f64::consts::PI;
use std::io::{self, Write};
use std::process::Command;

const RNG_SEED: u64 = 2137;
const TEST_RNG_SEED: u64 = 0xB00B135;

struct MLP {
    N: usize,                              // Liczba neuronów w warstawie ukrytej
    learning_steps: usize,                 // Maksymalna liczba kroków uczenia
    learning_rate: f64,                    // Wspolczynnik uczenia
    hidden_layer_weights: Vec<(f64, f64, f64)>, // Wagi warstwy ukrytej (V)
    output_layer_weights: Vec<f64>,        // Wagi warstwy wyjsciowej (W)
    output_layer_bias: f64,                 // Waga biasu warstwy ukrytej
}

impl MLP {

    /// Tworzy nowy model MLP z określonymi parametrami.
    ///
    /// # Argumenty
    ///
    /// * `N` - Liczba neuronów w warstwie ukrytej
    /// * `learning_steps` - Liczba kroków uczenia
    /// * `learning_rate` - Współczynnik uczenia
    ///
    /// # Zwraca
    ///
    /// Nowy model MLP z określonymi parametrami.
    fn new(N: usize, learning_steps: usize, learning_rate: f64) -> MLP {
        let mut model: MLP = MLP {
            N,
            learning_steps,
            learning_rate,
            output_layer_bias: 0.0,
            hidden_layer_weights: Vec::with_capacity(N),
            output_layer_weights: Vec::with_capacity(N),
        };

        let normal = Normal::new(0.0, 0.1).unwrap();
        model.output_layer_bias = normal.sample(&mut rand::thread_rng());
        for _ in 0..N {
            // Generowanie wag dla pojedynczego neuronu w warstwie ukrytej.
            let hidden_weight = (
                normal.sample(&mut rand::thread_rng()),
                normal.sample(&mut rand::thread_rng()),
                normal.sample(&mut rand::thread_rng()),
            );
            // Dodanie wygenerowanych wag do listy wag dla warstwy ukrytej.
            model.hidden_layer_weights.push(hidden_weight);
            // Generowanie i dodanie wagi dla neuronu w warstwie wyjściowej.
            model.output_layer_weights.push(normal.sample(&mut rand::thread_rng()));
        }

        model
    }

    /// Propagacja w przód dla podanego wejścia i zwrócenie wyniku.
    ///
    /// # Argumenty
    ///
    /// * `input` - Wartości wejściowe jako krotka `(f64, f64)`.
    /// * `index` - Indeks wag warstwy ukrytej do użycia.
    ///
    /// # Zwraca
    ///
    /// Krotka zawierająca wartość funkcji sigmoidalnej oraz końcową wartość wyniku.
    fn forward_propagation(&self, input: &(f64, f64)) -> (Vec<f64>, f64) {
        let mut y: f64 = self.output_layer_bias;
        let mut sigmoids_si: Vec<f64> = Vec::with_capacity(self.N);
        for n in 0..self.N {
            // Obliczanie wartości pośredniej s_i
            let s_i: f64 = (self.hidden_layer_weights[n].0 * input.0)
                + (self.hidden_layer_weights[n].1 * input.1)
                + self.hidden_layer_weights[n].2;

            // Zastosowanie funkcji sigmoidalnej do wartości pośredniej s_i
            sigmoids_si.push(self.sigmoid(s_i));

            // Obliczanie wartości y
            y += self.output_layer_weights[n] * sigmoids_si[n];
        }

        // Zwracanie wartości sigmoidalnej i końcowej wartości wyniku
        (sigmoids_si, y)
    }

    /// Oblicza funkcję sigmoidalną dla danego wejścia.
    ///
    /// # Argumenty
    ///
    /// * `x` - Wartość wejściowa.
    ///
    /// # Zwraca
    ///
    /// Wartość sigmoidalna dla danego wejścia.
    fn sigmoid(&self, x: f64) -> f64 {
        1.0 / (1.0 + f64::exp(-x))
    }

    /// Dopasowuje model sieci neuronowej do danych treningowych.
    ///
    /// # Argumenty
    ///
    /// * `data` - Wycinek krotek reprezentujących dane wejściowe.
    /// * `training_data` - Wycinek wartości docelowych dla danych treningowych.
    fn fit(&mut self, data: &[(f64, f64)], training_data: &[f64]) {
        // Proces uczenia
        for epoch in 0..self.learning_steps {
            print!("Epoch {}, ", epoch);
            io::stdout().flush().unwrap();
            let index = rand::thread_rng().gen_range(0..data.len());
            for n in 0..self.N {
                let (activation_values, y) = self.forward_propagation(&data[index]);
                // Aktualizacja wag warstwy ukrytej
                self.hidden_layer_weights[n].0 -= self.learning_rate
                    * (y - training_data[index])
                    * self.output_layer_weights[n]
                    * activation_values[n]
                    * (1.0 - activation_values[n])
                    * data[index].0;
                self.hidden_layer_weights[n].1 -= self.learning_rate
                    * (y - training_data[index])
                    * self.output_layer_weights[n]
                    * activation_values[n]
                    * (1.0 - activation_values[n])
                    * data[index].1;
                // Aktualizacja wag warstwy wyjściowej
                self.output_layer_weights[n] -=
                    self.learning_rate * (y - training_data[index]) * activation_values[n];
            }
        }
    }

    /// Przewiduje wynik dla danego zestawu punktów danych.
    ///
    /// # Argumenty
    ///
    /// * `data` - Wektor krotek reprezentujących punkty danych wejściowych.
    ///            Każda krotka zawiera dwie wartości typu `f64`.
    ///
    /// # Zwraca
    ///
    /// Wektor wartości `f64` reprezentujących przewidywany wynik dla każdego punktu danych.
    fn predict(&self, data: &[(f64, f64)]) -> Vec<f64> {
        // Utwórz pusty wektor do przechowywania przewidywanych wartości wyjściowych
        let mut output: Vec<f64> = Vec::new();

        // Iteruj po każdym punkcie danych i wykonaj propagację w przód
        for sample in data.iter() {
            // Wywołaj metodę `forward_propagation` i dodaj drugi element zwróconej krotki do `output`
            let (_, y) = self.forward_propagation(sample);
            output.push(y);
        }

        // Zwróć wektor zawierający przewidywane wartości wyjściowe
        output
    }
}

/// Generuje wektor losowych punktów danych w określonym zakresie.
///
/// # Argumenty
///
/// * `size` - Liczba punktów danych do wygenerowania.
/// * `min` - Minimalna wartość współrzędnych x i y.
/// * `max` - Maksymalna wartość współrzędnych x i y.
/// * `seed` - Ziarno dla generatora liczb pseudolosowych.
///
/// # Zwraca
///
/// Wektor krotek reprezentujących wygenerowane punkty danych.
fn generate_data(size: usize, min: f64, max: f64, seed: u64) -> Vec<(f64, f64)> {
    // Inicjalizacja generatora liczb pseudolosowych z podanym ziarnem.
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    // Inicjalizacja wektora na dane wyjściowe.
    let mut data: Vec<(f64, f64)> = Vec::new();

    // Pętla generująca losowe punkty danych.
    for _ in 0..size {
        // Generowanie losowej wartości dla x w zakresie od min do max.
        let x: f64 = rng.gen_range(min..=max);
        // Generowanie losowej wartości dla y w zakresie od min do max.
        let y: f64 = rng.gen_range(min..=max);
        // Dodawanie wygenerowanego punktu do wektora danych.
        data.push((x, y));
    }

    // Zwracanie wektora zawierającego wygenerowane punkty danych.
    data
}

/// Generuje dane treningowe na podstawie danych wejściowych.
///
/// # Argumenty
///
/// * `data` - Dane wejściowe jako wektor krotek typu `(f64, f64)`.
///
/// # Zwraca
///
/// Wektor typu `Vec<f64>` zawierający wygenerowane dane treningowe.
fn generate_training_data(data: &Vec<(f64, f64)>) -> Vec<f64> {
    let mut to_return: Vec<f64> = Vec::new();
    for (x1, x2) in data {
        // Oblicza wartość i dodaje ją do wektora to_return
        to_return.push(f64::cos(x1 * x2) * f64::cos(2.0 * x1));
    }
    to_return
}

/// Uruchamia benchmark dla różnych parametrów i zapisuje wyniki do pliku CSV.
fn benchmark() {
    // Definiuje minimalne i maksymalne wartości dla parametrów
    const MIN_N: usize = 2;
    const MAX_N: usize = 100;
    const MIN_STEP: usize = 1000;
    const MAX_STEP: usize = 10000;
    const MIN_LEARNING_RATE: f64 = 0.001;
    const MAX_LEARNING_RATE: f64 = 0.1;

    // Inicjuje wektor do przechowywania względnych błędów aproksymacji
    let mut relative_approximation_error: Vec<(usize, usize, f64, f64)> = Vec::new();

    // Generuje dane wejściowe i dane do trenowania
    let data: Vec<(f64, f64)> = generate_data(1000, 0.0, PI, RNG_SEED);
    let training_data: Vec<f64> = generate_training_data(&data);

    // Iteruje po wartościach parametrów
    for N in (MIN_N..=MAX_N).step_by(10) {
        for step in (MIN_STEP..=MAX_STEP).step_by(1000) {
            let mut learning_rate = MIN_LEARNING_RATE;
            while learning_rate <= MAX_LEARNING_RATE {
                // Wyświetla bieżące wartości parametrów
                println!("\n{} {} {}", N, step, learning_rate);

                // Tworzy nową sieć MLP z bieżącymi wartościami parametrów
                let mut mlp = MLP::new(N, step, learning_rate);
                mlp.fit(&data, &training_data);

                // Dokonuje predykcji przy użyciu sieci MLP
                let prediction = mlp.predict(&data);

                let test_data: Vec<(f64, f64)> = generate_data(10_000, 0.0, std::f64::consts::PI, RNG_SEED);
                let test_targets: Vec<f64> = generate_training_data(&test_data);
                let test_predictions = mlp.predict(&test_data);

                let mse: f64 = test_targets.iter()
                    .zip(test_predictions.iter())
                    .map(|(target, prediction)| (target - prediction).powi(2))
                    .sum::<f64>() / test_targets.len() as f64;

                println!("Średni błąd kwadratowy (MSE): {}", mse);

                relative_approximation_error.push((N, step, learning_rate, mse));

                // // Oblicza względny błąd aproksymacji
                // let mut sum: f64 = 0.0;
                // for (x, y) in training_data.iter().zip(prediction.iter()) {
                //     sum += (x - y).abs();
                // }
                // let relative_error: f64 = sum / training_data.len() as f64 * 100.0;
                // relative_approximation_error.push((N, step, learning_rate, relative_error));

                // Zwiększa współczynnik uczenia dziesięciokrotnie
                learning_rate *= 10.0;
            }
        }
    }

    // Zapisuje względne błędy aproksymacji do pliku CSV
    let mut wtr = Writer::from_path("benchmark.csv").unwrap();
    for (N, step, learning_rate, relative_error) in relative_approximation_error {
        wtr.write_record(&[
            N.to_string(),
            step.to_string(),
            learning_rate.to_string(),
            relative_error.to_string(),
        ])
        .unwrap();
    }
    wtr.flush().unwrap();
}

/// Generuje pliki CSV do wizualizacji danych i predykcji
///
/// # Argumenty
///
/// * `N` - Liczba neuronów w warstwie ukrytej
/// * `learning_steps` - Liczba kroków uczenia
/// * `learning_rate` - Współczynnik uczenia
fn generate_plot_csv(N: usize, learning_steps: usize, learning_rate: f64) {
    // Generowanie danych i danych treningowych
    let data: Vec<(f64, f64)> = generate_data(1000, 0.0, PI, RNG_SEED);
    let training_data: Vec<f64> = generate_training_data(&data);

    // Stworzenie sieci MLP z podanymi parametrami
    let mut mlp = MLP::new(N, learning_steps, learning_rate);

    // Dopasowanie sieci MLP do danych
    mlp.fit(&data, &training_data);

    let test_data: Vec<(f64, f64)> = generate_data(1000, 0.0, PI, RNG_SEED);
    let test_training_data: Vec<f64> = generate_training_data(&test_data);

    // Wykonanie predykcji przy użyciu wytrenowanej sieci MLP
    let prediction = mlp.predict(&data);

    // Eksport danych do pliku CSV
    let mut wtr = Writer::from_path("data.csv").unwrap();
    for (x, y) in data.iter() {
        wtr.write_record(&[x.to_string(), y.to_string()]).unwrap();
    }
    wtr.flush().unwrap();

    // Eksport danych treningowych do pliku CSV
    wtr = Writer::from_path("training_data.csv").unwrap();
    for y in training_data.iter() {
        wtr.write_record(&[y.to_string()]).unwrap();
    }
    wtr.flush().unwrap();

    // Eksport predykcji do pliku CSV
    wtr = Writer::from_path("prediction.csv").unwrap();
    for y in prediction.iter() {
        wtr.write_record(&[y.to_string()]).unwrap();
    }
    wtr.flush().unwrap();

    // Eksport predykcji do pliku CSV
    wtr = Writer::from_path("test_training_data.csv").unwrap();
    for y in test_training_data.iter() {
        wtr.write_record(&[y.to_string()]).unwrap();
    }
    wtr.flush().unwrap();

    // Eksport wag warstwy ukrytej do pliku CSV
    wtr = Writer::from_path("test_data.csv").unwrap();
    for (x1, x2) in test_data.iter() {
        wtr.write_record(&[x1.to_string(), x2.to_string()]).unwrap();
    }
    wtr.flush().unwrap();

    wtr = Writer::from_path("fit_X.csv").unwrap();
    for (x1, x2, x3) in mlp.hidden_layer_weights.iter() {
        wtr.write_record(&[x1.to_string(), x2.to_string(), x3.to_string()]).unwrap();
    }
    wtr.flush().unwrap();
    
    // Eksport wag warstwy wyjściowej do pliku CSV
    wtr = Writer::from_path("fit_Z.csv").unwrap();
    for z in mlp.output_layer_weights.iter() {
        wtr.write_record(&[z.to_string()]).unwrap();
    }
    wtr.flush().unwrap();

    println!("\nCSV exported");

    // Wykonanie skryptu Python do wizualizacji danych
    let plot_script = Command::new("python3")
        .arg("plots.py")
        .output()
        .expect("failed to execute process");
}

fn main() {
    // benchmark();

    generate_plot_csv(20, 1000000, 0.1);
}
