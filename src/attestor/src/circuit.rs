use ark_ff::Field;
use ark_relations::lc;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

/// A benchmark circuit that simulates the complexity of checking policy compliance
/// for a batch of requests.
///
/// In a real system, this would verify Merkle paths and Policy signatures.
/// Here, we perform N constraints of "dummy work" to simulate the proving load.
#[derive(Clone)]
pub struct ComplianceCircuit<F: Field> {
    pub num_constraints: usize,
    pub public_input: F,
}

impl<F: Field> ConstraintSynthesizer<F> for ComplianceCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        use ark_relations::r1cs::Variable;

        // Allocate the public input
        let mut curr_val = self.public_input;
        let mut curr_var = cs.new_witness_variable(|| Ok(curr_val))?;

        for _ in 0..self.num_constraints {
            // Square the previous value: curr * curr = next
            let next_val = curr_val * curr_val;
            let next_var = cs.new_witness_variable(|| Ok(next_val))?;

            // Enforce: curr * curr = next
            cs.enforce_constraint(lc!() + curr_var, lc!() + curr_var, lc!() + next_var)?;

            curr_val = next_val;
            curr_var = next_var;
        }

        Ok(())
    }
}
