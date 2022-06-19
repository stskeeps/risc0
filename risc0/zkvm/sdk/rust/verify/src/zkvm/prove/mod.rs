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

mod circuit;
pub(crate) mod exec;
mod io;
pub mod prover;
mod step;
mod step_context;

const OUTPUT_REGS: usize = 9;
const REGISTERS_GLOBAL_SIZE: usize = OUTPUT_REGS * 2;
const ACCUM_MIX_GLOBAL_SIZE: usize = 20;
const ACCUM_MIX_GLOBAL_OFFSET: usize = REGISTERS_GLOBAL_SIZE;
const GLOBAL_SIZE: usize = ACCUM_MIX_GLOBAL_OFFSET + ACCUM_MIX_GLOBAL_SIZE;
