use risc0_zkvm::Receipt;
use serde_json;
use std::fs;
fn main() {
    const METHOD_ID: [u32; 8] =  [8615346, 3088364040, 3309643489, 2403364783, 529424834, 3266678953, 590165670, 1240344216];


    //  deserialize the proof.json
    let data = fs::read_to_string("proof.json")
        .expect("Failed to read proof.json");

    let receipt: Receipt = serde_json::from_str(&data)
        .expect("Failed to parse receipt");

    //  Verify against method_id
    match receipt.verify(METHOD_ID) {
        Ok(_) => println!("✅ Proof verified successfully!"),
        Err(e) => println!("❌ Verification failed: {:?}", e),
    }
}