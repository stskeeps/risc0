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

mod fri;
mod merkle;
mod poly_group;
pub(crate) mod write_iop;

use alloc::{vec, vec::Vec};

use array_init::array_init;
use risc0_zkp_core::{
    fp::Fp,
    fp4::{Fp4, EXT_SIZE},
    poly::{poly_divide, poly_interpolate},
    rou::ROU_REV,
    sha::Sha,
    Random,
};
use risc0_zkp_hal::{Buffer, Hal};

use crate::zkp::{
    prove::{fri::fri_prove, poly_group::PolyGroup, write_iop::WriteIOP},
    taps::{RegisterGroup, Taps},
    CHECK_SIZE, INV_RATE, MAX_CYCLES_PO2,
};

pub trait Circuit {
    fn taps(&self) -> &'static Taps<'static>;

    /// Perform initial 'execution' setting code + data.
    /// Additionally, write any 'results' as needed.
    fn execute<S: Sha>(&mut self, iop: &mut WriteIOP<S>);

    /// Perform 'accumlate' stage, using the iop for any RNG state.
    fn accumulate<S: Sha>(&mut self, iop: &mut WriteIOP<S>);

    /// Compute check polynomial.
    fn eval_check(
        &self,
        check: &Buffer<Fp>,
        code: &Buffer<Fp>,
        data: &Buffer<Fp>,
        accum: &Buffer<Fp>,
        poly_mix: Fp4,
    );

    fn po2(&self) -> u32;

    fn get_code(&self) -> &[Fp];

    fn get_data(&self) -> &[Fp];

    fn get_accum(&self) -> &[Fp];
}

pub fn prove<H: Hal, S: Sha, C: Circuit>(hal: &H, sha: &S, circuit: &mut C) -> Vec<u32> {
    let taps = circuit.taps();
    let taps_info = taps.get_info();
    let mut iop = WriteIOP::new(sha);

    circuit.execute(&mut iop);

    let po2 = circuit.po2();
    assert!(po2 as usize <= MAX_CYCLES_PO2);
    let size = 1 << po2;

    // Make code + data PolyGroups + commit them
    let code_group = PolyGroup::new(
        hal,
        &make_coeffs(hal, circuit.get_code(), taps_info.code_size),
        taps_info.code_size,
        size,
    );
    code_group.merkle.commit(hal, &mut iop);
    //   LOG(1, "codeGroup: " << codeGroup.getMerkle().getRoot());

    let data_group = PolyGroup::new(
        hal,
        &make_coeffs(hal, circuit.get_data(), taps_info.data_size),
        taps_info.data_size,
        size,
    );
    data_group.merkle.commit(hal, &mut iop);
    //   LOG(1, "dataGroup: " << dataGroup.getMerkle().getRoot());

    circuit.accumulate(&mut iop);

    // Make the accum group + commit
    //   LOG(1, "size = " << size << ", accumSize = " << accumSize);
    //   LOG(1, "getAccum.size() = " << circuit.getAccum().size());
    let accum_group = PolyGroup::new(
        hal,
        &make_coeffs(hal, circuit.get_accum(), taps_info.accum_size),
        taps_info.accum_size,
        size,
    );
    accum_group.merkle.commit(hal, &mut iop);
    //   LOG(1, "accumGroup: " << accumGroup.getMerkle().getRoot());

    // Set the poly mix value
    let poly_mix = Fp4::random(&mut iop.rng);

    // Now generate the check polynomial
    let domain = size * INV_RATE;
    let check_poly = hal.alloc(EXT_SIZE * domain);
    circuit.eval_check(
        &check_poly,
        &code_group.evaluated,
        &data_group.evaluated,
        &accum_group.evaluated,
        poly_mix,
    );

    // #ifdef CIRCUIT_DEBUG
    //   Fp4 badZ; // = Fp4(1);
    //   {
    //     AccelReadLock<Fp> lock(checkPoly);
    //     for (size_t i = 0; i < 4 * domain; i += 4) {
    //       if (lock[i] != 0) {
    //         LOG(1, "ERROR AT i = " << i << " value = " << lock[i]);
    //         badZ = Fp4(pow(kRouFwd[po2], i / 4));
    //         break;
    //       }
    //     }
    //   }
    // #endif

    // Convert to coefficients.  Some tricky bizness here with the fact that
    // checkPoly is really an Fp4 polynomial.  Nicely for us, since all the
    // roots of unity (which are the only thing that and values get multiplied
    // by) are in Fp, Fp4 values act like simple vectors of Fp for the
    // purposes of interpolate/evaluate.
    hal.batch_interpolate_ntt(&check_poly, EXT_SIZE);

    // The next step is to convert the degree 4*n check polynomial into 4 degreen n
    // polynomials so that f(x) = g0(x^4) + g1(x^4) x + g2(x^4) x^2 + g3(x^4)
    // x^3.  To do this, we normally would grab all the coeffients of f(x) =
    // sum_i c_i x^i where i % 4 == 0 and put them into a new polynomial g0(x) =
    // sum_i d0_i*x^i, where d0_i = c_(i*4).
    //
    // Amazingingly, since the coefficients are bit reversed, the coefficients of g0
    // are all aleady next to each other and in bit-reversed for for g0, as are
    // the coeffients of g1, etc. So really, we can just reinterpret 4 polys of
    // invRate*size to 16 polys of size, without actually doing anything.

    // Make the PolyGroup + add it to the IOP;
    let check_group = PolyGroup::new(hal, &check_poly, CHECK_SIZE, size);
    check_group.merkle.commit(hal, &mut iop);
    //   LOG(1, "checkGroup: " << checkGroup.getMerkle().getRoot());

    // Now pick a value for Z
    let z = Fp4::random(&mut iop.rng);
    // #ifdef CIRCUIT_DEBUG
    //   if (badZ != Fp4(0)) {
    //     Z = badZ;
    //   }
    //   iop.write(&Z, 1);
    // #endif
    //   LOG(1, "Z = " << Z);

    // Get rev rou for size
    let back_one = Fp4::from_u32(ROU_REV[po2 as usize]);
    let mut all_xs = Vec::new();

    // Do evaluations of all of the various polynomials at the appropriate points.
    let mut eval_u: Vec<Fp4> = Vec::new();
    let mut eval_group = |id: RegisterGroup, pg: &PolyGroup| {
        let mut which = Vec::new();
        let mut xs = Vec::new();
        for reg in taps.registers.iter().take_while(|x| x.group == id) {
            for tap in reg.back {
                which.push(reg.offset as u32);
                let x = back_one.pow(*tap) * z;
                xs.push(x);
                all_xs.push(x);
            }
        }
        let which = hal.copy_from(which.as_slice());
        let xs = hal.copy_from(xs.as_slice());
        let out = hal.alloc(which.size());
        hal.batch_evaluate_any(&pg.coeffs, pg.count, &which, &xs, &out);
        out.view(&mut |view| {
            eval_u.extend(view);
        });
    };

    // Now, we evaluate each group at the approriate points (relative to Z).
    // From here on out, we always process groups in accum, code, data order,
    // since this is the order used by the codegen system (alphabetical).
    // Sometimes it's a requirement for matching generated code, but even when
    // it's not we keep the order for consistency.
    eval_group(RegisterGroup::Accum, &accum_group);
    eval_group(RegisterGroup::Code, &code_group);
    eval_group(RegisterGroup::Data, &data_group);

    // Now, convert the values to coefficients via interpolation
    let mut pos = 0;
    let mut coeff_u = vec![Fp4::default(); eval_u.len()];
    for reg in taps.registers {
        let size = reg.back.len();
        poly_interpolate(&mut coeff_u[pos..], &all_xs[pos..], &eval_u[pos..], size);
        pos += size;
    }

    // Add in the coeffs of the check polynomials.
    let z4 = z.pow(EXT_SIZE);
    let which: [u32; CHECK_SIZE] = array_init(|i| i as u32);
    let xs = [z4; CHECK_SIZE];
    let out = hal.alloc(CHECK_SIZE);
    let which = hal.copy_from(which.as_slice());
    let xs = hal.copy_from(xs.as_slice());
    hal.batch_evaluate_any(&check_group.coeffs, CHECK_SIZE, &which, &xs, &out);
    out.view(&mut |view| {
        coeff_u.extend(view);
    });

    //   LOG(1, "Size of U = " << coeffU.size());
    iop.write_fp4_slice(&coeff_u);
    let hash_u = sha.hash_fp4s(&coeff_u);
    iop.commit(&hash_u);

    // Set the mix mix value
    let mix = Fp4::random(&mut iop.rng);
    //   LOG(1, "Mix = " << mix);

    // Do the coefficent mixing
    // Begin by making a zeroed output buffer
    let combo_count = taps.combos.len();
    let combos = vec![Fp4::default(); combo_count + 1];
    let combos = hal.copy_from(combos.as_slice());
    let mut cur_mix = Fp4::one();

    let mut mix_group = |id: RegisterGroup, pg: &PolyGroup| {
        let mut which = Vec::new();
        for reg in taps.registers.iter().take_while(|x| x.group == id) {
            which.push(reg.combo_id as u32);
        }
        let which_buf = hal.copy_from(which.as_slice());
        let cur_mix_buf = hal.copy_from([cur_mix].as_slice());
        let mix_buf = hal.copy_from([mix].as_slice());
        hal.mix_poly_coeffs(
            &combos,
            &cur_mix_buf,
            &mix_buf,
            &pg.coeffs,
            &which_buf,
            which.len(),
            size,
        );
        cur_mix *= mix.pow(which.len());
    };

    mix_group(RegisterGroup::Accum, &accum_group);
    mix_group(RegisterGroup::Code, &code_group);
    mix_group(RegisterGroup::Data, &data_group);

    let which = [combo_count as u32; CHECK_SIZE];
    let which_buf = hal.copy_from(which.as_slice());
    let cur_mix_buf = hal.copy_from([cur_mix].as_slice());
    let mix_buf = hal.copy_from([mix].as_slice());
    hal.mix_poly_coeffs(
        &combos,
        &cur_mix_buf,
        &mix_buf,
        &check_group.coeffs,
        &which_buf,
        CHECK_SIZE,
        size,
    );

    // Load the near final coefficients back to the CPU
    combos.view_mut(&mut |combos| {
        // Subtract the U coeffs from the combos
        let mut cur_pos = 0;
        let mut cur = Fp4::one();
        for reg in taps.registers {
            for i in 0..reg.back.len() {
                combos[size * reg.combo_id + i] -= cur * coeff_u[cur_pos + i];
            }
            cur *= mix;
            cur_pos += reg.back.len();
        }
        // Subtract the final 'check' coefficents
        for i in 0..CHECK_SIZE {
            combos[size * combo_count] -= cur * coeff_u[cur_pos];
            cur_pos += 1;
            cur *= mix;
        }
        // Divide each element by (x - Z * back1^back) for each back
        for combo in 0..taps.combos.len() {
            for back in taps.combos[combo].back {
                assert_eq!(
                    poly_divide(&mut combos[combo * size..size], z * back_one.pow(*back)),
                    Fp4::zero()
                );
            }
        }
        // Divide check polys by Z4
        assert_eq!(
            poly_divide(&mut combos[combo_count * size..size], z4),
            Fp4::zero()
        );
    });

    // Sum the combos up into one final polynomial + make then make it into 4 Fp
    // polys Additionally, it needs to be bit reversed to make everyone happy
    let final_poly_coeffs = hal.alloc(size * EXT_SIZE);
    hal.eltwise_sum_fp4(&final_poly_coeffs, &combos);

    // Finally do the FRI protocol to prove the degree of the polynomial
    hal.batch_bit_reverse(&final_poly_coeffs, EXT_SIZE);
    //   LOG(1, "FRI-proof, size = " << finalPolyCoeffs.size() / 4);

    fri_prove(hal, &mut iop, &final_poly_coeffs, |iop, idx| {
        accum_group.merkle.prove(iop, idx);
        code_group.merkle.prove(iop, idx);
        data_group.merkle.prove(iop, idx);
        check_group.merkle.prove(iop, idx);
    });

    // Return final proof
    let proof = iop.proof;
    //   LOG(1, "Proof size = " << ret.size());
    proof
}

fn make_coeffs<H: Hal>(hal: &H, input: &[Fp], count: usize) -> Buffer<Fp> {
    // Copy into accel buffer
    let slice = hal.copy_from(input);
    // Do interpolate
    hal.batch_interpolate_ntt(&slice, count);
    // Convert f(x) -> f(3x), which effective multiplies cofficent c_i by 3^i.
    #[cfg(not(circuit_debug))]
    hal.zk_shift(&slice, count);
    slice
}
