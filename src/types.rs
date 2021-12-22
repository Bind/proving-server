use ark_circom::ethereum::Proof;
use ethers::types::U256;
use std::collections::HashMap;
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
