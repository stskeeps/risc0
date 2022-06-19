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

#include <memory>

#include "risc0/zkp/prove/prove.h"
#include "risc0/zkvm/prove/exec.h"

namespace risc0 {

std::unique_ptr<ProveCircuit>
getRiscVProveCircuit(const uint8_t* elfBytes, size_t elfLen, MemoryHandler& io);

} // namespace risc0
