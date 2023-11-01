mod circuits;
mod generator;

use anyhow::Result;
use circuits::simple::SimpleCircuit;
use generator::*;
use halo2_curves::bn256::Fr;
use halo2_proofs::circuit::Value;
use serde_json::Value as JsonValue;
use snark_verifier::loader::evm::encode_calldata;
use ws_sdk::log::log_info;
use ws_sdk::stream::get_data;

#[no_mangle]
pub extern "C" fn start(rid: i32) -> i32 {
    match handle(rid) {
        Ok(_) => return 0,
        _ => return -1,
    };
}

fn handle(rid: i32) -> Result<()> {
    log_info(&format!("start rid: {}", rid))?;
    let data_str = String::from_utf8(get_data(rid as _)?)?;
    log_info(&format!("get resource {}: `{}`", rid, data_str))?;
    let v: JsonValue = serde_json::from_str(&data_str).unwrap();
    let private_a = Fr::from(v["private_a"].as_str().unwrap().parse::<u64>().unwrap());
    let private_b = Fr::from(v["private_b"].as_str().unwrap().parse::<u64>().unwrap());
    let constant = Fr::from(3);

    // 1. gen params or use w3b params
    let params = gen_srs(4);
    let empty_circuit = SimpleCircuit {
        constant,
        a: Value::unknown(),
        b: Value::unknown(),
    };

    // 2. gen pk
    let pk = gen_pk(&params, &empty_circuit);

    let c = constant * private_a.square() * private_b.square();
    println!("{:?}", c);
    // let c = Fr::from(15);
    let circuit = SimpleCircuit {
        constant,
        a: Value::known(private_a),
        b: Value::known(private_b),
    };
    let instances = vec![vec![c]];
    let proof = gen_proof(&params, &pk, circuit.clone(), &instances);
    let calldata = encode_calldata(&instances, &proof);
    let _ = log_info(&format!("calldata is: 0x{}", hex::encode(&calldata)));

    Ok(())
}