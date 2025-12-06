use ark_bls12_381::{Bls12_381, Fr};
use ark_groth16::Groth16;
use ark_snark::SNARK;
use ark_std::rand::rngs::StdRng;
use ark_std::rand::SeedableRng;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem};
use clap::Parser;
use runtimeguard_attestor::circuit::ComplianceCircuit;
use std::time::Instant;
use serde::Serialize;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 50000)]
    constraints: usize,

    #[arg(short, long, default_value_t = 10)]
    samples: usize,
}

#[derive(Serialize)]
struct BenchmarkResult {
    constraints: usize,
    framework: String,
    hardware: String,
    mean_proving_ms: f64,
    mean_witness_ms: f64,
    std_dev_proving_ms: f64,
    samples: usize,
}

fn main() {
    let args = Args::parse();
    let mut rng = StdRng::seed_from_u64(42);

    println!("Setting up circuit with {} constraints...", args.constraints);

    let circuit = ComplianceCircuit::<Fr> {
        num_constraints: args.constraints,
        public_input: Fr::from(100),
    };

    // Trusted Setup
    let start_setup = Instant::now();
    let (pk, _) = Groth16::<Bls12_381>::circuit_specific_setup(circuit.clone(), &mut rng).unwrap();
    println!("Setup complete in {:.2}s", start_setup.elapsed().as_secs_f64());

    println!("Benchmarking witness generation and proof generation over {} samples...", args.samples);
    
    let mut proving_times_ms = Vec::with_capacity(args.samples);
    let mut witness_times_ms = Vec::with_capacity(args.samples);

    for i in 0..args.samples {
        // Measure Witness Generation
        let start_witness = Instant::now();
        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.clone().generate_constraints(cs.clone()).unwrap();
        // Force witness computation
        assert!(cs.is_satisfied().unwrap());
        let witness_duration = start_witness.elapsed();
        witness_times_ms.push(witness_duration.as_secs_f64() * 1000.0);

        // Measure Proving (Note: Groth16::prove re-runs witness gen internally usually, 
        // so this measures Total Proving Time including Witness Gen)
        let start_prove = Instant::now();
        let _proof = Groth16::<Bls12_381>::prove(&pk, circuit.clone(), &mut rng).unwrap();
        let prove_duration = start_prove.elapsed();
        proving_times_ms.push(prove_duration.as_secs_f64() * 1000.0);
        
        println!("Sample {}: Witness Generation {:.2} ms | Total Proving Time (incl. Witness) {:.2} ms", 
            i + 1, 
            witness_duration.as_secs_f64() * 1000.0,
            prove_duration.as_secs_f64() * 1000.0
        );
    }

    // Statistics
    let mean_prove = proving_times_ms.iter().sum::<f64>() / args.samples as f64;
    let mean_witness = witness_times_ms.iter().sum::<f64>() / args.samples as f64;
    
    let variance_prove: f64 = proving_times_ms.iter().map(|t| (t - mean_prove).powi(2)).sum::<f64>() / args.samples as f64;
    let std_dev_prove = variance_prove.sqrt();

    let result = BenchmarkResult {
        constraints: args.constraints,
        framework: "ark-groth16 (bls12-381)".to_string(),
        hardware: "User-Hardware".to_string(),
        mean_proving_ms: mean_prove,
        mean_witness_ms: mean_witness,
        std_dev_proving_ms: std_dev_prove,
        samples: args.samples,
    };

    println!("\nFinal Results:");
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
