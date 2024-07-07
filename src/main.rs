use ark_circom::{Wasm, WitnessCalculator};
use color_eyre::Result;
use num_bigint::BigInt;
use num_traits::Num;
use serde_json::Value;
use std::{collections::HashMap, path::PathBuf};
use wasmer::{Instance, Memory, MemoryType, Module, Store};
use wasmer_wasix::{generate_import_object_from_env, WasiEnv, WasiVersion};

#[tokio::main]
async fn main() {
    let wtns = generate_witness().unwrap();
    print!("witness {:?}", wtns);
}

// generate the witness given the input
fn generate_witness() -> Result<Vec<BigInt>> {
    let mut store = Store::default();
    let wasm = create_wasm_instance(&mut store, PathBuf::from("./circuit-assets"))?;
    let inputs = parse_inputs("./inputs.json");
    let mut wc = WitnessCalculator::new_from_wasm(&mut store, wasm)?;
    wc.calculate_witness(&mut store, inputs, false)
}

// parse json inputs where the values are either hex encoded int or an array of such
fn parse_inputs(inputs_path: impl AsRef<std::path::Path>) -> HashMap<String, Vec<BigInt>> {
    let inputs_str = std::fs::read_to_string(inputs_path).unwrap();
    let inputs: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&inputs_str).unwrap();

    fn value_to_bigint(v: Value) -> BigInt {
        match v {
            Value::String(inner) => {
                BigInt::from_str_radix(&inner.strip_prefix("0x").unwrap(), 16).unwrap()
            }
            _ => panic!("unsupported type, requires hex encoded int"),
        }
    }

    inputs
        .iter()
        .map(|(key, value)| {
            let res = match value {
                Value::String(_) => {
                    vec![value_to_bigint(value.clone())]
                }
                Value::Array(inner) => inner.iter().cloned().map(value_to_bigint).collect(),
                _ => panic!(),
            };

            (key.clone(), res)
        })
        .collect::<HashMap<_, _>>()
}

// Create a wasm instance that loads the witness solver binary and has access to the circuit.bin file
fn create_wasm_instance(store: &mut Store, circuit_dir: PathBuf) -> Result<Wasm> {
    let module = Module::from_file(&store, circuit_dir.clone().join("circuit.wasm"))?;
    let host_fs = wasmer_wasix::virtual_fs::host_fs::FileSystem::default();
    let mut wasi_env = WasiEnv::builder("calculateWitness")
        .fs(Box::new(host_fs))
        .preopen_dir("/")?
        .map_dir("/", circuit_dir)?
        .finalize(store)?;
    let wasi_env_imports =
        generate_import_object_from_env(store, &wasi_env.env, WasiVersion::Snapshot1);
    let memory = Memory::new(store, MemoryType::new(2000, None, false)).unwrap();
    let instance = Instance::new(store, &module, &wasi_env_imports)?;
    let exports = instance.exports.clone();
    wasi_env.initialize_with_memory(store, instance, Some(memory.clone()), false)?;
    let wasm = Wasm::new(exports, memory);
    Ok(wasm)
}
