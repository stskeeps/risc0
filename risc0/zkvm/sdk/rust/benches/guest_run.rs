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

//! `guest_run' runs benchmarks on various guest tasks.  The purpose
//! of these benchmarks is to gather performance data on code compiled
//! for the guest, as opposed to performance data on the prover. As
//! such, they do not generate seals, and these performance numbers
//! are not indicitive of performance with cryptographically secure
//! proofs.

use std::time::{Duration, Instant};

use criterion::{
    black_box, criterion_group, criterion_main, Bencher, BenchmarkId, Criterion, SamplingMode,
    Throughput,
};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use risc0_zkvm::host::{Prover, ProverOpts};
use risc0_zkvm::serde::to_vec;
use risc0_zkvm_methods::{
    bench::{BenchmarkSpec, SpecWithIters},
    BENCH_ID, BENCH_PATH,
};

fn run_guest(spec: SpecWithIters) -> Duration {
    let input_data: Vec<u32> = to_vec(&spec).unwrap();

    let mut prover = Prover::new_with_opts(
        &std::fs::read(BENCH_PATH).unwrap(),
        BENCH_ID,
        ProverOpts::default().with_skip_seal(true),
    )
    .unwrap();
    prover.add_input_u32_slice(input_data.as_slice());

    let start = Instant::now();
    black_box(prover.run().unwrap());
    start.elapsed()
}

fn guest_iter(b: &mut Bencher, spec: BenchmarkSpec) {
    b.iter_custom(|iters| run_guest(SpecWithIters(spec.clone(), iters)))
}

fn rand_buffer<T>(len: usize) -> Vec<T>
where
    Standard: Distribution<T>,
{
    let mut rng = rand::thread_rng();
    let mut buf: Vec<T> = Vec::new();
    buf.resize_with(len as usize, || rng.gen());
    buf
}

pub fn bench(c: &mut Criterion) {
    c.bench_function("simple_loop", move |b| {
        guest_iter(b, BenchmarkSpec::SimpleLoop)
    });

    let mut sha_group = c.benchmark_group("raw_sha");
    sha_group
        .sampling_mode(SamplingMode::Flat)
        .measurement_time(Duration::new(20, 0));
    for buf_bytes in [0u64, 64, 512, 2048, 8192] {
        sha_group.throughput(Throughput::Bytes(buf_bytes));
        sha_group.bench_function(BenchmarkId::from_parameter(buf_bytes), |b| {
            let buf: Vec<u32> = rand_buffer((buf_bytes / 4) as usize);
            guest_iter(b, BenchmarkSpec::RawSha { buf: buf.clone() })
        });
    }
    sha_group.finish();

    let mut memset_group = c.benchmark_group("memset");
    memset_group.sampling_mode(SamplingMode::Flat);
    for buf_bytes in [32u64, 64, 128, 256, 512, 1024, 2048, 4096] {
        memset_group.bench_with_input(
            BenchmarkId::new("memset", buf_bytes),
            &buf_bytes,
            |b, buf_bytes| {
                guest_iter(
                    b,
                    BenchmarkSpec::Memset {
                        len: *buf_bytes as usize,
                    },
                )
            },
        );
    }
    memset_group.finish();

    let mut memcpy_group = c.benchmark_group("memcpy");
    memcpy_group.sampling_mode(SamplingMode::Flat);
    for buf_bytes in [32u64, 64, 128, 256, 512, 1024, 2048, 4096] {
        memcpy_group.throughput(Throughput::Bytes(buf_bytes));

        memcpy_group.bench_with_input(
            BenchmarkId::new("memcpy-aligned", buf_bytes),
            &buf_bytes,
            |b, buf_bytes| {
                let buf: Vec<u8> = rand_buffer(*buf_bytes as usize);

                guest_iter(
                    b,
                    BenchmarkSpec::Memcpy {
                        src: buf.clone(),
                        src_align: 0,
                        dst_align: 0,
                    },
                )
            },
        );

        memcpy_group.bench_with_input(
            BenchmarkId::new("memcpy-src-unaligned", buf_bytes),
            &buf_bytes,
            |b, buf_bytes| {
                let buf: Vec<u8> = rand_buffer(*buf_bytes as usize);

                guest_iter(
                    b,
                    BenchmarkSpec::Memcpy {
                        src: buf.clone(),
                        src_align: 1,
                        dst_align: 0,
                    },
                )
            },
        );

        memcpy_group.bench_with_input(
            BenchmarkId::new("memcpy-dst-unaligned", buf_bytes),
            &buf_bytes,
            |b, buf_bytes| {
                let buf: Vec<u8> = rand_buffer(*buf_bytes as usize);

                guest_iter(
                    b,
                    BenchmarkSpec::Memcpy {
                        src: buf.clone(),
                        src_align: 0,
                        dst_align: 3,
                    },
                )
            },
        );

        memcpy_group.bench_with_input(
            BenchmarkId::new("memcpy-both-unaligned", buf_bytes),
            &buf_bytes,
            |b, buf_bytes| {
                let buf: Vec<u8> = rand_buffer(*buf_bytes as usize);

                guest_iter(
                    b,
                    BenchmarkSpec::Memcpy {
                        src: buf.clone(),
                        src_align: 1,
                        dst_align: 3,
                    },
                )
            },
        );
    }
    memcpy_group.finish();
}

criterion_group!(name = benches;
                 config = Criterion::default();
                 targets = bench);
criterion_main!(benches);
