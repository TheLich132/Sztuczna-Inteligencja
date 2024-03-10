use super::*;

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn test_k_bins_discretizer() {
        let mut data = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
            vec![7.0, 8.0, 9.0],
        ];
        let n_bins = 3;

        k_bins_discretizer(&mut data, n_bins);

        assert_eq!(data, vec![
            vec![0.0, 0.0, 0.0],
            vec![1.0, 1.0, 1.0],
            vec![2.0, 2.0, 2.0],
        ]);
    }

    #[test]
    fn test_train_test_split() {
        let mut data = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
            vec![7.0, 8.0, 9.0],
            vec![10.0, 11.0, 12.0],
        ];
        let mut classes = vec![0, 1, 2, 3];
        let test_size = 0.25;

        let (train_classes, test_classes, train_data, test_data) = train_test_split(&mut data, &mut classes, test_size);

        // Assert that the lengths of the train and test vectors are correct
        assert_eq!(train_data.len() + test_data.len(), data.len());
        assert_eq!(train_classes.len() + test_classes.len(), classes.len());

        // Assert that the train and test vectors contain the correct data
        for i in 0..train_data.len() {
            assert_eq!(data.contains(&train_data[i]), true);
            assert_eq!(classes.contains(&train_classes[i]), true);
        }

        for i in 0..test_data.len() {
            assert_eq!(data.contains(&test_data[i]), true);
            assert_eq!(classes.contains(&test_classes[i]), true);
        }

        println!("Train classes: {:?}", train_classes);
        println!("Test classes: {:?}", test_classes);
        println!("Train data: {:?}", train_data);
        println!("Test data: {:?}", test_data);
    }
}