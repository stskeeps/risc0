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

#include "risc0/zkvm/prove/method_id.h"

#include <fstream>
#include <iostream>
#include <map>

#include "risc0/core/elf.h"
#include "risc0/zkp/prove/poly_group.h"
#include "risc0/zkvm/prove/step.h"

namespace risc0 {

MethodId MethodId::fromElf(const uint8_t* bytes, const size_t len) {
  std::map<uint32_t, uint32_t> image;
  uint32_t startAddr = loadElf(bytes, len, kMemSize, image);

  // Start with an empty return value
  MethodDigest digest;

  // Make the digest for each level
  for (size_t i = 0; i < kCodeDigestCount; i++) {
    size_t cycles = kMinCycles * (1 << i);
    if (cycles < image.size() + 3 + kZkCycles) {
      // Can't even fit the program in this cycle size, just set to zero
      digest[i] = ShaDigest::zero();
      continue;
    }
    // Make a vector + set it up with the elf data
    std::vector<Fp> code(cycles * kCodeSize);
    setupCode(code.data(), cycles, startAddr, image);
    // Copy into accel buffer
    auto coeffs = AccelSlice<Fp>::copy(code);
    // Do interpolate + shift
    batchInterpolateNTT(coeffs, kCodeSize);
    zkShiftAccel(coeffs, kCodeSize);
    // Make the poly-group + extract the root
    PolyGroup codeGroup(coeffs, kCodeSize, cycles);
    digest[i] = codeGroup.getMerkle().getRoot();
  }
  return fromDigest(digest);
}

MethodId MethodId::fromElfFile(const std::string& elfPath) {
  std::vector<uint8_t> contents = loadFile(elfPath);
  return fromElf(contents.data(), contents.size());
}

MethodId MethodId::fromDigest(const MethodDigest& digest) {
  MethodId result;
  static_assert(sizeof(result.methodId) == sizeof(digest));
  memcpy(&result.methodId, &digest, sizeof(digest));
  return result;
}

MethodId MethodId::fromIdBytes(const uint8_t* bytes, size_t len) {
  MethodId result;
  if (len != sizeof(result.methodId)) {
    throw(std::runtime_error("Bad number of bytes in a method id"));
  }
  memcpy(&result.methodId, bytes, sizeof(result.methodId));
  return result;
}

std::array<uint8_t, sizeof(MethodId)> methodIdBytesFromElf(const uint8_t* bytes, const size_t len) {
  MethodId id = MethodId::fromElf(bytes, len);
  std::array<uint8_t, sizeof(MethodId)> idbytes;
  static_assert(sizeof(idbytes) == sizeof(id));
  memcpy(&idbytes, &id, sizeof(MethodId));
  return idbytes;
}

} // namespace risc0
