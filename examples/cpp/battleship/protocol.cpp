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

#include "protocol.h"

#include "risc0/core/log.h"
#include "risc0/zkp/core/sha256_cpu.h"

using risc0::hex;

InitMessage::Content InitMessage::decode() const {
  risc0::ReceiptReader reader(receipt);
  return reader.read<Content>();
}

RoundMessage::Content RoundMessage::decode() const {
  risc0::ReceiptReader reader(receipt);
  return reader.read<Content>();
}

InitMessage Battleship::init() {
  risc0::MethodId methodId = risc0::MethodId::fromElfFile("examples/cpp/battleship/init_method");
  risc0::Prover prover("examples/cpp/battleship/init_method", methodId);
  prover.writeInput(state);
  risc0::Receipt receipt = prover.run();
  LOG(1, name << "> InitMethod: " << receipt.seal.size());
  return InitMessage{receipt};
}

void Battleship::onInitMsg(const InitMessage& msg) {
  LOG(1, name << "> onInitMsg");
  risc0::MethodId methodId = risc0::MethodId::fromElfFile("examples/cpp/battleship/init_method");
  msg.receipt.verify(methodId);
  InitMessage::Content content = msg.decode();
  peer_state = content.state;
  LOG(1, name << "> peer_state: " << peer_state);
}

TurnMessage Battleship::turn(const Position& shot) {
  LOG(1, name << "> turn: " << shot);
  last_shot = shot;
  return TurnMessage{shot};
}

RoundMessage Battleship::onTurnMsg(const TurnMessage& msg) {
  LOG(1, name << "> onTurnMsg");
  RoundParams params{state, msg.shot};
  risc0::MethodId methodId = risc0::MethodId::fromElfFile("examples/cpp/battleship/turn_method");
  risc0::Prover prover("examples/cpp/battleship/turn_method", methodId);
  prover.writeInput(params);
  risc0::Receipt receipt = prover.run();
  LOG(1, name << "> RoundMethod: " << receipt.seal.size());
  const RoundResult& round = prover.readOutput<RoundResult>();
  LOG(1, name << "> RoundResult: " << round);
  state = round.state;
  return RoundMessage{receipt};
}

void Battleship::onRoundMsg(const RoundMessage& msg) {
  LOG(1, name << "> onRoundMsg");
  risc0::MethodId methodId = risc0::MethodId::fromElfFile("examples/cpp/battleship/turn_method");
  msg.receipt.verify(methodId);
  RoundMessage::Content content = msg.decode();

  if (content.old_state != peer_state) {
    throw new std::runtime_error("Cheater: state mismatch");
  }

  if (content.shot != last_shot) {
    throw new std::runtime_error("Cheater: shot mismatch");
  }

  LOG(0, name << "> shot: " << content.shot << " -> hit: " << content.hit);

  peer_state = content.new_state;
}
