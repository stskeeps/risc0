// Copyright 2022 Risc0, Inc.
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

use std::default::Default;
use std::{fs, io::Write};

use clap::Parser;
use risc0_zkvm::host::{MethodId, Prover, ProverOpts, Receipt, DEFAULT_METHOD_ID_LIMIT};

/// Generates a MethodID for a given RISC-V ELF binary.
#[derive(Parser)]
#[clap(about, version, author)]
struct Args {
    /// The ELF file to run
    #[clap(long)]
    elf: String,

    /// MethodID file; created if needed and it doesn't exist.
    #[clap(long)]
    method_id: Option<String>,

    /// Receipt output file.
    #[clap(long)]
    receipt: Option<String>,

    /// Skip generating the seal in receipt.  This should only be used
    /// for testing.  In this case, performace will be much better but
    /// we will not be able to cryptographically verify the execution.
    #[clap(long)]
    skip_seal: bool,

    /// File to read initial input from.
    #[clap(long)]
    initial_input: Option<String>,

    /// Display verbose output.
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Limit the number of hash table entries to compute.
    #[clap(short, long, default_value_t = DEFAULT_METHOD_ID_LIMIT)]
    limit: u32,
}

fn read_method_id(
    verbose: u8,
    elf_file: &str,
    method_id_file: &Option<String>,
) -> Option<MethodId> {
    let elf_mtime = fs::metadata(elf_file).ok()?.modified().ok()?;
    let id_mtime = fs::metadata(method_id_file.as_ref()?)
        .ok()?
        .modified()
        .ok()?;

    if elf_mtime > id_mtime {
        return None;
    }

    let id = MethodId::from_slice(&fs::read(method_id_file.as_ref()?).ok()?).ok()?;

    // TODO(nils): Check to make sure the limit is the same as the one
    // that was saved.

    if verbose > 0 {
        println!(
            "Successfully read method id from {}",
            method_id_file.as_ref().unwrap()
        );
    }

    Some(id)
}

fn main() {
    env_logger::init();

    let args = Args::parse();
    let elf_contents = fs::read(&args.elf).unwrap();

    if args.verbose > 0 {
        eprintln!(
            "Read {} bytes of ELF from {}",
            elf_contents.len(),
            &args.elf
        );
    }

    let method_id: MethodId = if args.receipt.is_none() || args.skip_seal {
        // No need to generate a method ID since we don't need to
        // generate an actual proof.
        MethodId::from_slice(&[]).unwrap()
    } else {
        read_method_id(args.verbose, &args.elf, &args.method_id).unwrap_or_else(|| {
            if args.verbose > 0 {
                eprintln!("Computing method id");
            }
            let computed = MethodId::compute_with_limit(&elf_contents, args.limit).unwrap();
            if let Some(method_id_file) = args.method_id {
                std::fs::write(&method_id_file, computed.as_slice().unwrap()).unwrap();
                if args.verbose > 0 {
                    eprintln!("Saved method id to {}", method_id_file);
                }
            }
            computed
        })
    };

    let opts: ProverOpts =
        ProverOpts::default().with_skip_seal(args.skip_seal || args.receipt.is_none());

    let mut prover =
        Prover::new_with_opts(&elf_contents, method_id.as_slice().unwrap(), opts).unwrap();
    if let Some(input) = args.initial_input {
        let input_bytes = fs::read(input).unwrap();
        if args.verbose > 0 {
            eprintln!("Supplying {} bytes of initial input", input_bytes.len());
        }
        prover.add_input_u8_slice(&input_bytes);
    }

    let receipt: Receipt = prover.run().unwrap();
    let receipt_data = risc0_zkvm::serde::to_vec(&receipt).unwrap();

    if args.skip_seal || args.receipt.is_none() {
        if args.verbose > 0 {
            eprintln!("Skipping seal generation.");
        }
    } else {
        if args.verbose > 0 {
            eprintln!("Verifying that we executed correctly.");
            receipt.verify(method_id.as_slice().unwrap()).unwrap();
        }
    }
    if let Some(receipt_file) = args.receipt {
        fs::write(&receipt_file, bytemuck::cast_slice(&receipt_data)).unwrap();
        if args.verbose > 0 {
            eprintln!(
                "Wrote {} bytes of receipt to {}",
                receipt_data.len(),
                receipt_file
            );
        }
    }
    let output = prover.get_output().unwrap();
    if args.verbose > 0 {
        eprintln!("Writing {} bytes of output to stdout", output.len());
    }
    std::io::stdout().write_all(output).unwrap();
}
