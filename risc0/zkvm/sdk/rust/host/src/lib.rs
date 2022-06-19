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

#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use std::mem;

mod exception;
mod ffi;

pub use exception::Exception;

#[cxx::bridge]
mod bridge {}

/// A Result specialized for [Exception].
pub type Result<T> = std::result::Result<T, Exception>;

/// A record attesting to the correct execution of a 'method'.
///
/// Consists of:
/// * journal: all data the method wants to publicly output and commit to.
/// * seal: the cryptographic blob which proves that the receipt is valid.
pub struct Receipt {
    ptr: *const ffi::RawReceipt,
}

/// The prover generates a [Receipt] by executing a given method in a ZKVM.
pub struct Prover {
    ptr: *mut ffi::RawProver,
}

fn into_words(slice: &[u8]) -> Result<Vec<u32>> {
    let mut vec = Vec::new();
    let chunks = slice.chunks_exact(4);
    assert!(chunks.remainder().len() == 0);
    for chunk in chunks {
        let word = chunk[0] as u32
            | (chunk[1] as u32) << 8
            | (chunk[2] as u32) << 16
            | (chunk[3] as u32) << 24;
        vec.push(word);
    }
    Ok(vec)
}

impl Receipt {
    /// Verify that the current [Receipt] is a valid result of executing the
    /// method associated with the given method ID in a ZKVM.
    pub fn verify(&self, method_id: &[u8]) -> Result<()> {
        let mut err = ffi::RawError::default();
        unsafe {
            ffi::risc0_receipt_verify(&mut err, self.ptr, method_id.as_ptr(), method_id.len())
        };
        ffi::check(err, || ())
    }

    /// Provides access to the `seal` of a [Receipt].
    pub fn get_seal(&self) -> Result<&[u32]> {
        unsafe {
            let mut err = ffi::RawError::default();
            let buf = ffi::risc0_receipt_get_seal_buf(&mut err, self.ptr);
            let buf = ffi::check(err, || buf)?;
            let mut err = ffi::RawError::default();
            let len = ffi::risc0_receipt_get_seal_len(&mut err, self.ptr);
            let len = ffi::check(err, || len)?;
            Ok(std::slice::from_raw_parts(buf, len))
        }
    }

    /// Provides access to the `journal` of a [Receipt].
    pub fn get_journal(&self) -> Result<&[u8]> {
        unsafe {
            let mut err = ffi::RawError::default();
            let buf = ffi::risc0_receipt_get_journal_buf(&mut err, self.ptr);
            let buf = ffi::check(err, || buf)?;
            let mut err = ffi::RawError::default();
            let len = ffi::risc0_receipt_get_journal_len(&mut err, self.ptr);
            let len = ffi::check(err, || len)?;
            Ok(std::slice::from_raw_parts(buf, len))
        }
    }

    /// Provides access to the `journal` of a [Receipt] as a [`Vec<u32>`].
    pub fn get_journal_vec(&self) -> Result<Vec<u32>> {
        into_words(self.get_journal()?)
    }
}

impl Prover {
    /// Create a new [Prover] with the given method (specified via `elf_contents`)
    /// and an associated method ID (specified via `method_id`).
    pub fn new(elf_contents: &[u8], method_id: &[u8]) -> Result<Self> {
        let mut err = ffi::RawError::default();
        let ptr = unsafe {
            ffi::risc0_prover_new(
                &mut err,
                elf_contents.as_ptr(),
                elf_contents.len(),
                method_id.as_ptr(),
                method_id.len(),
            )
        };
        ffi::check(err, || Prover { ptr })
    }

    /// Provide private input data that is availble to guest-side method code
    /// to 'read'.
    pub fn add_input(&mut self, slice: &[u32]) -> Result<()> {
        let mut err = ffi::RawError::default();
        unsafe {
            ffi::risc0_prover_add_input(
                &mut err,
                self.ptr,
                slice.as_ptr().cast(),
                slice.len() * mem::size_of::<u32>(),
            )
        };
        ffi::check(err, || ())
    }

    /// Provide access to private output data written by guest-side method code.
    pub fn get_output(&self) -> Result<&[u8]> {
        unsafe {
            let mut err = ffi::RawError::default();
            let buf = ffi::risc0_prover_get_output_buf(&mut err, self.ptr);
            let buf = ffi::check(err, || buf)?;
            let mut err = ffi::RawError::default();
            let len = ffi::risc0_prover_get_output_len(&mut err, self.ptr);
            let len = ffi::check(err, || len)?;
            Ok(std::slice::from_raw_parts(buf, len))
        }
    }

    /// Provide access to private output data written to by guest-side method
    /// code.
    ///
    /// This returns the data as a [`Vec<u32>`].
    pub fn get_output_vec(&self) -> Result<Vec<u32>> {
        into_words(self.get_output()?)
    }

    /// Execute the ZKVM to produce a [Receipt].
    pub fn run(&self) -> Result<Receipt> {
        let mut err = ffi::RawError::default();
        let ptr = unsafe { ffi::risc0_prover_run(&mut err, self.ptr) };
        ffi::check(err, || Receipt { ptr })
    }
}

impl Drop for Receipt {
    fn drop(&mut self) {
        let mut err = ffi::RawError::default();
        unsafe { ffi::risc0_receipt_free(&mut err, self.ptr) };
        ffi::check(err, || ()).unwrap()
    }
}

impl Drop for Prover {
    fn drop(&mut self) {
        let mut err = ffi::RawError::default();
        unsafe { ffi::risc0_prover_free(&mut err, self.ptr) };
        ffi::check(err, || ()).unwrap()
    }
}

#[ctor::ctor]
fn init() {
    unsafe { ffi::risc0_init() };
}

#[cfg(test)]
mod test {
    use super::Prover;
    use anyhow::Result;
    use risc0_zkvm_core::Digest;
    use risc0_zkvm_methods::methods::{FAIL_ID, FAIL_PATH, IO_ID, IO_PATH, SHA_ID, SHA_PATH};
    use risc0_zkvm_serde::{from_slice, to_vec};

    #[test]
    fn sha() {
        assert_eq!(
            run_sha(""),
            Digest::new([
                0xe3b0c442, 0x98fc1c14, 0x9afbf4c8, 0x996fb924, 0x27ae41e4, 0x649b934c, 0xa495991b,
                0x7852b855,
            ])
        );
        assert_eq!(
            run_sha("a"),
            Digest::new([
                0xca978112, 0xca1bbdca, 0xfac231b3, 0x9a23dc4d, 0xa786eff8, 0x147c4e72, 0xb9807785,
                0xafee48bb,
            ])
        );
        assert_eq!(
            run_sha("abc"),
            Digest::new([
                0xba7816bf, 0x8f01cfea, 0x414140de, 0x5dae2223, 0xb00361a3, 0x96177a9c, 0xb410ff61,
                0xf20015ad
            ])
        );
        assert_eq!(
            run_sha("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            Digest::new([
                0x248d6a61, 0xd20638b8, 0xe5c02693, 0x0c3e6039, 0xa33ce459, 0x64ff2167, 0xf6ecedd4,
                0x19db06c1
            ])
        );
    }

    fn run_sha(msg: &str) -> Digest {
        let elf_contents = std::fs::read(SHA_PATH).unwrap();
        let mut prover = Prover::new(&elf_contents, SHA_ID).unwrap();
        let vec = to_vec(&msg).unwrap();
        prover.add_input(vec.as_slice()).unwrap();
        let receipt = prover.run().unwrap();
        let vec = receipt.get_journal_vec().unwrap();
        from_slice::<Digest>(vec.as_slice()).unwrap()
    }

    #[test]
    fn memory_io() {
        const HEAP_START: u32 = 0x0008_0000;
        const COMMIT_START: u32 = 0x0038_0000;

        // Double write to WOM are fine
        assert!(run_memio(&[(COMMIT_START, 1), (COMMIT_START, 1)]).is_ok());

        // Double write to WOM with different values throw
        assert!(run_memio(&[(COMMIT_START, 1), (COMMIT_START, 2)]).is_err());

        // But they are OK at different addresses
        assert!(run_memio(&[(COMMIT_START, 1), (COMMIT_START + 4, 2)]).is_ok());

        // Aligned write is fine
        assert!(run_memio(&[(HEAP_START, 1)]).is_ok());

        // Unaligned write is bad
        assert!(run_memio(&[(HEAP_START + 1, 1)]).is_err());

        // Aligned read is fine
        assert!(run_memio(&[(HEAP_START, 0)]).is_ok());

        // Unaligned read is bad
        assert!(run_memio(&[(HEAP_START + 1, 0)]).is_err());
    }

    fn run_memio(pairs: &[(u32, u32)]) -> Result<()> {
        let mut vec = Vec::new();
        vec.push(pairs.len() as u32);
        for (first, second) in pairs {
            vec.push(*first);
            vec.push(*second);
        }
        let elf_contents = std::fs::read(IO_PATH).unwrap();
        let mut prover = Prover::new(&elf_contents, IO_ID).unwrap();
        prover.add_input(vec.as_slice()).unwrap();
        let receipt = prover.run()?;
        receipt.verify(IO_ID).unwrap();
        Ok(())
    }

    #[test]
    fn fail() {
        // Check that a compliant host will fault.
        let elf_contents = std::fs::read(FAIL_PATH).unwrap();
        let prover = Prover::new(&elf_contents, FAIL_ID).unwrap();
        assert!(prover.run().is_err());
    }
}
