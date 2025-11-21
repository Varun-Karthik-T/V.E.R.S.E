
#![no_main]
use risc0_zkvm::guest::env;
risc0_zkvm::guest::entry!(main);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Sample {
    features: Vec<f32>,
    expected: i64,
}

pub fn main() {
    // Validation dataset
    let validation_data = vec![
        Sample { features: vec![5.1, 3.5, 1.4, 0.2], expected: 0 },
        Sample { features: vec![4.9, 3.0, 1.4, 0.2], expected: 0 },
        Sample { features: vec![6.0, 2.2, 4.0, 1.0], expected: 1 },
        Sample { features: vec![5.9, 3.0, 5.1, 1.8], expected: 2 },
        Sample { features: vec![6.5, 3.0, 5.2, 2.0], expected: 2 },
    ];

    // Commit the validation dataset to the journal
    env::commit(&validation_data);
}

