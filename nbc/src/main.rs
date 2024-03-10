use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use chrono::{DateTime, Local, Datelike, Timelike};
use core::fmt;

macro_rules! split_result {
    () => {(Vec<usize>, Vec<usize>, Vec<Vec<f64>>, Vec<Vec<f64>>)};
}

struct Stats {
    loop_count: usize,
    n_bins_size: usize,
    data_type: String,
    dataset: String,
    accuracy_vec_average: Vec<((bool,bool), f64)>,
}

impl Stats {
    fn new(data_type: String, dataset: String) -> Self {
        Stats {
            loop_count: 0,
            n_bins_size: 0,
            data_type,
            dataset,
            accuracy_vec_average: Vec::new(),
        }
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{} - {}", self.data_type, self.dataset)?;
        for (settings, avg) in self.accuracy_vec_average.iter() {
            writeln!(f, "laplace_smoothing: {}, logarithmic: {}, accuracy: {:.2}%", settings.0, settings.1, avg*100.)?;
        }
        Ok(())
    }
}

struct NBC {
    n_bins: usize,
    separate_classes: HashMap<usize, Vec<Vec<f64>>>,
    classes_probabilities: HashMap<usize, f64>,
    probabilities: HashMap<(usize, usize, usize), f64>,
}

impl NBC {
    fn new(n_bins: usize) -> Self {
        NBC {
            n_bins,
            separate_classes: HashMap::new(),
            classes_probabilities: HashMap::new(),
            probabilities: HashMap::new(),
        }
    }

    fn fit(&mut self, classes: &[usize], data: &[Vec<f64>], laplace_smoothing: bool) {
        for (&class, data_val) in classes.iter().zip(data.iter()) {
            self.separate_classes
                .entry(class)
                .or_default()
                .push(data_val.clone());
        }
    
        for (class, data) in self.separate_classes.iter_mut() {
            for n_bin in 0..self.n_bins {
                for i in 0..data[0].len() {
                    let count = data.iter().filter(|item| item[i] == n_bin as f64).count();
                    let probability = if !laplace_smoothing {
                        count as f64 / data.len() as f64
                    } else {
                        let k = 1;
                        (count + k) as f64 / (data.len() + k) as f64
                    };
                    self.probabilities.insert((*class, i, n_bin), probability);
                }
            }
        }
    
        let all_classes: usize = self.separate_classes.values().map(Vec::len).sum();
        for (class, data) in self.separate_classes.iter() {
            let class_probabilities = data.len() as f64 / all_classes as f64;
            self.classes_probabilities
                .insert(*class, class_probabilities);
        }
    }

    fn predict(&self, data: &[Vec<f64>], logarithmic: bool) -> Vec<usize> {
        let mut predicted_classes: Vec<usize> = Vec::new();

        for data in data.iter() {
            let mut predicted_class = 0;
            let mut max_probability = f64::MIN;
            for class in self.separate_classes.keys() {
                let mut probability: f64;
                if !logarithmic {
                    probability = *self.classes_probabilities.get(class).unwrap();
                } else {
                    probability = 0.0;
                }
                for (i, value) in data.iter().enumerate() {
                    if !logarithmic {
                        probability *= self
                            .probabilities
                            .get(&(*class, i, *value as usize))
                            .unwrap();
                    } else {
                        probability += self
                            .probabilities
                            .get(&(*class, i, *value as usize))
                            .unwrap()
                            .ln() + self.classes_probabilities.get(class).unwrap().ln();
                    }
                }
                if probability > max_probability {
                    max_probability = probability;
                    predicted_class = *class;
                }
            }
            predicted_classes.push(predicted_class);
        }
        predicted_classes
    }

    fn predict_proba(&self, data: &[Vec<f64>], logarithmic: bool) -> Vec<Vec<f64>> {
        let mut classes_prob: Vec<Vec<f64>> = Vec::new();

        for data in data.iter() {
            let mut data_classes_prob: Vec<f64> = Vec::new();
            for class in self.separate_classes.keys() {
                let mut probability: f64;
                if !logarithmic {
                    probability = *self.classes_probabilities.get(class).unwrap();
                } else {
                    probability = 0.0;
                }
                for (i, value) in data.iter().enumerate() {
                    if !logarithmic {
                        probability *= self
                            .probabilities
                            .get(&(*class, i, *value as usize))
                            .unwrap();
                    } else {
                        probability += self
                            .probabilities
                            .get(&(*class, i, *value as usize))
                            .unwrap()
                            .ln() + self.classes_probabilities.get(class).unwrap().ln();
                    }
                }
                data_classes_prob.push(probability);
            }
            classes_prob.push(data_classes_prob);
        }
        classes_prob
    }
}

fn load_from_txt(path: &str, delimiter: &str) -> (Vec<usize>, Vec<Vec<f64>>) {
    let mut classes: Vec<usize> = Vec::new();
    let mut data: Vec<Vec<f64>> = Vec::new();

    let file: File = File::open(path).unwrap();
    let reader: BufReader<File> = BufReader::new(file);

    for line in reader.lines().map(|l| l.unwrap()) {
        let mut temp: Vec<f64> = Vec::new();
        for (i, num) in line.split(delimiter).enumerate() {
            if i == 0 {
                classes.push(num.parse().unwrap());
            } else {
                temp.push(num.parse().unwrap());
            }
        }
        data.push(temp);
    }

    (classes, data)
}

fn k_bins_discretizer(data: &mut [Vec<f64>], n_bins: usize) {
    assert!(n_bins >= 2, "n_bins must be greater than or equal to 2");
    for column in 0..data[0].len() {
        let mut values: Vec<f64> = data.iter().map(|row| row[column]).collect();
        values.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

        let min_val = values.first().unwrap();
        let max_val = values.last().unwrap();
        let bin_width = (max_val - min_val) / ((n_bins - 1) as f64);

        for row in data.iter_mut() {
            let bin = ((row[column] - min_val) / bin_width).floor() as usize;
            row[column] = bin as f64;
        }
    }
}

fn train_test_split(
    data: &mut [Vec<f64>],
    classes: &mut [usize],
    test_size: f64,
) -> split_result!() {
    let n = data.len();
    let test_size = (test_size * n as f64) as usize;

    let mut rng = thread_rng();

    let mut data_classes: Vec<_> = data.iter().zip(classes.iter()).collect();
    data_classes.shuffle(&mut rng);

    let (shuffled_data, shuffled_classes): (Vec<Vec<f64>>, Vec<usize>) = data_classes
        .into_iter()
        .map(|(d, c)| (d.clone(), c))
        .unzip();

    let mut train_data: Vec<Vec<f64>> = Vec::new();
    let mut train_classes: Vec<usize> = Vec::new();
    let mut test_data: Vec<Vec<f64>> = Vec::new();
    let mut test_classes: Vec<usize> = Vec::new();

    for i in 0..n {
        if i < test_size {
            test_data.push(shuffled_data[i].clone());
            test_classes.push(shuffled_classes[i]);
        } else {
            train_data.push(shuffled_data[i].clone());
            train_classes.push(shuffled_classes[i]);
        }
    }

    (train_classes, test_classes, train_data, test_data)
}

fn print_data(data: &[Vec<f64>]) {
    if data.len() > 20 {
        // Print first 10 and last 10 rows
        for item in data.iter().take(10) {
            println!("{:?}", item);
        }
        println!("...");
        for item in data.iter().skip(data.len() - 10) {
            println!("{:?}", item);
        }
    } else {
        // Print all rows
        for row in data {
            println!("{:?}", row);
        }
    }
}

fn return_correct_matches(classes_predicted: &[usize], classes: &[usize]) -> usize {
    classes_predicted
        .iter()
        .zip(classes.iter())
        .filter(|(a, b)| a == b)
        .count()
}

fn save_to_file(test_stats: &Stats, train_stats: &Stats) {
    // Tworzenie katalogu do przechowywania wyników testów
    let directory: &str = "./benchmarks";
    fs::create_dir_all(directory).unwrap();

    // Nazwa pliku z godziną teraz
    let now: DateTime<Local> = Local::now();
    let file_name: String = format!("{}-{}_{}_{}_{}_{}_{}.txt",test_stats.dataset, now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());
    let path: String = format!("{}/{}", directory, file_name);

    // Tworzenie pliku na podanej ścieżce
    let mut file: File = File::create(path).unwrap();

    // Tworzenie tekstu do zapisania do pliku
    let text_start: String = format!("Średnia z {} prób. Podział dyskretyzacji na {} bins\n\n", test_stats.loop_count, test_stats.n_bins_size) ;
    file.write_all(text_start.as_bytes()).unwrap();
    let text: String = format!("{}\n\n{}", test_stats, train_stats);
    file.write_all(text.as_bytes()).unwrap();
}

fn benchmark(dataset: &str, print: bool, save: bool) {
    let mut test_stats = Stats::new("Test data".to_string(), dataset.to_string());
    let mut train_stats = Stats::new("Train data".to_string(), dataset.to_string());

    const N_BINS: usize = 5;
    const LOOP_COUNT: usize = 100;

    test_stats.n_bins_size = N_BINS;
    test_stats.loop_count = LOOP_COUNT;
    train_stats.n_bins_size = N_BINS;
    train_stats.loop_count = LOOP_COUNT;
    
    
    let (mut classes, mut data) = load_from_txt(dataset, ",");

    k_bins_discretizer(&mut data, N_BINS);

    let (train_classes, test_classes, train_data, test_data) =
        train_test_split(&mut data, &mut classes, 0.3);

    let mut laplace_smoothing: bool;
    let mut logarithmic: bool;

    for j in 0..4 {
        match j {
            0 => {laplace_smoothing = false; logarithmic = false;}
            1 => {laplace_smoothing = true; logarithmic = false;}
            2 => {laplace_smoothing = false; logarithmic = true;}
            3 => {laplace_smoothing = true; logarithmic = true;}
            _ => {laplace_smoothing = false; logarithmic = false;}
        }
        let mut test_accuracy_vec: Vec<f64> = vec![0.0; LOOP_COUNT];
        let mut train_accuracy_vec: Vec<f64> = vec![0.0; LOOP_COUNT];
        for (test_item, train_item) in test_accuracy_vec.iter_mut().take(LOOP_COUNT).zip(train_accuracy_vec.iter_mut().take(LOOP_COUNT)) {

            let mut nbc = NBC::new(N_BINS);
            nbc.fit(&train_classes, &train_data, laplace_smoothing);

            let classes_predicted: Vec<usize> = nbc.predict(&test_data, logarithmic);
            let correct_matches = return_correct_matches(&classes_predicted, &test_classes);
            let accuracy = correct_matches as f64 / classes_predicted.len() as f64;
            *test_item = accuracy;

            let mut nbc = NBC::new(N_BINS);
            nbc.fit(&train_classes, &train_data, laplace_smoothing);
            let classes_predicted: Vec<usize> = nbc.predict(&train_data, logarithmic);
            let correct_matches = return_correct_matches(&classes_predicted, &train_classes);
            let accuracy = correct_matches as f64 / classes_predicted.len() as f64;
            *train_item = accuracy;
        }

        let test_average_accuracy = test_accuracy_vec.iter().sum::<f64>() / test_accuracy_vec.len() as f64;
        let train_average_accuracy = train_accuracy_vec.iter().sum::<f64>() / train_accuracy_vec.len() as f64;

        test_stats.accuracy_vec_average.push(((laplace_smoothing, logarithmic), test_average_accuracy));
        train_stats.accuracy_vec_average.push(((laplace_smoothing, logarithmic), train_average_accuracy));
    }

    if print {
        println!("{test_stats}");
        println!("{train_stats}");
    }

    if save {
        save_to_file(&test_stats, &train_stats);
    }
}

fn main() {
    benchmark("wine.data", true, false);

    // benchmark("abalone.data", true, false);

    // let (mut classes, mut data) = load_from_txt("wine.data", ",");
    // let n_bins: usize = 5;
    // let laplace_smoothing: bool = false;
    // let logarithmic: bool = false;

    // k_bins_discretizer(&mut data, n_bins);

    // let (train_classes, test_classes, train_data, test_data) =
    //     train_test_split(&mut data, &mut classes, 0.3);

    // println!("********************************");
    // println!("*   Laplace smoothing = false  *");
    // println!("*   logharithmic      = false  *");
    // println!("********************************\n");

    // let mut nbc = NBC::new(n_bins);
    // nbc.fit(&train_classes, &train_data, laplace_smoothing);

    // // Test predict_proba i predict na danych uczonych
    // let classes_prob: Vec<Vec<f64>> = nbc.predict_proba(&train_data, logarithmic);

    // // println!("Predicted probabilities - train data:");
    // // print_data(&classes_prob);

    // let classes_predicted: Vec<usize> = nbc.predict(&train_data, logarithmic);

    // let correct_matches = return_correct_matches(&classes_predicted, &train_classes);

    // let accuracy = correct_matches as f64 / classes_predicted.len() as f64;
    // println!("Accuracy - train data: {:.2}%\n\n", accuracy * 100.0);

    // // Test predict_proba i predict na danych testowych
    // let classes_prob: Vec<Vec<f64>> = nbc.predict_proba(&test_data, logarithmic);

    // // println!("Predicted probabilities - test data:");
    // // print_data(&classes_prob);

    // let classes_predicted: Vec<usize> = nbc.predict(&test_data, logarithmic);

    // let correct_matches = return_correct_matches(&classes_predicted, &test_classes);

    // let accuracy = correct_matches as f64 / classes_predicted.len() as f64;
    // println!("Accuracy - test data: {:.2}%\n", accuracy * 100.0);

    // println!("********************************");
    // println!("*   Laplace smoothing = true   *");
    // println!("*   logharithmic      = false  *");
    // println!("********************************\n");

    // let laplace_smoothing: bool = true;
    // let mut nbc = NBC::new(n_bins);
    // nbc.fit(&train_classes, &train_data, laplace_smoothing);

    // // Test predict_proba i predict na danych uczonych
    // let classes_prob: Vec<Vec<f64>> = nbc.predict_proba(&train_data, logarithmic);

    // // println!("Predicted probabilities - train data:");
    // // print_data(&classes_prob);

    // let classes_predicted: Vec<usize> = nbc.predict(&train_data, logarithmic);

    // let correct_matches = return_correct_matches(&classes_predicted, &train_classes);

    // let accuracy = correct_matches as f64 / classes_predicted.len() as f64;
    // println!("Accuracy - train data: {:.2}%\n\n", accuracy * 100.0);

    // // Test predict_proba i predict na danych testowych
    // let classes_prob: Vec<Vec<f64>> = nbc.predict_proba(&test_data, logarithmic);

    // // println!("Predicted probabilities - test data:");
    // // print_data(&classes_prob);

    // let classes_predicted: Vec<usize> = nbc.predict(&test_data, logarithmic);

    // let correct_matches = return_correct_matches(&classes_predicted, &test_classes);

    // let accuracy = correct_matches as f64 / classes_predicted.len() as f64;
    // println!("Accuracy - test data: {:.2}%\n", accuracy * 100.0);

    // println!("********************************");
    // println!("*   Laplace smoothing = false  *");
    // println!("*   logharithmic      = true   *");
    // println!("********************************\n");

    // let logarithmic: bool = true;
    // let laplace_smoothing: bool = false;

    // let mut nbc = NBC::new(n_bins);
    // nbc.fit(&train_classes, &train_data, laplace_smoothing);

    // // Test predict_proba i predict na danych uczonych
    // let classes_prob: Vec<Vec<f64>> = nbc.predict_proba(&train_data, logarithmic);

    // // println!("Predicted probabilities - train data:");
    // // print_data(&classes_prob);

    // let classes_predicted: Vec<usize> = nbc.predict(&train_data, logarithmic);

    // let correct_matches = return_correct_matches(&classes_predicted, &train_classes);

    // let accuracy = correct_matches as f64 / classes_predicted.len() as f64;
    // println!("Accuracy - train data: {:.2}%\n\n", accuracy * 100.0);

    // // Test predict_proba i predict na danych testowych
    // let classes_prob: Vec<Vec<f64>> = nbc.predict_proba(&test_data, logarithmic);

    // // println!("Predicted probabilities - test data:");
    // // print_data(&classes_prob);

    // let classes_predicted: Vec<usize> = nbc.predict(&test_data, logarithmic);

    // let correct_matches = return_correct_matches(&classes_predicted, &test_classes);

    // let accuracy = correct_matches as f64 / classes_predicted.len() as f64;
    // println!("Accuracy - test data: {:.2}%\n", accuracy * 100.0);

    // println!("********************************");
    // println!("*   Laplace smoothing = true   *");
    // println!("*   logharithmic      = true   *");
    // println!("********************************\n");

    // let laplace_smoothing: bool = true;
    // let mut nbc = NBC::new(n_bins);
    // nbc.fit(&train_classes, &train_data, laplace_smoothing);

    // // Test predict_proba i predict na danych uczonych
    // let classes_prob: Vec<Vec<f64>> = nbc.predict_proba(&train_data, logarithmic);

    // // println!("Predicted probabilities - train data:");
    // // print_data(&classes_prob);

    // let classes_predicted: Vec<usize> = nbc.predict(&train_data, logarithmic);

    // let correct_matches = return_correct_matches(&classes_predicted, &train_classes);

    // let accuracy = correct_matches as f64 / classes_predicted.len() as f64;
    // println!("Accuracy - train data: {:.2}%\n\n", accuracy * 100.0);

    // // Test predict_proba i predict na danych testowych
    // let classes_prob: Vec<Vec<f64>> = nbc.predict_proba(&test_data, logarithmic);

    // // println!("Predicted probabilities - test data:");
    // // print_data(&classes_prob);

    // let classes_predicted: Vec<usize> = nbc.predict(&test_data, logarithmic);

    // let correct_matches = return_correct_matches(&classes_predicted, &test_classes);

    // let accuracy = correct_matches as f64 / classes_predicted.len() as f64;
    // println!("Accuracy - test data: {:.2}%\n", accuracy * 100.0);
}

#[cfg(test)]
mod tests;
