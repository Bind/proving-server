use ark_bn254::Bn254;
use ark_circom::ethereum::Proof;
use ark_circom::{CircomBuilder, CircomConfig};
use ark_groth16::{Proof as GrothProof, ProvingKey};
use ethers::types::U256;
use std::collections::HashMap;
use std::{fs::File, path::PathBuf};

pub type Abc = ([U256; 2], [[U256; 2]; 2], [U256; 2]);
pub fn to_eth_type<P: Into<Proof>>(proof: P) -> Abc {
    // lay the proof in the correct order
    let proof = proof.into();
    let proof = proof.as_tuple();
    let a = [proof.0 .0, proof.0 .1];
    // b.as_tuple() already handles the reverse ordering in G2 elements
    let b = [proof.1 .0, proof.1 .1];
    let c = [proof.2 .0, proof.2 .1];
    (a, b, c)
}

pub type ProofInputs = HashMap<String, u64>;

pub type ProofWithInputs = (GrothProof<Bn254>, Vec<ark_bn254::Fr>);

pub struct CircuitProver {
    pub builder: CircomBuilder<Bn254>,
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
