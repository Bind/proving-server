/// Helper for proving things about circuits
use ark_bn254::Bn254;
use ark_circom::{CircomBuilder, CircomCircuit, CircomConfig, CircomReduction};
use ark_groth16::{create_random_proof_with_reduction, Proof, ProvingKey};
use ark_std::rand::thread_rng;
use num_bigint::ToBigInt;
use std::{collections::HashMap, fs::File, path::PathBuf};

use crate::storage::ProverConfig;

pub type ProofWithInputs = (Proof<Bn254>, Vec<ark_bn254::Fr>);

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

pub fn build_inputs(
    circuit: &CircuitProver,
    cfg: ProverConfig,
    params: HashMap<String, u64>,
) -> CircomCircuit<Bn254> {
    let mut builder = circuit.builder.clone();

    for param in cfg.builder_params {
        builder.push_input(
            param.clone(),
            ToBigInt::to_bigint(params.get(&param).unwrap()).unwrap(),
        )
    }
    builder.build().unwrap()
}

pub fn prove(
    circuit: CircomCircuit<Bn254>,
    params: &ProvingKey<Bn254>,
) -> Result<ProofWithInputs, ()> {
    let public_inputs = circuit.get_public_inputs().unwrap();
    let proof = create_random_proof_with_reduction::<_, _, _, CircomReduction>(
        circuit,
        params,
        &mut thread_rng(),
    )
    .unwrap();
    Ok((proof, public_inputs))
}

#[cfg(test)]
mod tests {
    use super::CircuitProver;
    use ark_bn254::Bn254;
    use ark_circom::CircomConfig;
    use ark_groth16::{
        create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
    };
    use ark_std::rand::thread_rng;
    #[test]
    fn test_circom_config_init() {
        CircomConfig::<Bn254>::new(
            "./zkey_files/6.6.6/move.wasm",
            "./zkey_files/6.6.6/move.r1cs",
        )
        .unwrap();
    }
    #[test]
    fn test_build_circuit() {
        fn max_distance(x1: i64, y1: i64, x2: i64, y2: i64) -> u64 {
            ((x1 - x2).pow(2) as f64 + (y1 - y2).pow(2) as f64).sqrt() as u64 + 1
        }
        let circuit = CircuitProver::new_path(
            String::from("./zkey_files/6.6.6/move.zkey"),
            String::from("./zkey_files/6.6.6/move.wasm"),
            String::from("./zkey_files/6.6.6/move.r1cs"),
        )
        .unwrap();

        let mut builder = circuit.builder;

        builder.push_input(String::from("y1"), 100);
        builder.push_input(String::from("x1"), 100);
        builder.push_input(String::from("x2"), 120);
        builder.push_input(String::from("y2"), 120);
        builder.push_input(String::from("r"), 8000);
        builder.push_input(String::from("distMax"), max_distance(100, 100, 120, 120));
        builder.push_input(String::from("PLANETHASH_KEY"), 1729);
        builder.push_input(String::from("SPACETYPE_KEY"), 1730);
        builder.push_input(String::from("SCALE"), 16384);
        builder.push_input(String::from("xMirror"), false as u64);
        builder.push_input(String::from("yMirror"), false as u64);
        let circom = builder.setup();
        let mut rng = thread_rng();
        let params = generate_random_parameters::<Bn254, _, _>(circom, &mut rng).unwrap();
        let circom = builder.build().unwrap();
        let inputs = circom.get_public_inputs().unwrap();
        let proof = create_random_proof(circom, &params, &mut rng).unwrap();
        let pvk = prepare_verifying_key(&params.vk);
        let verified = verify_proof(&pvk, &proof, &inputs).unwrap();
        assert!(verified);
    }
}
