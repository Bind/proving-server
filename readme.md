# Proving Server v1

- [ ] Dynamically store proving information
- [ ] Dynamically fetch wasm, zkey, etc
- [ ] Dynamically configure CircomProver to be invoked via endpoint

tentative endpoints:
  Post /prover
    Create a new Prover
    Req {
      path_to_wasm: String,
      path_to_zkey: String
      builder_params: Vec<String> //named parameters to be defined when calling the generate proof
      name: String, user defined name,
      version: String, user defined version
    }
  
  Get /prover/${name}/${version}
    Get current creation status
  
     Resp {
      0: not started
      1: loading external resources
      2: instantiating prover
      3: ready
      }
  Post /prover/${name}/${version}
      Generate new proof 
      {
      // Object with each item in builder_params as a key
  
      }

Refs:

- https://github.com/projectsophon/darkforest-rs/blob/main/mimc-fast/src/main.rs
- https://github.com/gakonst/dark-forest/blob/eaad405c0e9c9f7acf2189a14d87767026d98651/crates/df-engine/src/prover/mover.rs
