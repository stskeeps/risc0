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

//! Platform definitions for ZKVM, including IO port addresses, memory
//! regions, and low-level runtime functions.

#![no_std]

pub mod io;
pub mod memory;

pub const WORD_SIZE: usize = core::mem::size_of::<u32>();

#[cfg(target_os = "zkvm")]
/// Runtime support for running on the ZKVM.  These are low-level
/// routines; in general, users should prefer to use the
/// risc0-zkvm-guest package.
pub mod rt;

/// Stub out guest-only routines for non-guest compiles.
#[cfg(not(target_os = "zkvm"))]
pub mod rt {
    pub mod host_io {
        pub fn host_sendrecv(_channel: u32, _buf: &[u8]) -> (&'static [u32], usize) {
            unimplemented!()
        }
        pub fn host_recv(_nwords: usize) -> &'static [u32] {
            unimplemented!()
        }
    }
}
