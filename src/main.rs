use wasmer::{Memory, MemoryType, Instance, Module, Store};
use wasmer_wasix::{generate_import_object_from_env, WasiEnv, WasiVersion};
use color_eyre::Result;
use num_bigint::BigInt;
use ark_circom::{Wasm, WitnessCalculator};
use serde_json::Value;
use std::{collections::HashMap, str::FromStr};
use num_traits::Num;


fn main() {

}

fn run_example() -> Result<()> {
    let mut store = Store::default();
    let wasm = create_wasm_instance(&mut store)?;
    let wc = WitnessCalculator::new_from_wasm(&mut store, wasm);
    Ok(())
    
}

fn parse_inputs(inputs_path: impl AsRef<std::path::Path>) -> HashMap<String, Vec<BigInt>>{
    let inputs_str = std::fs::read_to_string(inputs_path).unwrap();
    let inputs: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&inputs_str).unwrap();

    inputs
        .iter()
        .map(|(key, value)| {
            let res = match value {
                Value::String(inner) => {
                    vec![BigInt::from_str(inner).unwrap()]
                }
                Value::Array(inner) => inner.iter().cloned().map(value_to_bigint).collect(),
                _ => panic!(),
            };

            (key.clone(), res)
        })
        .collect::<HashMap<_, _>>()
}


fn value_to_bigint(v: Value) -> BigInt {
     match v {
         Value::String(inner) => BigInt::from_str_radix(&inner, 16).unwrap(),
         _ => panic!("unsupported type"),
     }
 }

fn create_wasm_instance(store: &mut Store) -> Result<Wasm> {
    let module = Module::from_file(&store, "./circuit-assets/circuit.wasm")?;
    let wasi_env = 
          WasiEnv::builder("calculateWitness")
            .preopen_dir("./circuit-assets")?
            .finalize(store)?;
    let wasi_env_imports =
        generate_import_object_from_env(store, &wasi_env.env, WasiVersion::Snapshot1);
    let instance = Instance::new(store, &module, &wasi_env_imports)?;
    let exports = instance.exports.clone();
    let memory = Memory::new(store, MemoryType::new(2000, None, false)).unwrap();
    let wasm = Wasm::new(exports, memory);
    Ok(wasm)
}