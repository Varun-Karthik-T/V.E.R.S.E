
use methods::{GUEST_CODE_FOR_ZK_PROOF_ELF, GUEST_CODE_FOR_ZK_PROOF_ID};
use ort::{
    session::{builder::GraphOptimizationLevel, Session},
    value::{DynValue, Tensor},
};
use risc0_zkvm::{default_prover, ExecutorEnv};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Sample { 
    features: Vec<f32>,
    expected: i64,
}

fn main() -> Result<()> {
    // Step 1: Run guest to get validation data
    let env = ExecutorEnv::builder().build()?;
    let prover = default_prover();
    let session = prover.prove(env, GUEST_CODE_FOR_ZK_PROOF_ELF)?;

    // Step 2: Decode validation data
    let validation_data: Vec<Sample> = session.receipt.journal.decode()?;
    println!("Received validation dataset from guest: {:?}", validation_data);

    // Step 3: Load ONNX model
    let mut model = Session::builder()?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .commit_from_file("iris_tree_model.onnx")?;

    let iris_classes = ["Iris-setosa", "Iris-versicolor", "Iris-virginica"];
    let mut correct = 0;

    // Step 4: Run inference for each sample
    for (i, sample) in validation_data.iter().enumerate() {
        let shape = [1usize, 4usize];
        let input_tensor = Tensor::from_array((shape, sample.features.clone().into_boxed_slice()))?;

        let mut outputs = model.run(ort::inputs!["float_input" => input_tensor])?;

        let label_value: DynValue = outputs.remove("output_label").unwrap();
        let prob_value: DynValue = outputs.remove("output_probability").unwrap();

        // Extract predicted class index
        let predicted_indices = label_value.try_extract_array::<i64>()?;
        let predicted_index = predicted_indices[0] as usize;

        // Class mapping
        let predicted_class = iris_classes
            .get(predicted_index)
            .unwrap_or(&"Unknown class");

        // Check accuracy
        if predicted_index == sample.expected as usize {
            correct += 1;
        }

        println!(
            "\nSample {}:\n  Features: {:?}\n  Expected: {} ({})\n  Predicted: {} ({})",
            i + 1,
            sample.features,
            sample.expected,
            iris_classes[sample.expected as usize],
            predicted_index,
            predicted_class
        );

        // ✅ Fixed API call here
        if let Ok(prob_map) = prob_value.try_extract_map::<i64, f32>() {
            println!("  Probabilities:");
            for (cls, prob) in prob_map.iter() {
                println!("    {:<15} ({}) → {:.4}", iris_classes[*cls as usize], cls, prob);
            }
        }
    }

    let accuracy = (correct as f32 / validation_data.len() as f32) * 100.0;
    println!("\n✅ Model accuracy on validation data: {:.2}%", accuracy);

    // Step 5: Verify proof authenticity
    session.receipt.verify(GUEST_CODE_FOR_ZK_PROOF_ID)?;
    println!("✅ Guest dataset verified via ZK proof");

    Ok(())
}

