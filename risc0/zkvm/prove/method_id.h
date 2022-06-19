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

#pragma once

#include <array>
#include <memory>

#include "risc0/core/archive.h"
#include "risc0/core/util.h"
#include "risc0/zkp/core/sha256.h"
#include "risc0/zkvm/circuit/constants.h"

namespace risc0 {

static constexpr size_t kCodeDigestCount = log2Ceil(kMaxCycles / kMinCycles) + 1;

// A MethodDigest is intended for internal use in verification
// A MethodId is an intentionally opaque version of a MethodDigest for use in APIs
using MethodDigest = std::array<ShaDigest, kCodeDigestCount>;

class MethodId {
public:
  MethodId() = default;

  static MethodId fromElf(const uint8_t* bytes, const size_t len);
  static MethodId fromDigest(const MethodDigest& digest);
  static MethodId fromElfFile(const std::string& elfPath);
  static MethodId fromIdBytes(const uint8_t* bytes, size_t len);

  MethodDigest asDigest() const {
    MethodDigest digest;
    static_assert(sizeof(methodId) == sizeof(digest));
    memcpy(&digest, &methodId, sizeof(digest));
    return digest;
  }

  template <typename Archive> void transfer(Archive& ar) { ar.transfer(methodId); }

private:
  std::array<uint8_t, sizeof(MethodDigest)> methodId;
};

// Convert an in-memory elf to bytes representing a method id all in one go;
// useful for FFIs.
std::array<uint8_t, sizeof(MethodId)> methodIdBytesFromElf(const uint8_t* bytes, const size_t len);

} // namespace risc0
