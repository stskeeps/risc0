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

use rand::{Error, RngCore};

use crate::core::{
    sha::{Digest, Sha},
    sha_rng::ShaRng,
};

#[derive(Debug)]
pub struct ReadIOP<'a, S: Sha + 'a> {
    sha: S,
    proof: &'a [u32],
    rng: ShaRng<S>,
}

impl<'a, S: Sha + 'a> ReadIOP<'a, S> {
    pub fn new(sha: &'a S, proof: &'a [u32]) -> Self {
        ReadIOP {
            sha: sha.clone(),
            proof,
            rng: ShaRng::new(sha),
        }
    }

    pub fn get_sha(&self) -> &S {
        &self.sha
    }

    pub fn read_u32s(&mut self, n: usize) -> &'a [u32] {
        let u32s;
        (u32s, self.proof) = self.proof.split_at(n);
        u32s
    }

    pub fn read_pod_slice<T: bytemuck::Pod>(&mut self, n: usize) -> &'a [T] {
        let u32s;
        (u32s, self.proof) = self
            .proof
            .split_at(n * core::mem::size_of::<T>() / core::mem::size_of::<u32>());
        bytemuck::cast_slice(u32s)
    }

    pub fn commit(&mut self, digest: &Digest) {
        self.rng.mix(digest);
    }

    pub fn verify_complete(&self) {
        assert_eq!(self.proof.len(), 0);
    }
}

impl<'a, S: Sha> RngCore for ReadIOP<'a, S> {
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.rng.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.rng.try_fill_bytes(dest)
    }
}
