
// // // // use risc0_zkvm::{default_prover, ExecutorEnv};
// // // // use risc0_zkvm::serde::from_slice;
// // // // use serde_json;
// // // // use std::fs;
// // // // use std::io;

// // // // fn main() {
// // // //     println!("Enter path to guest ELF file:");
// // // //     let mut path = String::new();
// // // //     io::stdin().read_line(&mut path).expect("Failed to read input");
// // // //     let path = path.trim();
// // // //     let elf_bytes = fs::read(path).expect("Failed to read ELF file");
// // // //     let guest_elf: &[u8] = &elf_bytes;

// // // //     tracing_subscriber::fmt()
// // // //         .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
// // // //         .init();

// // // //     println!("Select model type (1=linear, 2=multiple, 3=polynomial, 4=logistic):");
// // // //     let mut buffer = String::new();
// // // //     io::stdin().read_line(&mut buffer).expect("Failed to read model type");
// // // //     let model_type: u32 = buffer.trim().parse().expect("Please enter a valid number");

// // // //     println!("Enter weights (comma-separated, e.g., 1.0,2.0,3.0):");
// // // //     buffer.clear();
// // // //     io::stdin().read_line(&mut buffer).expect("Failed to read weights");
// // // //     let weights: Vec<f32> = buffer
// // // //         .trim()
// // // //         .split(',')
// // // //         .map(|s| s.parse::<f32>().expect("Invalid weight"))
// // // //         .collect();

// // // //     println!("Enter bias (b):");
// // // //     buffer.clear();
// // // //     io::stdin().read_line(&mut buffer).expect("Failed to read bias");
// // // //     let b: f32 = buffer.trim().parse().expect("Please enter a valid number");

// // // //     let env = ExecutorEnv::builder()
// // // //         .write(&model_type)
// // // //         .unwrap()
// // // //         .write(&weights)
// // // //         .unwrap()
// // // //         .write(&b)
// // // //         .unwrap()
// // // //         .build()
// // // //         .unwrap();

// // // //     let prover = default_prover();
// // // //     let prove_info = prover.prove(env, &guest_elf).unwrap();
// // // //     let receipt = prove_info.receipt;

// // // //     let output: Vec<(f32, f32)> = from_slice(receipt.journal.bytes.as_slice()).unwrap();

// // // //     println!("\n======= Inference Results =======");
// // // //     for (i, (y_pred, y_true)) in output.iter().enumerate() {
// // // //         println!("Sample {} => Predicted: {}, True: {}", i + 1, y_pred, y_true);
// // // //     }
// // // //     println!("================================\n");

// // // //     let proof_json = serde_json::to_string_pretty(&receipt).expect("Failed to serialize receipt");
// // // //     fs::write("proof.json", &proof_json).expect("Failed to write proof file");
// // // //     println!("✅ Proof saved to proof.json");
// // // // }

// // // use risc0_zkvm::{default_prover, ExecutorEnv};
// // // use risc0_zkvm::serde::from_slice;
// // // use serde_json;
// // // use std::fs;
// // // use std::io;
// // // use std::time::Instant;

// // // fn main() {
// // //     println!("Enter path to guest ELF file:");
// // //     let mut path = String::new();
// // //     io::stdin().read_line(&mut path).unwrap();
// // //     let guest_elf = fs::read(path.trim()).unwrap();

// // //     println!("Optimized? (0 = float, 1 = fixed):");
// // //     let mut buffer = String::new();
// // //     io::stdin().read_line(&mut buffer).unwrap();
// // //     let use_opt_flag: u32 = buffer.trim().parse().unwrap();

// // //     println!("Select model (1=linear, 2=multiple, 3=poly, 4=logistic):");
// // //     buffer.clear();
// // //     io::stdin().read_line(&mut buffer).unwrap();
// // //     let model_type: u32 = buffer.trim().parse().unwrap();

// // //     println!("Enter weights (comma-separated):");
// // //     buffer.clear();
// // //     io::stdin().read_line(&mut buffer).unwrap();
// // //     let weights: Vec<f32> = buffer.trim().split(',')
// // //         .map(|s| s.parse().unwrap())
// // //         .collect();

// // //     println!("Enter bias (b):");
// // //     buffer.clear();
// // //     io::stdin().read_line(&mut buffer).unwrap();
// // //     let b: f32 = buffer.trim().parse().unwrap();

// // //     let env = ExecutorEnv::builder()
// // //         .write(&use_opt_flag).unwrap()
// // //         .write(&model_type).unwrap()
// // //         .write(&weights).unwrap()
// // //         .write(&b).unwrap()
// // //         .build()
// // //         .unwrap();

// // //     let prover = default_prover();

// // //     println!("Running proof…");
// // //     let start = Instant::now();
// // //     let prove_info = prover.prove(env, &guest_elf).unwrap();
// // //     let prove_time = start.elapsed();

// // //     let receipt = prove_info.receipt;
// // //     let cycles = prove_info.stats.total_cycles;

// // //     let output: Vec<(f32, f32)> = from_slice(receipt.journal.bytes.as_slice()).unwrap();

// // //     println!("\n=== Results (showing first 5) ===");
// // //     for i in 0..5.min(output.len()) {
// // //         println!("{}: pred={}, true={}", i, output[i].0, output[i].1);
// // //     }

// // //     println!("\n=== Benchmark ===");
// // //     println!("Dataset size inside guest ✅");
// // //     println!("Prove time: {:?}", prove_time);
// // //     println!("Cycle count: {}", prove_info.stats.total_cycles);
// // //     println!("Journal size: {} bytes", receipt.journal.bytes.len());
// // //     let proof_json = serde_json::to_string(&receipt).unwrap();
// // //     println!("Proof size: {} bytes", proof_json.len());

// // //     fs::write("proof.json", &proof_json).unwrap();
// // //     println!("✅ Proof saved to proof.json");
// // // }

// workssssssssssssssssssss









// use risc0_zkvm::{default_prover, ExecutorEnv};
// use risc0_zkvm::serde::from_slice;
// use serde_json;
// use std::fs;
// use std::io;
// use std::time::Instant;

// fn main() {
//     println!("Enter path to guest ELF file:");
//     let mut path = String::new();
//     io::stdin().read_line(&mut path).unwrap();
//     let guest_elf_path = path.trim();
//     let guest_elf = fs::read(guest_elf_path).expect("Failed to read guest ELF");

//     println!("Optimized? (0 = float, 1 = fixed):");
//     let mut buffer = String::new();
//     io::stdin().read_line(&mut buffer).unwrap();
//     let use_opt_flag: u32 = buffer.trim().parse().expect("Enter 0 or 1");

//     println!("Select model (1=linear, 2=multiple, 3=poly, 4=logistic):");
//     buffer.clear();
//     io::stdin().read_line(&mut buffer).unwrap();
//     let model_type: u32 = buffer.trim().parse().expect("Enter 1..4");

//     println!("Enter weights (comma-separated):");
//     buffer.clear();
//     io::stdin().read_line(&mut buffer).unwrap();
//     let weights: Vec<f32> = buffer.trim()
//         .split(',')
//         .filter(|s| !s.trim().is_empty())
//         .map(|s| s.trim().parse::<f32>().expect("Invalid weight"))
//         .collect();

//     // Validate weight counts
//     match model_type {
//         1 => if weights.len() != 1 { panic!("Model 1 expects 1 weight"); },
//         2 | 4 => if weights.len() != 3 { panic!("Model 2/4 expect 3 weights"); },
//         3 => if weights.is_empty() { panic!("Polynomial expects >=1 coeff"); },
//         _ => panic!("Unknown model"),
//     }

//     println!("Enter bias (b) (use dot, e.g. 2.0):");
//     buffer.clear();
//     io::stdin().read_line(&mut buffer).unwrap();
//     let b: f32 = buffer.trim().parse().expect("Invalid bias");

    // let env = ExecutorEnv::builder()
    //     .write(&use_opt_flag).unwrap()
//         .write(&model_type).unwrap()
//         .write(&weights).unwrap()
//         .write(&b).unwrap()
//         .build().unwrap();

//     let prover = default_prover();

//     println!("Running proof…");
//     let start = Instant::now();
//     let prove_info = prover.prove(env, &guest_elf).expect("Prove failed");
//     let elapsed = start.elapsed();

//     let receipt = prove_info.receipt;
//     let output: Vec<(f32, f32)> = from_slice(receipt.journal.bytes.as_slice()).expect("Failed decode journal");

//     println!("\n=== Results (first 5) ===");
//     for (i, (p, t)) in output.iter().enumerate().take(5) {
//         println!("{}: pred={:.6}, true={:.6}", i, p, t);
//     }

//     println!("\n=== Benchmark ===");
//     // println!("Dataset size inside guest (edit guest DATASET_SIZE): {}", DATASET_SIZE);
//     println!("Prove time: {:?}", elapsed);
//     println!("Cycle count: {}", prove_info.stats.total_cycles);
//     println!("Journal size: {} bytes", receipt.journal.bytes.len());

//     let proof_json = serde_json::to_string(&receipt).expect("serialize failed");
//     println!("Proof size: {} bytes", proof_json.len());
//     fs::write("proof.json", &proof_json).expect("failed write proof");
//     println!("✅ Proof saved to proof.json");
// }










use risc0_zkvm::{default_prover, ExecutorEnv};
use risc0_zkvm::serde::from_slice;
use serde_json;
use std::fs;
use std::io;
use std::time::Instant;

fn main() {
    println!("Enter path to guest ELF file:");
    let mut path = String::new();
    io::stdin().read_line(&mut path).unwrap();
    let guest_elf_path = path.trim();
    let guest_elf = fs::read(guest_elf_path).expect("Failed to read guest ELF");

    println!("Optimized? (0 = float, 1 = fixed):");
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    let use_opt_flag: u32 = buffer.trim().parse().expect("Enter 0 or 1");

    println!("Select model (1=linear, 2=multiple, 3=poly, 4=logistic, 5=decision tree):");
    buffer.clear();
    io::stdin().read_line(&mut buffer).unwrap();
    let model_type: u32 = buffer.trim().parse().expect("Enter 1..5");

    // Variables to be passed to guest
    let mut weights: Vec<f32> = Vec::new();
    let mut b: f32 = 0.0;
    let mut tree_path = String::new();
    let mut tree_json = String::new();

    if model_type != 5 {
        println!("Enter weights (comma-separated):");
        buffer.clear();
        io::stdin().read_line(&mut buffer).unwrap();
        weights = buffer
            .trim()
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().parse::<f32>().expect("Invalid weight"))
            .collect();

        match model_type {
            1 => if weights.len() != 1 { panic!("Model 1 expects 1 weight"); },
            2 | 4 => if weights.len() != 3 { panic!("Model 2/4 expect 3 weights"); },
            3 => if weights.is_empty() { panic!("Polynomial expects >=1 coeff"); },
            _ => panic!("Unknown model"),
        }

        println!("Enter bias (b) (use dot, e.g. 2.0):");
        buffer.clear();
        io::stdin().read_line(&mut buffer).unwrap();
        b = buffer.trim().parse().expect("Invalid bias");
    } else {
        println!("Enter path to Decision Tree JSON file (default: tree.json):");
        buffer.clear();
        io::stdin().read_line(&mut buffer).unwrap();
        tree_path = buffer.trim().to_string();
        if tree_path.is_empty() {
            tree_path = "tree.json".to_string();
        }

        println!("[host] Using tree JSON path: {}", tree_path);
        let read_start = Instant::now();
        tree_json = fs::read_to_string(&tree_path).expect("Failed to read tree.json");
        println!(
            "[host] Loaded JSON ({} bytes) in {:.2?}",
            tree_json.len(),
            read_start.elapsed()
        );
        let preview: String = tree_json.chars().take(120).collect();
        println!(
            "[host] JSON preview: {}{}",
            preview,
            if tree_json.len() > 120 { "..." } else { "" }
        );
    }

    println!("\n[host] Building zkVM executor environment...");
    let mut builder = ExecutorEnv::builder();
    builder.write(&use_opt_flag).unwrap();
    builder.write(&model_type).unwrap();

    if model_type == 5 {
        builder.write(&tree_path).unwrap();
        builder.write(&tree_json).unwrap();
    } else {
        builder.write(&weights).unwrap();
        builder.write(&b).unwrap();
    }

    let env = builder.build().unwrap();


    let prover = default_prover();
    println!("[host] Starting proof generation...");
    let start = Instant::now();
    let prove_info = prover.prove(env, &guest_elf).expect("Prove failed");
    let elapsed = start.elapsed();

    let receipt = prove_info.receipt;

    if model_type == 5 {
        println!("[host] Decoding journal to predictions...");
        let predictions: Vec<(Vec<f64>, u32)> = from_slice(receipt.journal.bytes.as_slice()).expect("Failed decode journal");
        println!("[host] Decoded {} predictions", predictions.len());

        println!("\nSample | PredClass | Prob    | Expected");
        println!("---------------------------------------");
        for (i, (probs, expected)) in predictions.iter().enumerate().take(5) {
            let (pred_idx, pred_p) = probs
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(idx, p)| (idx, *p))
                .unwrap_or((usize::MAX, f64::NAN));
            println!("{:<6} | {:<9} | {:<6.3} | {}", i, pred_idx, pred_p, expected);
        }
    } else {
        let output: Vec<(f32, f32)> = from_slice(receipt.journal.bytes.as_slice()).expect("Failed decode journal");
        println!("\n=== Results (first 5) ===");
        for (i, (p, t)) in output.iter().enumerate().take(5) {
            println!("{}: pred={:.6}, true={:.6}", i, p, t);
        }
    }

    println!("\n=== Benchmark ===");
    println!("Prove time: {:?}", elapsed);
    println!("Cycle count: {}", prove_info.stats.total_cycles);
    println!("Journal size: {} bytes", receipt.journal.bytes.len());

    let proof_json = serde_json::to_string(&receipt).expect("serialize failed");
    fs::write("proof.json", &proof_json).expect("failed write proof");
    println!("✅ Proof saved to proof.json");
    fs::write("combined_outputs.json", &proof_json).expect("Write failed");
    println!("📄 Combined outputs saved to combined_outputs.json");
}



// // use risc0_zkvm::{default_prover, ExecutorEnv};
// // use risc0_zkvm::serde::from_slice;
// // use serde_json;
// // use std::fs;
// // use std::io;
// // use std::time::Instant;
// // use rayon::prelude::*; // ✅ for parallelism

// // fn main() {
// //     println!("Enter path to guest ELF file:");
// //     let mut path = String::new();
// //     io::stdin().read_line(&mut path).unwrap();
// //     let guest_elf_path = path.trim();
// //     let guest_elf = fs::read(guest_elf_path).expect("Failed to read guest ELF");

// //     println!("Optimized? (0 = float, 1 = fixed):");
// //     let mut buffer = String::new();
// //     io::stdin().read_line(&mut buffer).unwrap();
// //     let use_opt_flag: u32 = buffer.trim().parse().expect("Enter 0 or 1");

// //     println!("Select model (1=linear, 2=multiple, 3=poly, 4=logistic):");
// //     buffer.clear();
// //     io::stdin().read_line(&mut buffer).unwrap();
// //     let model_type: u32 = buffer.trim().parse().expect("Enter 1..4");

// //     println!("Enter weights (comma-separated):");
// //     buffer.clear();
// //     io::stdin().read_line(&mut buffer).unwrap();
// //     let weights: Vec<f32> = buffer.trim()
// //         .split(',')
// //         .filter(|s| !s.trim().is_empty())
// //         .map(|s| s.trim().parse::<f32>().expect("Invalid weight"))
// //         .collect();

// //     // Validate weight counts
// //     match model_type {
// //         1 => if weights.len() != 1 { panic!("Model 1 expects 1 weight"); },
// //         2 | 4 => if weights.len() != 3 { panic!("Model 2/4 expect 3 weights"); },
// //         3 => if weights.is_empty() { panic!("Polynomial expects >=1 coeff"); },
// //         _ => panic!("Unknown model"),
// //     }

// //     println!("Enter bias (b) (use dot, e.g. 2.0):");
// //     buffer.clear();
// //     io::stdin().read_line(&mut buffer).unwrap();
// //     let b: f32 = buffer.trim().parse().expect("Invalid bias");

// //     println!("Enter number of batches to run in parallel:");
// //     buffer.clear();
// //     io::stdin().read_line(&mut buffer).unwrap();
// //     let num_batches: usize = buffer.trim().parse().expect("Invalid batch count");

// //     // Run parallel batches
// //     println!("Running {num_batches} proofs in parallel…");
// //     let start_all = Instant::now();

// //     let results: Vec<_> = (0..num_batches).into_par_iter().map(|batch_id| {
// //         let env = ExecutorEnv::builder()
// //             .write(&use_opt_flag).unwrap()
// //             .write(&model_type).unwrap()
// //             .write(&weights).unwrap()
// //             .write(&b).unwrap()
// //             .build().unwrap();

// //         let prover = default_prover();

// //         let start = Instant::now();
// //         let prove_info = prover.prove(env, &guest_elf).expect("Prove failed");
// //         let elapsed = start.elapsed();

// //         let receipt = prove_info.receipt;
// //         let output: Vec<(f32, f32)> =
// //             from_slice(receipt.journal.bytes.as_slice()).expect("Failed decode journal");

// //         println!(
// //             "✅ Batch {} done in {:?} ({} cycles)",
// //             batch_id,
// //             elapsed,
// //             prove_info.stats.total_cycles
// //         );

// //         (output, prove_info.stats.total_cycles, elapsed)
// //     }).collect();

// //     // Combine results
// //     let all_outputs: Vec<(f32, f32)> = results.iter()
// //         .flat_map(|(o, _, _)| o.clone())
// //         .collect();

// //     let total_cycles: u64 = results.iter().map(|(_, c, _)| *c).sum();
// //     let avg_time = results.iter().map(|(_, _, t)| t.as_secs_f64()).sum::<f64>() / num_batches as f64;

// //     let total_time = start_all.elapsed();

// //     println!("\n=== Parallel Benchmark Summary ===");
// //     println!("Total batches: {}", num_batches);
// //     println!("Combined outputs: {}", all_outputs.len());
// //     println!("Average per batch time: {:.3}s", avg_time);
// //     println!("Total wall time: {:.3}s", total_time.as_secs_f64());
// //     println!("Total cycles (sum of all batches): {}", total_cycles);

// //     // Save outputs
// //     let proof_json = serde_json::to_string(&all_outputs).expect("serialize failed");
// //     fs::write("combined_outputs.json", &proof_json).expect("failed write combined_outputs.json");
// //     println!("✅ Combined outputs saved to combined_outputs.json");
// // }


// use risc0_zkvm::{default_prover, ExecutorEnv, serde::from_slice};

// use rayon::prelude::*;
// use serde_json;
// use std::{
//     fs,
//     io::{self, Write},
//     time::Instant,
// };

// fn main() {
//     // === User Inputs ===
//     println!("Enter path to guest ELF file:");
//     let mut path = String::new();
//     io::stdin().read_line(&mut path).unwrap();
//     let guest_elf_path = path.trim();
//     let guest_elf_bytes = fs::read(guest_elf_path).expect("❌ Failed to read guest ELF file");

//     println!("Optimized? (0 = float, 1 = fixed):");
//     let mut buffer = String::new();
//     io::stdin().read_line(&mut buffer).unwrap();
//     let use_opt_flag: u32 = buffer.trim().parse().expect("Enter 0 or 1");

//     println!("Select model (1=linear, 2=multiple, 3=poly, 4=logistic):");
//     buffer.clear();
//     io::stdin().read_line(&mut buffer).unwrap();
//     let model_type: u32 = buffer.trim().parse().expect("Enter 1..4");

//     println!("Enter weights (comma-separated):");
//     buffer.clear();
//     io::stdin().read_line(&mut buffer).unwrap();
//     let weights: Vec<f32> = buffer
//         .trim()
//         .split(',')
//         .filter(|s| !s.trim().is_empty())
//         .map(|s| s.trim().parse::<f32>().expect("Invalid weight"))
//         .collect();

//     // Validate weight counts for model type
//     match model_type {
//         1 => if weights.len() != 1 { panic!("Model 1 expects 1 weight"); },
//         2 | 4 => if weights.len() != 3 { panic!("Model 2/4 expect 3 weights"); },
//         3 => if weights.is_empty() { panic!("Polynomial expects >=1 coefficient"); },
//         _ => panic!("Unknown model type"),
//     }

//     println!("Enter bias (b) (use dot, e.g. 2.0):");
//     buffer.clear();
//     io::stdin().read_line(&mut buffer).unwrap();
//     let b: f32 = buffer.trim().parse().expect("Invalid bias");

//     println!("Enter number of batches to run in parallel:");
//     buffer.clear();
//     io::stdin().read_line(&mut buffer).unwrap();
//     let num_batches: usize = buffer.trim().parse().expect("Invalid batch count");

//     // === Parallel Batch Execution ===
//     println!("🚀 Running {num_batches} proofs in parallel…");
//     let start_all = Instant::now();

//     let results: Vec<_> = (0..num_batches)
//         .into_par_iter()
//         .map(|batch_id| {
//             let env = ExecutorEnv::builder()
//                 .write(&use_opt_flag).unwrap()
//                 .write(&model_type).unwrap()
//                 .write(&weights).unwrap()
//                 .write(&b).unwrap()
//                 .build().unwrap();

//             let prover = default_prover();
//             let guest_copy = guest_elf_bytes.clone(); // thread-safe ELF copy

//             let start = Instant::now();
//             match prover.prove(env, &guest_copy) {
//                 Ok(prove_info) => {
//                     let elapsed = start.elapsed();
//                     let receipt = prove_info.receipt;
//                     let output: Vec<(f32, f32)> = from_slice(receipt.journal.bytes.as_slice())
//                         .unwrap_or_else(|_| {
//                             eprintln!("⚠️ Batch {batch_id}: Failed to decode output journal");
//                             vec![]
//                         });

//                     println!(
//                         "✅ Batch {} done in {:?} ({} cycles)",
//                         batch_id,
//                         elapsed,
//                         prove_info.stats.total_cycles
//                     );

//                     (Some(output), prove_info.stats.total_cycles, elapsed)
//                 }
//                 Err(e) => {
//                     eprintln!("❌ Batch {} failed: {:?}", batch_id, e);
//                     (None, 0, Instant::now().elapsed())
//                 }
//             }
//         })
//         .collect();

//     // === Combine All Results ===
//     let all_outputs: Vec<(f32, f32)> = results
//         .iter()
//         .filter_map(|(o, _, _)| o.as_ref())
//         .flat_map(|o| o.clone())
//         .collect();

//     let total_cycles: u64 = results.iter().map(|(_, c, _)| *c).sum();
//     let avg_time = results
//         .iter()
//         .map(|(_, _, t)| t.as_secs_f64())
//         .sum::<f64>()
//         / num_batches as f64;
//     let total_time = start_all.elapsed();

//     println!("\n=== 🧾 Parallel Benchmark Summary ===");
//     println!("Total batches: {}", num_batches);
//     println!("Combined outputs: {}", all_outputs.len());
//     println!("Average per batch time: {:.3}s", avg_time);
//     println!("Total wall time: {:.3}s", total_time.as_secs_f64());
//     println!("Total cycles (sum): {}", total_cycles);

//     // Save results
//     let proof_json = serde_json::to_string_pretty(&all_outputs).expect("Serialize failed");
//     fs::write("combined_outputs.json", &proof_json).expect("Write failed");
//     println!("📄 Combined outputs saved to combined_outputs.json");
// }

