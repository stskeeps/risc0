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

use anyhow::Result;

pub const METHOD_ID_LEN: usize = 384; // https://github.com/dtolnay/cxx/issues/1051
pub type MethodId = [u8; METHOD_ID_LEN];

#[cxx::bridge(namespace = "risc0")]
pub mod ffi {
    unsafe extern "C++" {
        include!("risc0/zkvm/prove/method_id.h");

	// Unfortunately cxx::bridge doesn't seem to let us reference METHOD_ID_LEN here. :(
        unsafe fn methodIdBytesFromElf(elf_contents: *const u8, len: usize) -> Result<[u8; 384]>;
    }
}

pub fn make_method_id_from_elf(elf_contents: &[u8]) -> Result<MethodId> {
    unsafe {
        Ok(ffi::methodIdBytesFromElf(
            elf_contents.as_ptr(),
            elf_contents.len(),
        )?)
    }
}
