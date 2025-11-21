#![no_main]
use risc0_zkvm::guest::env;

// pub fn run_onnx_inference(model_path: &str, input_data: &[f32]) -> Result<(), Box<dyn std::error::Error>> {
//     use ort::{session::{Session, builder::GraphOptimizationLevel}, value::{Tensor, DynValue, MapValueType}};
//     use std::collections::HashMap;

//     let mut session = Session::builder()?
//         .with_optimization_level(GraphOptimizationLevel::Level3)?
//         .with_intra_threads(4)?
//         .commit_from_file(model_path)?;

//     let shape = [1usize, input_data.len()];
//     let input_tensor = Tensor::from_array((shape, input_data.to_vec().into_boxed_slice()))?;

//     let mut outputs = session.run(ort::inputs!("float_input" => input_tensor))?;

//     let label_value: DynValue = outputs
//         .remove("output_label")
//         .expect("missing output_label");
//     let prob_value: DynValue = outputs
//         .remove("output_probability")
//         .expect("missing output_probability");

//     println!("Predicted label: {:?}", label_value.try_extract_array::<i64>()?);

//     let allocator = session.allocator();
//     let prob_sequence = prob_value.try_extract_sequence::<MapValueType<i64, f32>>(allocator)?;
//     println!("Predicted probabilities: {:?}", prob_sequence);

//     for (i, map_val) in prob_sequence.iter().enumerate() {
//         let prob_map: HashMap<i64, f32> = map_val.try_extract_map::<i64, f32>()?;
//         println!("--- Probability map {} ---", i + 1);
//         for (class, prob) in &prob_map {
//             println!("Class {} → Probability {:.3}", class, prob);
//         }
//         if let Some((best_class, best_prob)) = prob_map.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()) {
//             println!("Most likely class: {}, Probability: {:.3}", best_class, best_prob);
//         }
//     }

//     Ok(())
// }






extern crate alloc;

use alloc::{vec, vec::Vec, string::String};
use serde::{Deserialize, Serialize};

// ------------------ Fixed point configuration ------------------
const SCALE_BITS: i32 = 16;            // 2^16 scaling
const SCALE: i64 = 1 << SCALE_BITS;    // 65536

// ------------------ Fixed helpers ------------------
#[inline(always)]
fn f32_to_fixed(x: f32) -> i64 {
    ((x as f64) * (SCALE as f64)).round() as i64
}

#[inline(always)]
fn fixed_to_f32(x: i64) -> f32 {
    (x as f64 / SCALE as f64) as f32
}

#[inline(always)]
fn fixed_mul(a: i64, b: i64) -> i64 {
    let prod = (a as i128) * (b as i128);
    (prod >> SCALE_BITS) as i64
}

#[inline(always)]
fn clamp_fx(x: i64, lo: i64, hi: i64) -> i64 {
    if x < lo { lo } else if x > hi { hi } else { x }
}

// ------------------ Fixed-model math primitives ------------------
fn multiple_regression_fixed_accumulate(features_fx: &[i64], weights_fx: &[i64], b_fx: i64) -> i64 {
    let mut acc: i64 = 0;
    for (x_fx, w_fx) in features_fx.iter().zip(weights_fx.iter()) {
        acc += fixed_mul(*x_fx, *w_fx);
    }
    acc + b_fx
}

fn polynomial_fixed_horner(x_fx: i64, coeffs_fx: &[i64]) -> i64 {
    let mut acc: i64 = 0;
    for &c in coeffs_fx.iter().rev() {
        acc = fixed_mul(acc, x_fx) + c;
    }
    acc
}

// Cubic sigmoid approximation in fixed domain
fn sigmoid_fixed_approx(z_fx: i64) -> i64 {
    const A1_F: f32 = 0.1963;
    const A3_F: f32 = 0.004375;
    let a1_fx = f32_to_fixed(A1_F);
    let a3_fx = f32_to_fixed(A3_F);
    let half_fx = f32_to_fixed(0.5);

    let z2 = fixed_mul(z_fx, z_fx);
    let z3 = fixed_mul(z2, z_fx);

    let term1 = fixed_mul(a1_fx, z_fx);
    let term3 = fixed_mul(a3_fx, z3);

    let mut y_fx = half_fx + term1 - term3;
    y_fx = clamp_fx(y_fx, 0, SCALE);
    y_fx
}

// ------------------ Float models ------------------
fn linear_regression_f(x: f32, a: f32, b: f32) -> f32 { x * a + b }

fn multiple_regression_f(xs: &[f32], weights: &[f32], b: f32) -> f32 {
    xs.iter().zip(weights.iter()).map(|(x, w)| x * w).sum::<f32>() + b
}

fn polynomial_regression_f(x: f32, coeffs: &[f32]) -> f32 {
    let mut acc = 0.0_f32;
    for &c in coeffs.iter().rev() {
        acc = acc * x + c;
    }
    acc
}

fn logistic_regression_f(xs: &[f32], weights: &[f32], b: f32) -> f32 {
    let z = xs.iter().zip(weights.iter()).map(|(x,w)| x * w).sum::<f32>() + b;
    1.0 / (1.0 + (-z).exp())
}

// ------------------ Decision Tree Structures ------------------
#[derive(Debug, Serialize, Deserialize)]
pub struct TreeNode {
    pub id: usize,
    pub feature: Option<usize>,
    pub threshold: Option<f64>,
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub value: Vec<Vec<f64>>,
}

#[derive(Debug)]
pub struct Sample {
    pub features: Vec<f64>,
    pub expected: u32,
}

fn get_dataset_tree() -> Vec<Sample> {
    vec![
        Sample { features: vec![5.1, 3.5, 1.4, 0.2], expected: 0 },
        Sample { features: vec![4.9, 3.0, 1.4, 0.2], expected: 0 },
        Sample { features: vec![6.0, 2.2, 4.0, 1.0], expected: 1 },
        Sample { features: vec![5.9, 3.0, 5.1, 1.8], expected: 2 },
        Sample { features: vec![6.5, 3.0, 5.2, 2.0], expected: 2 },
    ]
}

fn build_id_index(nodes: &Vec<TreeNode>) -> Vec<usize> {
    let mut max_id = 0usize;
    for n in nodes.iter() { if n.id > max_id { max_id = n.id; } }
    let mut map = vec![usize::MAX; max_id + 1];
    for (idx, n) in nodes.iter().enumerate() { map[n.id] = idx; }
    map
}

fn traverse_tree(nodes: &Vec<TreeNode>, id_index: &Vec<usize>, x: &[f64]) -> Vec<f64> {
    let mut current_id: usize = 0;
    loop {
        if current_id >= id_index.len() { panic!("Unknown node id"); }
        let idx = id_index[current_id];
        if idx == usize::MAX { panic!("Unmapped node id"); }
        let node = &nodes[idx];

        if node.feature.is_none() {
            return node.value[0].clone();
        }

        let feat = node.feature.unwrap();
        let thr = node.threshold.unwrap();
        let xf = x[feat];

        if xf <= thr {
            current_id = node.left.expect("Missing left child");
        } else {
            current_id = node.right.expect("Missing right child");
        }
    }
}

// ------------------ Original dataset ------------------
fn get_dataset() -> Vec<(Vec<f32>, f32)> {
    vec![
        (vec![2.0, 2.0, 3.0], 14.0),
        (vec![2.0, 3.0, 4.0], 20.0),
        (vec![3.0, 4.0, 5.0], 26.0),
        (vec![4.0, 5.0, 6.0], 32.0),
    ]
}

// ------------------ Guest Entry ------------------
risc0_zkvm::guest::entry!(main);

fn main() {
    let use_opt_flag: u32 = env::read(); // 0 = float, 1 = fixed
    let model_type: u32 = env::read();   // 1..5

    // Decision tree (case 5) needs tree JSON instead of weights/bias
    if model_type == 5 {
        let _tree_path: String = env::read();
        let tree_json: String = env::read();
        let tree: Vec<TreeNode> = match serde_json::from_str(&tree_json) {
            Ok(t) => t,
            Err(_) => panic!("Failed to parse tree JSON in guest"),
        };
        let id_index = build_id_index(&tree);
        let dataset = get_dataset_tree();

        let mut predictions = Vec::new();
        for sample in dataset.iter() {
            let pred = traverse_tree(&tree, &id_index, &sample.features);
            predictions.push((pred, sample.expected));
        }

        env::commit(&predictions);
        return;
    }

    // Other models (1–4)
    let weights: Vec<f32> = env::read();
    let b: f32 = env::read();

    let use_opt = use_opt_flag != 0;
    let dataset = get_dataset();
    assert!(!dataset.is_empty(), "Dataset loaded is empty");

    if use_opt {
        let weights_fx: Vec<i64> = weights.iter().map(|&w| f32_to_fixed(w)).collect();
        let b_fx = f32_to_fixed(b);
        let mut out_fx: Vec<(i64, i64)> = Vec::with_capacity(dataset.len());

        for (features, y_true_f) in dataset.iter() {
            let features_fx: Vec<i64> = features.iter().map(|&x| f32_to_fixed(x)).collect();
            let y_pred_fx = match model_type {
                1 => fixed_mul(weights_fx[0], features_fx[0]) + b_fx,
                2 => multiple_regression_fixed_accumulate(&features_fx, &weights_fx, b_fx),
                3 => polynomial_fixed_horner(features_fx[0], &weights_fx),
                4 => {
                    let z_fx = multiple_regression_fixed_accumulate(&features_fx, &weights_fx, b_fx) - b_fx;
                    sigmoid_fixed_approx(z_fx)
                }
                _ => panic!("Unknown model type {}", model_type),
            };
            let y_true_fx = f32_to_fixed(*y_true_f);
            out_fx.push((y_pred_fx, y_true_fx));
        }

        let out_float: Vec<(f32, f32)> = out_fx.into_iter()
            .map(|(p_fx, t_fx)| (fixed_to_f32(p_fx), fixed_to_f32(t_fx)))
            .collect();
        env::commit(&out_float);
    } else {
        let mut out: Vec<(f32, f32)> = Vec::with_capacity(dataset.len());

        for (features, y_true) in dataset.iter() {
            let y_pred = match model_type {
                1 => linear_regression_f(features[0], weights[0], b),
                2 => multiple_regression_f(&features, &weights, b),
                3 => polynomial_regression_f(features[0], &weights),
                4 => logistic_regression_f(&features, &weights, b),
                _ => panic!("Unknown model type {}", model_type),
            };
            out.push((y_pred, *y_true));
        }

        env::commit(&out);
    }
}





// // ------------------ Fixed point configuration ------------------
// const SCALE_BITS: i32 = 16;            // 2^16 scaling
// const SCALE: i64 = 1 << SCALE_BITS;    // 65536

// // ------------------ Fixed helpers ------------------
// #[inline(always)]
// fn f32_to_fixed(x: f32) -> i64 {
//     ((x as f64) * (SCALE as f64)).round() as i64
// }

// #[inline(always)]
// fn fixed_to_f32(x: i64) -> f32 {
//     (x as f64 / SCALE as f64) as f32
// }

// #[inline(always)]
// fn fixed_mul(a: i64, b: i64) -> i64 {
//     // use i128 transient to keep precision, then shift right
//     let prod = (a as i128) * (b as i128);
//     (prod >> SCALE_BITS) as i64
// }

// #[inline(always)]
// fn clamp_fx(x: i64, lo: i64, hi: i64) -> i64 {
//     if x < lo { lo } else if x > hi { hi } else { x }
// }

// // ------------------ Fixed-model math primitives ------------------
// fn multiple_regression_fixed_accumulate(features_fx: &[i64], weights_fx: &[i64], b_fx: i64) -> i64 {
//     let mut acc: i64 = 0;
//     for (x_fx, w_fx) in features_fx.iter().zip(weights_fx.iter()) {
//         acc += fixed_mul(*x_fx, *w_fx);
//     }
//     acc + b_fx
// }

// // Horner in fixed domain for polynomial evaluation
// fn polynomial_fixed_horner(x_fx: i64, coeffs_fx: &[i64]) -> i64 {
//     let mut acc: i64 = 0;
//     for &c in coeffs_fx.iter().rev() {
//         acc = fixed_mul(acc, x_fx) + c;
//     }
//     acc
// }

// // Cubic sigmoid approximation in fixed:
// // sigmoid(z) ≈ 0.5 + a1*z - a3*z^3  with a1=0.1963, a3=0.004375
// fn sigmoid_fixed_approx(z_fx: i64) -> i64 {
//     const A1_F: f32 = 0.1963;
//     const A3_F: f32 = 0.004375;
//     let a1_fx = f32_to_fixed(A1_F);
//     let a3_fx = f32_to_fixed(A3_F);
//     let half_fx = f32_to_fixed(0.5);

//     let z2 = fixed_mul(z_fx, z_fx);
//     let z3 = fixed_mul(z2, z_fx);

//     let term1 = fixed_mul(a1_fx, z_fx);
//     let term3 = fixed_mul(a3_fx, z3);

//     let mut y_fx = half_fx + term1 - term3;
//     y_fx = clamp_fx(y_fx, 0, SCALE); // clamp between 0 and 1
//     y_fx
// }

// // ------------------ Float math (used in float-mode and for fallback) ------------------
// fn linear_regression_f(x: f32, a: f32, b: f32) -> f32 { x * a + b }

// fn multiple_regression_f(xs: &[f32], weights: &[f32], b: f32) -> f32 {
//     xs.iter().zip(weights.iter()).map(|(x, w)| x * w).sum::<f32>() + b
// }

// // Horner in float for polynomial (coeff[0] + coeff[1]*x + coeff[2]*x^2 ...)
// fn polynomial_regression_f(x: f32, coeffs: &[f32]) -> f32 {
//     let mut acc = 0.0_f32;
//     for &c in coeffs.iter().rev() {
//         acc = acc * x + c;
//     }
//     acc
// }

// fn logistic_regression_f(xs: &[f32], weights: &[f32], b: f32) -> f32 {
//     let z = xs.iter().zip(weights.iter()).map(|(x,w)| x * w).sum::<f32>() + b;
//     1.0 / (1.0 + (-z).exp())
// }

// // ------------------ Static dataset ------------------
// // Returns 4 samples with 3 features each and a scalar target.
// fn get_dataset() -> Vec<(Vec<f32>, f32)> {
//     vec![
//         (vec![2.0, 2.0, 3.0], 14.0),
//         (vec![2.0, 3.0, 4.0], 20.0),
//         (vec![3.0, 4.0, 5.0], 26.0),
//         (vec![4.0, 5.0, 6.0], 32.0),
//     ]
// }

// // ------------------ Guest entry ------------------
// risc0_zkvm::guest::entry!(main);

// fn main() {
//     // Host-provided parameters (same as before)
//     let use_opt_flag: u32 = env::read(); // 0 = float, 1 = fixed
//     let use_opt = use_opt_flag != 0;
//     let model_type: u32 = env::read();   // 1..4
//     let weights: Vec<f32> = env::read();
//     let b: f32 = env::read();

//     // Use a built-in static dataset instead of reading CSV
//     let dataset = get_dataset();
    
//     assert!(!dataset.is_empty(), "Dataset loaded is empty");

//     if use_opt {
       
//         let weights_fx: Vec<i64> = weights.iter().map(|&w| f32_to_fixed(w)).collect();
//         let b_fx = f32_to_fixed(b);

//         let mut out_fx: Vec<(i64, i64)> = Vec::with_capacity(dataset.len());

//         for (features, y_true_f) in dataset.iter() {
//             // convert features to fixed
//             let features_fx: Vec<i64> = features.iter().map(|&x| f32_to_fixed(x)).collect();

//             // compute predicted value in fixed domain
//             let y_pred_fx = match model_type {
//                 1 => {
//                     // linear: uses first feature & weights[0]
//                     assert!(weights_fx.len() >= 1, "Linear model requires 1 weight");
//                     fixed_mul(weights_fx[0], features_fx[0]) + b_fx
//                 }
//                 2 => {
//                     // multiple regression: requires weights.len() == features.len()
//                     assert!(weights_fx.len() == features_fx.len(), "Multiple regression: weights length must match feature length");
//                     multiple_regression_fixed_accumulate(&features_fx, &weights_fx, b_fx)
//                 }
//                 3 => {
//                     // polynomial: use first feature as x, coeffs = weights_fx
//                     polynomial_fixed_horner(features_fx[0], &weights_fx)
//                 }
//                 4 => {
//                     // logistic: z = w·x + b, then sigmoid approx
//                     assert!(weights_fx.len() == features_fx.len(), "Logistic regression: weights length must match feature length");
//                     let z_fx = multiple_regression_fixed_accumulate(&features_fx, &weights_fx, b_fx) - b_fx;
//                     sigmoid_fixed_approx(z_fx)
//                 }
//                 _ => panic!("Unknown model type {}", model_type),
//             };

//             let y_true_fx = f32_to_fixed(*y_true_f);
//             out_fx.push((y_pred_fx, y_true_fx));
//         }

//         // convert outputs to f32 and commit
//         let out_float: Vec<(f32, f32)> = out_fx.into_iter()
//             .map(|(p_fx, t_fx)| (fixed_to_f32(p_fx), fixed_to_f32(t_fx)))
//             .collect();
//         env::commit(&out_float);
//     } else {
//         // Float-mode: produce float results
//         let mut out: Vec<(f32, f32)> = Vec::with_capacity(dataset.len());

//         for (features, y_true) in dataset.iter() {
//             let y_pred = match model_type {
//                 1 => {
//                     assert!(weights.len() >= 1, "Linear model requires 1 weight");
//                     linear_regression_f(features[0], weights[0], b)
//                 }
//                 2 => {
//                     assert!(weights.len() == features.len(), "Multiple regression: weights length must match feature length");
//                     multiple_regression_f(&features, &weights, b)
//                 }
//                 3 => {
//                     assert!(!weights.is_empty(), "Polynomial needs >= 1 coefficient");
//                     polynomial_regression_f(features[0], &weights)
//                 }
//                 4 => {
//                     assert!(weights.len() == features.len(), "Logistic regression: weights length must match feature length");
//                     logistic_regression_f(&features, &weights, b)
//                 }
//                 _ => panic!("Unknown model type {}", model_type),
//             };
//             out.push((y_pred, *y_true));
//         }

//         env::commit(&out);
//     }
// }
