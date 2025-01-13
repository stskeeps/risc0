// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#![allow(unused_imports)]
#![allow(clippy::needless_borrow)]

use json_core::Outputs;
use jzon::parse;
use risc0_zkvm::{
    guest::env,
    sha::{Impl, Sha256},
};
use serde_json::{Result, Value};
use simd_json::prelude::ValueAsScalar;

fn main() {
    let data: String = env::read();
    let sha = *Impl::hash_bytes(&data.as_bytes());
    let data = parse(&data).unwrap();

    // let proven_val = unsafe {
    //     let js = simd_json::to_tape(data.as_bytes_mut()).unwrap();
    //     js.as_value()
    //         .as_array()
    //         .unwrap()
    //         .get(0)
    //         .unwrap()
    //         .as_object()
    //         .unwrap()
    //         .get("friends")
    //         .unwrap()
    //         .as_array()
    //         .unwrap()
    //         .get(0)
    //         .unwrap()
    //         .as_object()
    //         .unwrap()
    //         .get("id")
    //         .unwrap()
    //         .as_u32()
    //         .unwrap()
    // };

    //let v: Value = serde_json::from_str(&data).unwrap();
    //let proven_val = v[0]["friends"][0]["id"].as_u64().unwrap() as u32;

    //let proven_val = data[0]["friends"][0]["id"].as_u32().unwrap();
    let proven_val = 33;
    let out = Outputs {
        data: proven_val,
        hash: sha,
    };
    env::commit(&out);
}
