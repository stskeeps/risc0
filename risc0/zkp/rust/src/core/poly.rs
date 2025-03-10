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

//! Polynomial utilities (currently only those used in polynomial evaluation).

use alloc::vec;

use super::fp4::Fp4;
use crate::field::Elem;

/// Evaluate a polynomial whose coefficients are in the extension field at a
/// point.
pub fn poly_eval(coeffs: &[Fp4], x: Fp4) -> Fp4 {
    let mut mul = Fp4::ONE;
    let mut tot = Fp4::ZERO;
    for i in 0..coeffs.len() {
        tot += coeffs[i] * mul;
        mul *= x;
    }
    tot
}

/// General purpose polynomial interpolation.
///
/// Given the goal value f(x) at a set of evalation points x, compute
/// coefficients.
pub fn poly_interpolate(out: &mut [Fp4], x: &[Fp4], fx: &[Fp4], size: usize) {
    // Special case the very easy ones
    if size == 1 {
        out[0] = fx[0];
        return;
    }
    if size == 2 {
        out[1] = (fx[1] - fx[0]) * (x[1] - x[0]).inv();
        out[0] = fx[0] - out[1] * x[0];
        return;
    }
    // Compute ft = product of (x - x_i) for all i
    let mut ft = vec![Fp4::ZERO; size + 1];
    ft[0] = Fp4::ONE;
    for i in 0..size {
        for j in (0..i + 1).rev() {
            let value = ft[j];
            ft[j + 1] += value;
            ft[j] *= -x[i];
        }
    }
    // Clear output
    for i in 0..size {
        out[i] = Fp4::ZERO;
    }
    for i in 0..size {
        // Compute fr = ft / (x - x_i)
        let mut fr = ft.clone();
        poly_divide(&mut fr, x[i]);
        // Evaluate at x[i]
        let fr_xi = poly_eval(&fr, x[i]);
        // Compute multiplier (fx[i] / fr_xi)
        let mul = fx[i] * fr_xi.inv();
        // Multiply into output
        for j in 0..size {
            out[j] += mul * fr[j];
        }
    }
}

/// In-place polynomial division.
///
/// Take the coefficients in P, and divide by (X - z) for some z, return the
/// remainder.
pub fn poly_divide(p: &mut [Fp4], z: Fp4) -> Fp4 {
    let mut cur = Fp4::ZERO;
    for i in (0..p.len()).rev() {
        let next = z * cur + p[i];
        p[i] = cur;
        cur = next;
    }
    cur
}
