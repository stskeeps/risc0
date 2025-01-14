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

use core::{
    cmp::Ordering,
    ops::{Index, IndexMut},
};
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

use anyhow::{bail, Result};
use bytemuck::Pod;
use lazy_regex::{regex, Captures};
use log::{debug, trace};
use risc0_zkp::{
    adapter::{CircuitDef, CustomStep, PolyExt, PolyExtContext, TapsProvider},
    core::{
        fp::Fp,
        fp4::Fp4,
        log2_ceil,
        sha::{Digest, Sha, DIGEST_WORDS, SHA256_INIT},
    },
    field::Elem,
    prove::executor::Executor,
    MAX_CYCLES_PO2, ZK_CYCLES,
};
use risc0_zkvm_circuit::CircuitImpl;
use risc0_zkvm_platform::{
    io::{
        addr::{
            GPIO_COMMIT, GPIO_COMPUTE_POLY, GPIO_CYCLECOUNT, GPIO_FAULT, GPIO_FFPU, GPIO_GETKEY,
            GPIO_INSECURESHACOMPRESS, GPIO_INSECURESHAHASH, GPIO_LOG, GPIO_POLY_EVAL,
            GPIO_SENDRECV_ADDR, GPIO_SENDRECV_CHANNEL, GPIO_SENDRECV_SIZE, GPIO_SHA,
        },
        ComputePolyDescriptor, FfpuDescriptor, InsecureShaCompressDescriptor,
        InsecureShaHashDescriptor, IoDescriptor, PolyEvalDescriptor, SHADescriptor,
        SliceDescriptor,
    },
    memory::{INPUT, MEM_BITS},
    WORD_SIZE,
};

use super::ffpu::ffpu_execute;
use crate::{elf::Program, CIRCUIT};

pub trait IoHandler {
    fn on_commit(&mut self, buf: &[u32]) -> Result<()>;
    fn on_fault(&mut self, msg: &str) -> Result<()>;
    fn on_txrx(&mut self, channel: u32, buf: &[u8]) -> Result<Vec<u8>>;
}

#[derive(Clone, PartialEq, Eq)]
struct MemoryEvent {
    pub cycle: u32,
    pub addr: u32,
    pub data: u32,
    pub is_write: bool,
}

impl std::fmt::Debug for MemoryEvent {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("MemoryEvent")
            .field("cycle", &self.cycle)
            .field("addr", &format_args!("0x{:08X}", self.addr * 4))
            .field("data", &format_args!("0x{:08X}", self.data))
            .field("is_write", &self.is_write)
            .finish()
    }
}

struct MemoryState {
    pub memory: BTreeMap<u32, u32>,
    pub history: BTreeSet<MemoryEvent>,
}

/// Align the given address `addr` upwards to alignment `align`.
///
/// Requires that `align` is a power of two.
const fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

impl MemoryState {
    #[track_caller]
    fn load_u8(&self, addr: u32) -> u8 {
        // debug!("load_u8: 0x{addr:08X}");
        // align to the nearest word
        let aligned = addr & !(WORD_SIZE as u32 - 1);
        let offset = addr % WORD_SIZE as u32;
        let word = self.load_u32(aligned);
        ((word >> (offset * 8)) & 0xff) as u8
    }

    #[track_caller]
    fn load_u32(&self, addr: u32) -> u32 {
        // debug!("load_u32: 0x{addr:08X}");
        assert_eq!(addr % WORD_SIZE as u32, 0, "unaligned load");
        let key = addr / 4;
        match self.memory.get(&key) {
            Some(word) => *word,
            None => panic!("addr out of range: 0x{addr:08X}"),
        }
    }

    #[track_caller]
    fn load_fp4(&self, addr: u32) -> Fp4 {
        let words = &[
            self.load_u32(addr + 0 * WORD_SIZE as u32),
            self.load_u32(addr + 1 * WORD_SIZE as u32),
            self.load_u32(addr + 2 * WORD_SIZE as u32),
            self.load_u32(addr + 3 * WORD_SIZE as u32),
        ];
        Fp4::from_u32_words(words)
    }

    #[track_caller]
    fn load_region_u32(&self, start: u32, size: u32) -> Vec<u32> {
        (start..start + size)
            .step_by(WORD_SIZE)
            .map(|addr| self.load_u32(addr))
            .collect()
    }

    #[track_caller]
    fn load_region_fp4(&self, start: u32, size: u32) -> Vec<Fp4> {
        debug!("load_region_fp4: 0x{start:08X}:{size}");
        (start..start + size)
            .step_by(WORD_SIZE * Fp4::WORDS)
            .map(|addr| self.load_fp4(addr))
            .collect()
    }

    #[track_caller]
    fn load_region(&self, addr: u32, size: u32) -> Vec<u8> {
        let mut region = Vec::new();
        for addr in addr..addr + size {
            region.push(self.load_u8(addr));
        }
        region
    }

    #[track_caller]
    fn store_u8(&mut self, addr: u32, value: u8) {
        // debug!("store_u8: 0x{addr:08X} <= 0x{value:08X}");
        // align to the nearest word
        let aligned = addr & !(WORD_SIZE as u32 - 1);
        let offset = addr % WORD_SIZE as u32;
        let key = aligned / 4;
        let mut word = self.memory.get(&key).unwrap_or(&0) & !(0xff << (offset * 8));
        word |= (value as u32) << (offset * 8);
        self.store_u32(aligned, word);
    }

    #[track_caller]
    fn store_u32(&mut self, addr: u32, value: u32) {
        // debug!("store_u32: 0x{addr:08X} <= 0x{value:08X}");
        assert_eq!(addr % WORD_SIZE as u32, 0, "unaligned store");
        let key = addr / 4;
        match self.memory.entry(key) {
            Entry::Occupied(mut entry) => {
                let min = MemoryEvent {
                    cycle: 0,
                    addr: key,
                    data: 0,
                    is_write: false,
                };
                let max = MemoryEvent {
                    cycle: 0,
                    addr: key + 1,
                    data: 0,
                    is_write: false,
                };
                if let Some(txn) = self.history.range(min..max).next() {
                    let last_value = *entry.get();
                    if txn.addr == key && last_value != value {
                        debug!("addr: 0x{key:08X}, value: 0x{value:08X}, txn: {txn:?}");
                        // The guest has actually touched this memory, and we are not writing the
                        // same value
                        panic!("Host cannot mutate existing memory");
                    }
                }
                entry.insert(value);
            }
            Entry::Vacant(entry) => {
                entry.insert(value);
            }
        }
    }

    #[track_caller]
    fn store_region(&mut self, addr: u32, slice: &[u8]) {
        // debug!("store_region: 0x{addr:08X} <= {} bytes", slice.len());
        for i in 0..slice.len() {
            self.store_u8(addr + i as u32, slice[i]);
        }
    }

    #[track_caller]
    fn store_region_u32(&mut self, addr: u32, slice: &[u32]) {
        assert!(addr % WORD_SIZE as u32 == 0);
        for (offset, word) in slice.iter().enumerate() {
            self.store_u32(addr + WORD_SIZE as u32 * offset as u32, *word);
        }
    }

    // Reads a C structure from a guest's memory and transmutes it
    // into the given structure, which should be repr(C) and plain old
    // data.
    unsafe fn read_descriptor<T: Send>(&self, addr: u32) -> T {
        let size = core::mem::size_of::<T>();
        assert_eq!(size % WORD_SIZE, 0, "Descriptors should be word aligned");
        let buf = self.load_region_u32(addr, size as u32);
        assert_eq!(buf.len() * WORD_SIZE, size);
        core::ptr::read_unaligned(buf.as_ptr() as *const T)
    }

    // Reads words from guest's memory and transmutes it into the given structure,
    // which should be repr(C) and plain old data.
    fn read_slice<T: Pod>(&self, desc: &SliceDescriptor) -> Vec<T> {
        let elt_size = core::mem::size_of::<T>();
        assert_eq!(elt_size % WORD_SIZE, 0, "T should be word aligned");
        let bytes = self.load_region(desc.addr, desc.size);
        bytes
            .chunks_exact(elt_size)
            .map(|chunk| *bytemuck::from_bytes(chunk))
            .collect()
    }

    fn read_slices(&self, desc: &SliceDescriptor) -> Vec<SliceDescriptor> {
        let elt_size = core::mem::size_of::<SliceDescriptor>();
        assert_eq!(elt_size % WORD_SIZE, 0, "T should be word aligned");
        let bytes = self.load_region(desc.addr, desc.size);
        bytes
            .chunks_exact(elt_size)
            .map(|chunk| {
                let size: u32 = *bytemuck::from_bytes(&chunk[..4]);
                let addr: u32 = *bytemuck::from_bytes(&chunk[4..]);
                SliceDescriptor { size, addr }
            })
            .collect()
    }

    // Reads words from guest's memory and transmutes it into the given value,
    // which should be repr(C) and plain old data.
    fn read_value<T: Pod>(&self, addr: u32) -> T {
        let size = core::mem::size_of::<T>();
        assert_eq!(size % WORD_SIZE, 0, "T should be word aligned");
        let bytes = self.load_region(addr, size as u32);
        *bytemuck::from_bytes(&bytes)
    }

    fn strlen(&self, addr: u32) -> usize {
        let mut addr = addr;
        let mut len = 0;
        while self.load_u8(addr) != 0 {
            addr += 1;
            len += 1;
        }
        len
    }

    fn pop_history(&mut self) -> MemoryEvent {
        let event = self
            .history
            .iter()
            .next()
            .expect("mem_check called on empty history")
            .clone();
        self.history.remove(&event);
        event
    }
}

pub struct MachineContext<'a, H: IoHandler> {
    memory: MemoryState,
    io: &'a mut H,
    cur_host_to_guest_offset: usize,
}

impl PartialOrd for MemoryEvent {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl Ord for MemoryEvent {
    fn cmp(&self, rhs: &Self) -> Ordering {
        match self.addr.cmp(&rhs.addr) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.cycle.cmp(&rhs.cycle)
    }
}

impl<'a, H: IoHandler> CircuitDef<MachineContext<'a, H>> for CircuitImpl {}

impl<'a, H: IoHandler> CustomStep for MachineContext<'a, H> {
    fn call(&mut self, name: &str, extra: &str, args: &[Fp], outs: &mut [Fp]) -> Result<()> {
        match name {
            "divide32" => {
                let ((x0, x1), (x2, x3)) = self.divide32((args[0], args[1]), (args[2], args[3]));
                outs[0] = x0;
                outs[1] = x1;
                outs[2] = x2;
                outs[3] = x3;
                Ok(())
            }
            "log" => {
                self.log(extra, args);
                Ok(())
            }
            "memCheck" => {
                let (x0, x1, x2, x3, x4) = self.mem_check();
                outs[0] = x0;
                outs[1] = x1;
                outs[2] = x2;
                outs[3] = x3;
                outs[4] = x4;
                Ok(())
            }
            "memRead" => {
                let (x0, x1) = self.mem_read(args[0], args[1]);
                outs[0] = x0;
                outs[1] = x1;
                Ok(())
            }
            "memWrite" => {
                self.mem_write(args[0], args[1], (args[2], args[3]))?;
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

impl MemoryState {
    fn new() -> Self {
        Self {
            memory: BTreeMap::new(),
            history: BTreeSet::new(),
        }
    }
}

fn split_word(value: u32) -> (Fp, Fp) {
    (Fp::new(value & 0xffff), Fp::new(value >> 16))
}

fn merge_word((low, high): (Fp, Fp)) -> u32 {
    let low: u32 = low.into();
    let high: u32 = high.into();
    low | high << 16
}

impl<'a, H: IoHandler> MachineContext<'a, H> {
    pub fn new(io: &'a mut H) -> Self {
        MachineContext {
            memory: MemoryState::new(),
            io,
            cur_host_to_guest_offset: INPUT.start(),
        }
    }

    fn divide32(&self, numer: (Fp, Fp), denom: (Fp, Fp)) -> ((Fp, Fp), (Fp, Fp)) {
        let numer = merge_word(numer);
        let denom = merge_word(denom);
        let (quot, rem) = if denom == 0 {
            (0xffffffff, numer)
        } else {
            (numer / denom, numer % denom)
        };
        (split_word(quot), split_word(rem))
    }

    fn log(&self, msg: &str, args: &[Fp]) {
        if log::max_level() < log::LevelFilter::Trace {
            // Don't bother to format it if we're not even logging.
            return;
        }

        // "msg" is given to us in C++-style formatting, so interpret it.
        let re = regex!("%([0-9]*)([xud])");
        let mut args_left = args;
        let formatted = re.replace_all(msg, |captures: &Captures| {
            let arg: u32 = args_left[0].into();
            args_left = &args_left[1..];
            let width = captures
                .get(1)
                .map_or(0, |x| x.as_str().parse::<usize>().unwrap_or(0));
            let format = captures.get(2).map_or("", |x| x.as_str());
            match format {
                "u" => format!("{:width$}", arg),
                "x" => format!("{:0width$x}", arg),
                "d" => format!("{:width$}", arg as i32),
                _ => panic!("Unhandled printf format specification '{format}'"),
            }
        });
        assert_eq!(
            args_left.len(),
            0,
            "Args missing formatting: {:?}",
            args_left
        );
        trace!("{}", formatted);
    }

    fn mem_check(&mut self) -> (Fp, Fp, Fp, Fp, Fp) {
        let event = self.memory.pop_history();
        let parts = split_word(event.data);
        // debug!("mem_check: {event:?}");
        (
            event.cycle.into(),
            event.addr.into(),
            if event.is_write { Fp::ONE } else { Fp::ZERO },
            parts.0,
            parts.1,
        )
    }

    fn mem_read(&mut self, cycle: Fp, addr: Fp) -> (Fp, Fp) {
        let cycle: u32 = cycle.into();
        let addr: u32 = addr.into();
        // debug!("[{}] R: 0x{:08X}", cycle, addr);
        let data = *self.memory.memory.entry(addr).or_insert(0);
        // debug!("data: 0x{data:08X}");
        self.memory.history.insert(MemoryEvent {
            cycle,
            addr,
            data,
            is_write: false,
        });
        split_word(data)
    }

    fn mem_write(&mut self, cycle: Fp, addr: Fp, value: (Fp, Fp)) -> Result<()> {
        let cycle: u32 = cycle.into();
        let addr: u32 = addr.into();
        let data = merge_word(value);
        let is_write = addr < (1 << (MEM_BITS - 1));
        // debug!("[{}] W: 0x{:08X} <= 0x{:08X}", cycle, addr, data);
        self.memory.history.insert(MemoryEvent {
            cycle,
            addr,
            data,
            is_write,
        });
        match self.memory.memory.entry(addr) {
            Entry::Occupied(mut entry) => {
                if *entry.get() != data && !is_write {
                    bail!(
                        "Double wrote write-once memory at 0x{:08X}. old: 0x{:08X}, new: 0x{:08X}",
                        addr * 4,
                        *entry.get(),
                        data
                    );
                }
                *entry.get_mut() = data;
            }
            Entry::Vacant(entry) => {
                entry.insert(data);
            }
        };
        self.on_write(cycle, addr * 4, data)?;
        Ok(())
    }

    // Writes to the guest's INPUT region.  Zero pads up to the next
    // word, and advances the host to guest offset.
    fn send_to_guest(&mut self, bytes: &[u8]) {
        let nwords = align_up(bytes.len(), WORD_SIZE);
        assert!(
            self.cur_host_to_guest_offset + nwords < INPUT.end(),
            "Read buffer overrun"
        );
        self.memory
            .store_region(self.cur_host_to_guest_offset as u32, bytes);
        self.cur_host_to_guest_offset += nwords
    }

    fn on_write(&mut self, cycle: u32, addr: u32, value: u32) -> Result<()> {
        // debug!("on_write: 0x{:08X}: 0x{:08X}", addr, value);
        match addr {
            GPIO_COMMIT => {
                debug!("on_write> GPIO_COMMIT, ptr = {value:08X}");
                // SAFETY: IoDescriptor is a plain-old-data type with
                // repr(C) and no pointers so it's safe to fill it from bytes.
                let desc: IoDescriptor = unsafe { self.memory.read_descriptor(value) };
                debug!(
                    "on_write> GPIO_COMMIT, commit region starts at {} and is {} bytes long",
                    desc.addr, desc.size
                );

                let buf = self.memory.load_region_u32(desc.addr, desc.size);
                debug!("Data: {:08X?}", &buf);
                self.io.on_commit(buf.as_slice())?;
            }
            GPIO_FAULT => {
                debug!("on_write> GPIO_FAULT");
                let len = self.memory.strlen(value);
                let buf = self.memory.load_region(value, len as u32);
                let str = String::from_utf8(buf).unwrap();
                self.io.on_fault(&str)?;
            }
            GPIO_GETKEY => {
                debug!("on_write> GPIO_GETKEY");
                todo!()
            }
            GPIO_LOG => {
                debug!("on_write> GPIO_LOG");
                let len = self.memory.strlen(value);
                let buf = self.memory.load_region(value, len as u32);
                let str = String::from_utf8(buf).unwrap();
                println!("R0VM[{cycle}]> {}", str);
            }
            GPIO_SENDRECV_ADDR => {
                debug!("on_write> GPIO_SENDRECV_ADDR");
                let channel = self.memory.load_u32(GPIO_SENDRECV_CHANNEL);
                let size = self.memory.load_u32(GPIO_SENDRECV_SIZE);
                let region = self.memory.load_region(value, size);
                let result = self.io.on_txrx(channel, &region)?;
                self.send_to_guest(bytemuck::cast_slice(&[result.len() as u32]));
                self.send_to_guest(result.as_slice())
            }
            GPIO_SHA => {
                debug!("on_write> GPIO_SHA, descriptor ptr = {value:08X}");
                // SAFETY: ShaDescriptor is a plain old repr(C)
                // structure and has no pointers.
                let desc: SHADescriptor = unsafe { self.memory.read_descriptor(value) };
                self.process_sha(&desc);
            }
            GPIO_CYCLECOUNT => {
                debug!("onWrite> GPIO_CycleCount, cycle = {cycle:08X}");

                assert_eq!(
                    value, 0,
                    "CycleCount request should only be written as zero"
                );
                self.send_to_guest(bytemuck::cast_slice(&[cycle as u32]));
            }
            GPIO_INSECURESHACOMPRESS => {
                debug!("onWrite> GPIO_InsecureShaCompress");
                // SAFETY: InsecureShaCompressDescriptor is a plain
                // old repr(C) structure and has no pointers.
                let desc: InsecureShaCompressDescriptor =
                    unsafe { self.memory.read_descriptor(value) };

                let sha = risc0_zkp::core::sha::default_implementation();

                const DIGEST_BYTES: u32 = (WORD_SIZE * DIGEST_WORDS) as u32;
                let state = self.memory.load_region_u32(desc.state, DIGEST_BYTES);
                let block_half1 = self.memory.load_region_u32(desc.block_half1, DIGEST_BYTES);
                let block_half2 = self.memory.load_region_u32(desc.block_half2, DIGEST_BYTES);

                let digest = sha.compress(
                    &Digest::from_slice(state.as_slice()),
                    &Digest::from_slice(block_half1.as_slice()),
                    &Digest::from_slice(block_half2.as_slice()),
                );
                self.send_to_guest(bytemuck::cast_slice(digest.as_slice()));
            }
            GPIO_INSECURESHAHASH => {
                debug!("onWrite> GPIO_InsecureShaHash");
                // SAFETY: InsecureShaHashDescriptor is a plain old
                // repr(C) structure and has no pointers.
                let desc: InsecureShaHashDescriptor = unsafe { self.memory.read_descriptor(value) };

                let sha = risc0_zkp::core::sha::default_implementation();

                let orig_state = self
                    .memory
                    .load_region_u32(desc.state, (WORD_SIZE * DIGEST_WORDS) as u32);
                let state: Digest = Digest::from_slice(orig_state.as_slice());
                let bytes = self.memory.load_region(desc.start, desc.len);

                let digest = sha.update(&state, bytes.as_slice());
                self.send_to_guest(bytemuck::cast_slice(digest.as_slice()));
            }
            GPIO_COMPUTE_POLY => {
                // SAFETY: ComputePolyDescriptor is a plain old
                // repr(C) structure and has no pointers.
                let desc: ComputePolyDescriptor = unsafe { self.memory.read_descriptor(value) };
                let eval_u: Vec<Fp4> = self.memory.read_slice(&desc.eval_u);
                let poly_mix = self.memory.read_value(desc.poly_mix);
                let out: Vec<Fp> = self.memory.read_slice(&desc.out);
                let mix: Vec<Fp> = self.memory.read_slice(&desc.mix);

                let ctx = PolyExtContext { mix: poly_mix };
                let args: &[&[Fp]] = &[&out, &mix];
                let result = CIRCUIT.poly_ext(&ctx, &eval_u, args);
                self.send_to_guest(bytemuck::bytes_of(&result.tot));
            }
            GPIO_POLY_EVAL => {
                // SAFETY: PolyEvalDescriptor is a plain old
                // repr(C) structure and has no pointers.
                let desc: PolyEvalDescriptor = unsafe { self.memory.read_descriptor(value) };
                let coeffs: Vec<Fp4> = self.memory.read_slice(&desc.coeffs);
                let x: Fp4 = self.memory.read_value(desc.x);
                let y: Fp = self.memory.read_value(desc.y);

                let mut mul_fp = Fp::ONE;
                let mut mul_fp4 = Fp4::ONE;
                let mut tot = Fp4::ZERO;
                for i in 0..coeffs.len() {
                    tot += coeffs[i] * mul_fp * mul_fp4;
                    mul_fp *= y;
                    mul_fp4 *= x;
                }
                self.send_to_guest(bytemuck::bytes_of(&tot));
            }
            GPIO_FFPU => {
                // SAFETY: FfpuDescriptor is a plain old
                // repr(C) structure and has no pointers.
                let desc: FfpuDescriptor = unsafe { self.memory.read_descriptor(value) };
                let code: Vec<u32> = self.memory.read_slice(&desc.code);
                log::debug!(
                    "GPIO_FFPU> code: 0x{:08X}:{}, args: 0x{:08X}:{}",
                    desc.code.addr,
                    desc.code.size,
                    desc.args.addr,
                    desc.args.size
                );
                let desc_args = self.memory.read_slices(&desc.args);
                let mut args: Vec<Vec<Fp4>> = desc_args
                    .iter()
                    .map(|desc| self.memory.load_region_fp4(desc.addr, desc.size))
                    .collect();
                let mut args: Vec<&mut [Fp4]> = args.iter_mut().map(|x| x.as_mut_slice()).collect();
                debug!("args[0]: {:?}", args[0]);
                debug!("args[1]: {:?}", args[1]);
                debug!("args[2]: {:?}", args[2]);
                debug!("args[3]: {:?}", args[3]);
                debug!("args[4]: {:?}", args[4]);
                ffpu_execute(&code, &mut args);
                debug!("args[4]: {:?}", args[4]);
                self.send_to_guest(bytemuck::cast_slice(&args[4]));
            }
            _ => {}
        };
        Ok(())
    }

    fn process_sha(&mut self, desc: &SHADescriptor) {
        let sha_type: u16 = ((desc.type_count & 0xFFFF) >> 4) as u16;
        let count: u16 = (desc.type_count & 0xFFFF) as u16;
        debug!(
            "SHA256 type: {}, count: {}, idx: {}, source: {:08X}, digest: {:08X}",
            sha_type, count, desc.idx, desc.source, desc.digest
        );

        let sha = risc0_zkp::core::sha::default_implementation();
        let words = self
            .memory
            .load_region_u32(desc.source as u32, (count * 64) as u32);
        let digest = sha.hash_raw_words(bytemuck::cast_slice(words.as_slice()));

        debug!("Digest result is {:X?}", digest.as_slice());

        self.memory
            .store_region_u32(desc.digest as u32, digest.as_slice());
    }
}

const SHA_ROUND: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn cond(expr: bool) -> Fp {
    Fp::new(if expr { 1 } else { 0 })
}

enum CodeIndex {
    Cycle,
    TypeNormal,
    TypeFinal,
    TypeInit,
    TypeLoad,
    TypeReset,
    TypeFini,
    ShaCtrl,
    ShaLoad,
    ShaMix,
    P1,
    P2,
    Data1Low,
    Data1High,
    Data2Low,
    Data2High,
}

const ZERO: Fp = Fp::new(0);
const ONE: Fp = Fp::new(1);

struct CodeRegisters(Vec<Fp>);

impl CodeRegisters {
    fn new() -> Self {
        Self(vec![ZERO; CIRCUIT.code_size()])
    }

    fn reset(&mut self) {
        self.0.fill(ZERO);
    }
}

impl Index<CodeIndex> for CodeRegisters {
    type Output = Fp;

    fn index(&self, index: CodeIndex) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<CodeIndex> for CodeRegisters {
    fn index_mut(&mut self, index: CodeIndex) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

struct CodeLoader<F>
where
    F: FnMut(&[Fp], usize) -> Result<bool>,
{
    cycle: usize,
    code: CodeRegisters,
    step: F,
}

impl<F> CodeLoader<F>
where
    F: FnMut(&[Fp], usize) -> Result<bool>,
{
    pub fn new(step: F) -> Self {
        Self {
            cycle: 0,
            code: CodeRegisters::new(),
            step,
        }
    }

    pub fn init(&mut self) -> Result<bool> {
        debug!("INIT");
        self.start();
        self.code[CodeIndex::TypeInit] = ONE;
        self.next()
    }

    pub fn load(&mut self, addr: u32, data: u32) -> Result<bool> {
        // debug!("LOAD: 0x{:08X} <= 0x{:08X}", addr, data);
        let (low, high) = split_word(data);
        self.start();
        self.code[CodeIndex::TypeLoad] = ONE;
        self.code[CodeIndex::P1] = Fp::new(addr / 4);
        self.code[CodeIndex::P2] = cond((addr / 4) >= (1 << (MEM_BITS - 1)));
        self.code[CodeIndex::Data1Low] = low;
        self.code[CodeIndex::Data1High] = high;
        self.next()
    }

    pub fn reset(&mut self, start_addr: u32) -> Result<bool> {
        debug!("RESET");
        self.start();
        self.code[CodeIndex::TypeReset] = ONE;
        self.code[CodeIndex::P1] = Fp::new(start_addr);
        self.next()
    }

    pub fn fini(&mut self) -> Result<bool> {
        debug!("FINI");
        self.start();
        self.code[CodeIndex::TypeFini] = ONE;
        self.next()
    }

    pub fn body(&mut self) -> Result<()> {
        let base_cycle = self.cycle;
        loop {
            self.start();

            let inst_phase = (self.cycle - base_cycle) % 3;
            if inst_phase == 2 {
                self.code[CodeIndex::TypeFinal] = ONE;
            } else {
                self.code[CodeIndex::TypeNormal] = ONE;
            }

            let sha_phase = (self.cycle - base_cycle) % 72;
            if sha_phase < 4 {
                let init1 = split_word(SHA256_INIT.get()[3 - sha_phase]);
                let init2 = split_word(SHA256_INIT.get()[7 - sha_phase]);
                self.code[CodeIndex::ShaCtrl] = ONE;
                self.code[CodeIndex::P1] = Fp::new(sha_phase as u32);
                self.code[CodeIndex::P2] = cond(sha_phase == 0);
                self.code[CodeIndex::Data1Low] = init1.0;
                self.code[CodeIndex::Data1High] = init1.1;
                self.code[CodeIndex::Data2Low] = init2.0;
                self.code[CodeIndex::Data2High] = init2.1;
            } else if sha_phase < 20 {
                let round = split_word(SHA_ROUND[sha_phase - 4]);
                self.code[CodeIndex::ShaLoad] = ONE;
                self.code[CodeIndex::Data1Low] = round.0;
                self.code[CodeIndex::Data1High] = round.1;
            } else if sha_phase < 68 {
                let round = split_word(SHA_ROUND[sha_phase - 4]);
                self.code[CodeIndex::ShaMix] = ONE;
                self.code[CodeIndex::P1] = cond(sha_phase >= 64);
                self.code[CodeIndex::P2] = cond(sha_phase == 67);
                self.code[CodeIndex::Data1Low] = round.0;
                self.code[CodeIndex::Data1High] = round.1;
            } else {
                self.code[CodeIndex::ShaCtrl] = ONE;
                self.code[CodeIndex::P1] = Fp::new((sha_phase - 68 + 4) as u32);
            }

            if !self.next_fini(1)? {
                break;
            }
        }

        Ok(())
    }

    fn start(&mut self) {
        self.code.reset();
        self.code[CodeIndex::Cycle] = Fp::new(self.cycle as u32);
    }

    fn next(&mut self) -> Result<bool> {
        self.cycle += 1;
        let keep_going = (self.step)(&self.code.0, 0)?;
        assert!(keep_going);
        Ok(keep_going)
    }

    fn next_fini(&mut self, fini: usize) -> Result<bool> {
        self.cycle += 1;
        (self.step)(&self.code.0, fini)
    }
}

pub fn load_code<F>(start_addr: u32, image: &BTreeMap<u32, u32>, step: F) -> Result<()>
where
    F: FnMut(&[Fp], usize) -> Result<bool>,
{
    let mut loader = CodeLoader::new(step);
    loader.init()?;
    for (addr, data) in image.iter() {
        loader.load(*addr, *data)?;
    }
    loader.reset(start_addr)?;
    loader.body()?;
    loader.fini()?;
    Ok(())
}

pub struct RV32Executor<'a, H: IoHandler> {
    elf: &'a Program,
    pub executor: Executor<CircuitImpl, MachineContext<'a, H>>,
}

impl<'a, H: IoHandler> RV32Executor<'a, H> {
    pub fn new(circuit: &'static CircuitImpl, elf: &'a Program, io: &'a mut H) -> Self {
        debug!("image.size(): {}", elf.image.len());
        let machine = MachineContext::new(io);
        let min_po2 = log2_ceil(elf.image.len() + 3 + ZK_CYCLES);
        let executor = Executor::new(circuit, machine, min_po2, MAX_CYCLES_PO2);
        Self { elf, executor }
    }

    pub fn run(&mut self) -> Result<()> {
        load_code(self.elf.entry, &self.elf.image, |chunk, fini| {
            self.executor.step(chunk, fini)
        })?;
        self.executor.finalize();
        Ok(())
    }
}
