/// Helper for proving things about circuits
use ark_bn254::Bn254;
use ark_circom::{CircomBuilder, CircomCircuit, CircomConfig, CircomReduction};
use ark_groth16::{create_random_proof_with_reduction, Proof, ProvingKey};
use ark_std::rand::thread_rng;
use std::{fs::File, path::PathBuf};

pub type ProofWithInputs = (Proof<Bn254>, Vec<ark_bn254::Fr>);

pub struct CircuitProver {
    builder: CircomBuilder<Bn254>,
    pub params: ProvingKey<Bn254>,
}

impl CircuitProver {
    pub fn new_path<P: Into<PathBuf>>(zkey: P, wasm: P, r1cs: P) -> Result<Self, ()> {
        let cfg = CircomConfig::<Bn254>::new(wasm.into(), r1cs.into()).unwrap();
        let builder = CircomBuilder::new(cfg);

        let mut reader = File::open(zkey.into()).unwrap();
        let (params, _) = ark_circom::read_zkey(&mut reader).unwrap();

        Ok(CircuitProver::new(builder, params))
    }

    pub fn new(builder: CircomBuilder<Bn254>, params: ProvingKey<Bn254>) -> Self {
        Self { builder, params }
    }
}

fn prove(circuit: CircomCircuit<Bn254>, params: &ProvingKey<Bn254>) -> Result<ProofWithInputs, ()> {
    // TODO: Make this a Result
    let public_inputs = circuit.get_public_inputs().unwrap();
    let proof = create_random_proof_with_reduction::<_, _, _, CircomReduction>(
        circuit,
        params,
        &mut thread_rng(),
    )
    .unwrap();
    Ok((proof, public_inputs))
}
