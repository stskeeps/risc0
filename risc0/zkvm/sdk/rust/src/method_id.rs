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

use alloc::vec::Vec;

use anyhow::Result;
use risc0_zkp::{
    core::{
        log2_ceil,
        sha::{Digest, DIGEST_WORDS, DIGEST_WORD_SIZE},
    },
    MAX_CYCLES, MIN_CYCLES,
};

/// The default digest count when generating a MethodId.
pub const DEFAULT_METHOD_ID_LIMIT: u32 = 16;

pub const MAX_CODE_DIGEST_COUNT: u32 = (log2_ceil(MAX_CYCLES / MIN_CYCLES) + 1) as _;

#[derive(Clone, Eq, PartialEq)]
pub struct MethodId {
    pub table: Vec<Digest>,
}

impl From<&[u8]> for MethodId {
    fn from(bytes: &[u8]) -> Self {
        MethodId::from_slice(bytes).unwrap()
    }
}

impl From<&[u32]> for MethodId {
    fn from(words: &[u32]) -> Self {
        let mut table = Vec::new();
        for digest in words.chunks_exact(DIGEST_WORDS) {
            table.push(Digest::from_slice(digest));
        }
        MethodId { table }
    }
}

impl MethodId {
    pub fn as_slice(&self) -> Result<&[u8]> {
        Ok(bytemuck::cast_slice(self.table.as_slice()))
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self> {
        let mut table = Vec::new();
        for digest in bytes.chunks_exact(DIGEST_WORDS * DIGEST_WORD_SIZE) {
            let words: Vec<u32> = digest
                .chunks_exact(DIGEST_WORD_SIZE)
                .map(|x| {
                    let mut word = 0;
                    for i in 0..4 {
                        word |= (x[i] as u32) << (i * 8);
                    }
                    word
                })
                .collect();
            table.push(Digest::try_from_slice(&words)?);
        }
        Ok(MethodId { table })
    }

    #[cfg(feature = "prove")]
    pub fn compute(elf_contents: &[u8]) -> Result<Self> {
        MethodId::compute_with_limit(elf_contents, DEFAULT_METHOD_ID_LIMIT)
    }

    #[cfg(feature = "prove")]
    pub fn compute_with_limit(elf_contents: &[u8], limit: u32) -> Result<Self> {
        prove::compute_with_limit(elf_contents, limit)
    }
}

#[cfg(feature = "prove")]
mod prove {
    use anyhow::Result;
    use risc0_zkp::{
        adapter::TapsProvider,
        core::{fp::Fp, sha::Digest},
        field::baby_bear::BabyBear,
        hal::{cpu::CpuHal, Hal},
        prove::poly_group::PolyGroup,
        MIN_CYCLES, ZK_CYCLES,
    };
    use risc0_zkvm_platform::memory::MEM_SIZE;

    use super::{MethodId, MAX_CODE_DIGEST_COUNT};
    use crate::{elf::Program, prove::exec, CIRCUIT};

    pub fn compute_with_limit(elf_contents: &[u8], limit: u32) -> Result<MethodId> {
        let code_size = CIRCUIT.code_size();
        let hal: CpuHal<BabyBear> = CpuHal::new();
        let program = Program::load_elf(elf_contents, MEM_SIZE as u32)?;

        // Start with an empty table
        let mut table = Vec::new();

        // Make the digest for each level
        let count = std::cmp::min(limit, MAX_CODE_DIGEST_COUNT);
        for i in 0..count {
            let cycles = MIN_CYCLES * (1 << i);
            if cycles < program.image.len() + 3 + ZK_CYCLES {
                // Can't even fit the program in this cycle size, just set to zero
                table.push(Digest::default());
                continue;
            }

            // Make a vector & set it up with the elf data
            let mut code = vec![Fp::default(); cycles * code_size];
            load_code(&mut code, &program, cycles)?;

            // Copy into accel buffer
            let coeffs = hal.copy_fp_from(&code);
            // Do interpolate & shift
            hal.batch_interpolate_ntt(&coeffs, code_size);
            hal.zk_shift(&coeffs, code_size);
            // Make the poly-group & extract the root
            let code_group = PolyGroup::new(&hal, &coeffs, code_size, cycles);
            table.push(code_group.merkle.root().clone());
        }

        Ok(MethodId { table })
    }

    fn load_code(code: &mut [Fp], elf: &crate::elf::Program, cycles: usize) -> Result<()> {
        let code_size = CIRCUIT.code_size();
        let mut cycle = 0;
        exec::load_code(elf.entry, &elf.image, |chunk, fini| {
            for i in 0..code_size {
                code[cycles * i + cycle] = chunk[i];
            }
            if cycle + fini + ZK_CYCLES < cycles {
                cycle += 1;
                Ok(true)
            } else {
                Ok(false)
            }
        })
    }
}
