// This code is automatically generated

#include "ffi.h"
#include "fp.h"

#include <array>
#include <stdexcept>

// clang-format off
namespace risc0::circuit::rv32im {

Fp step_verify(void* ctx, HostBridge host, size_t steps, size_t cycle, Fp** args) {
  size_t mask = steps - 1;
  std::array<Fp, 32> host_args;
  std::array<Fp, 5> host_outs;
  Fp x0(2);
  Fp x1(3);
  Fp x2(4);
  Fp x3(1509949441);
  Fp x4(16);
  Fp x5(64);
  Fp x6(256);
  Fp x7(1024);
  Fp x8(4096);
  Fp x9(16384);
  Fp x10(65536);
  Fp x11(262144);
  Fp x12(1048576);
  Fp x13(4194304);
  Fp x14(2013265801);
  Fp x15(1);
  Fp x16 = args[0][0 * steps + ((cycle - 0) & mask)];
  Fp x17 = args[0][3 * steps + ((cycle - 0) & mask)];
  Fp x18 = args[0][4 * steps + ((cycle - 0) & mask)];
  Fp x19 = args[0][5 * steps + ((cycle - 0) & mask)];
  Fp x20 = args[0][6 * steps + ((cycle - 0) & mask)];
  Fp x21 = args[0][1 * steps + ((cycle - 0) & mask)];
  Fp x22 = args[0][2 * steps + ((cycle - 0) & mask)];
  Fp x23 = x21 + x22;
  Fp x24 = x23 + x17;
  Fp x25 = x24 + x18;
  Fp x26 = x25 + x19;
  Fp x27 = x26 + x20;
  if (x27 != 0) {
    {
      host(ctx, "memCheck", "", host_args.data(), 0, host_outs.data(), 5);
      Fp x28 = host_outs[0];
      Fp x29 = host_outs[1];
      Fp x30 = host_outs[2];
      Fp x31 = host_outs[3];
      Fp x32 = host_outs[4];
      args[2][148 * steps + cycle] = x28;
      args[2][144 * steps + cycle] = x29;
      args[2][147 * steps + cycle] = x30;
      args[2][145 * steps + cycle] = x31;
      args[2][146 * steps + cycle] = x32;
    }
    if (x16 != 0) {
      Fp x33 = args[2][144 * steps + ((cycle - 0) & mask)];
      Fp x34 = args[2][144 * steps + ((cycle - 1) & mask)];
      {
        Fp x35 = x33 - x34;
        Fp x36 = (x35 == 0) ? Fp(1) : Fp(0);
        Fp x37 = x15 - x36;
        Fp x38 = x15 - x37;
        args[2][149 * steps + cycle] = x38;
      }
      Fp x39 = args[2][149 * steps + ((cycle - 0) & mask)];
      if (x39 != 0) {
        Fp x40 = x33 - x34;
        if (x40 != 0) throw std::runtime_error("eqz failed at: external/risc0/risc0/zkvm/circuit/mem_check.cpp:41");
        Fp x41 = args[2][147 * steps + ((cycle - 0) & mask)];
        Fp x42 = x15 - x41;
        if (x42 != 0) {
          Fp x43 = args[2][145 * steps + ((cycle - 0) & mask)];
          Fp x44 = args[2][145 * steps + ((cycle - 1) & mask)];
          Fp x45 = x43 - x44;
          if (x45 != 0) throw std::runtime_error("eqz failed at: external/risc0/risc0/zkvm/circuit/mem_check.cpp:43");
          Fp x46 = args[2][146 * steps + ((cycle - 0) & mask)];
          Fp x47 = args[2][146 * steps + ((cycle - 1) & mask)];
          Fp x48 = x46 - x47;
          if (x48 != 0) throw std::runtime_error("eqz failed at: external/risc0/risc0/zkvm/circuit/mem_check.cpp:44");
        }
        Fp x49 = args[2][148 * steps + ((cycle - 0) & mask)];
        Fp x50 = args[2][148 * steps + ((cycle - 1) & mask)];
        Fp x51 = x49 - x50;
        Fp x52 = x51 - x15;
        {
          Fp x53 = Fp(x52.asUInt32() & x1.asUInt32());
          args[2][150 * steps + cycle] = x53;
          Fp x54 = x52 - x53;
          Fp x55 = x54 * x3;
          Fp x56 = Fp(x55.asUInt32() & x1.asUInt32());
          args[2][151 * steps + cycle] = x56;
          Fp x57 = x55 - x56;
          Fp x58 = x57 * x3;
          Fp x59 = Fp(x58.asUInt32() & x1.asUInt32());
          args[2][152 * steps + cycle] = x59;
          Fp x60 = x58 - x59;
          Fp x61 = x60 * x3;
          Fp x62 = Fp(x61.asUInt32() & x1.asUInt32());
          args[2][153 * steps + cycle] = x62;
          Fp x63 = x61 - x62;
          Fp x64 = x63 * x3;
          Fp x65 = Fp(x64.asUInt32() & x1.asUInt32());
          args[2][154 * steps + cycle] = x65;
          Fp x66 = x64 - x65;
          Fp x67 = x66 * x3;
          Fp x68 = Fp(x67.asUInt32() & x1.asUInt32());
          args[2][155 * steps + cycle] = x68;
          Fp x69 = x67 - x68;
          Fp x70 = x69 * x3;
          Fp x71 = Fp(x70.asUInt32() & x1.asUInt32());
          args[2][156 * steps + cycle] = x71;
          Fp x72 = x70 - x71;
          Fp x73 = x72 * x3;
          Fp x74 = Fp(x73.asUInt32() & x1.asUInt32());
          args[2][157 * steps + cycle] = x74;
          Fp x75 = x73 - x74;
          Fp x76 = x75 * x3;
          Fp x77 = Fp(x76.asUInt32() & x1.asUInt32());
          args[2][158 * steps + cycle] = x77;
          Fp x78 = x76 - x77;
          Fp x79 = x78 * x3;
          Fp x80 = Fp(x79.asUInt32() & x1.asUInt32());
          args[2][159 * steps + cycle] = x80;
          Fp x81 = x79 - x80;
          Fp x82 = x81 * x3;
          Fp x83 = Fp(x82.asUInt32() & x1.asUInt32());
          args[2][160 * steps + cycle] = x83;
          Fp x84 = x82 - x83;
          Fp x85 = x84 * x3;
          Fp x86 = Fp(x85.asUInt32() & x1.asUInt32());
          args[2][161 * steps + cycle] = x86;
        }
        Fp x87 = args[2][150 * steps + ((cycle - 0) & mask)];
        Fp x88 = args[2][151 * steps + ((cycle - 0) & mask)];
        Fp x89 = x88 * x2;
        Fp x90 = x87 + x89;
        Fp x91 = args[2][152 * steps + ((cycle - 0) & mask)];
        Fp x92 = x91 * x4;
        Fp x93 = x90 + x92;
        Fp x94 = args[2][153 * steps + ((cycle - 0) & mask)];
        Fp x95 = x94 * x5;
        Fp x96 = x93 + x95;
        Fp x97 = args[2][154 * steps + ((cycle - 0) & mask)];
        Fp x98 = x97 * x6;
        Fp x99 = x96 + x98;
        Fp x100 = args[2][155 * steps + ((cycle - 0) & mask)];
        Fp x101 = x100 * x7;
        Fp x102 = x99 + x101;
        Fp x103 = args[2][156 * steps + ((cycle - 0) & mask)];
        Fp x104 = x103 * x8;
        Fp x105 = x102 + x104;
        Fp x106 = args[2][157 * steps + ((cycle - 0) & mask)];
        Fp x107 = x106 * x9;
        Fp x108 = x105 + x107;
        Fp x109 = args[2][158 * steps + ((cycle - 0) & mask)];
        Fp x110 = x109 * x10;
        Fp x111 = x108 + x110;
        Fp x112 = args[2][159 * steps + ((cycle - 0) & mask)];
        Fp x113 = x112 * x11;
        Fp x114 = x111 + x113;
        Fp x115 = args[2][160 * steps + ((cycle - 0) & mask)];
        Fp x116 = x115 * x12;
        Fp x117 = x114 + x116;
        Fp x118 = args[2][161 * steps + ((cycle - 0) & mask)];
        Fp x119 = x118 * x13;
        Fp x120 = x117 + x119;
        Fp x121 = x52 - x120;
        Fp x122 = x121 * x14;
        if (x122 != 0) throw std::runtime_error("eqz failed at: external/risc0/risc0/zkvm/circuit/mem_check.cpp:46");
      }
      Fp x123 = x15 - x39;
      if (x123 != 0) {
        Fp x124 = x33 - x34;
        Fp x125 = x124 - x15;
        {
          Fp x126 = Fp(x125.asUInt32() & x1.asUInt32());
          args[2][150 * steps + cycle] = x126;
          Fp x127 = x125 - x126;
          Fp x128 = x127 * x3;
          Fp x129 = Fp(x128.asUInt32() & x1.asUInt32());
          args[2][151 * steps + cycle] = x129;
          Fp x130 = x128 - x129;
          Fp x131 = x130 * x3;
          Fp x132 = Fp(x131.asUInt32() & x1.asUInt32());
          args[2][152 * steps + cycle] = x132;
          Fp x133 = x131 - x132;
          Fp x134 = x133 * x3;
          Fp x135 = Fp(x134.asUInt32() & x1.asUInt32());
          args[2][153 * steps + cycle] = x135;
          Fp x136 = x134 - x135;
          Fp x137 = x136 * x3;
          Fp x138 = Fp(x137.asUInt32() & x1.asUInt32());
          args[2][154 * steps + cycle] = x138;
          Fp x139 = x137 - x138;
          Fp x140 = x139 * x3;
          Fp x141 = Fp(x140.asUInt32() & x1.asUInt32());
          args[2][155 * steps + cycle] = x141;
          Fp x142 = x140 - x141;
          Fp x143 = x142 * x3;
          Fp x144 = Fp(x143.asUInt32() & x1.asUInt32());
          args[2][156 * steps + cycle] = x144;
          Fp x145 = x143 - x144;
          Fp x146 = x145 * x3;
          Fp x147 = Fp(x146.asUInt32() & x1.asUInt32());
          args[2][157 * steps + cycle] = x147;
          Fp x148 = x146 - x147;
          Fp x149 = x148 * x3;
          Fp x150 = Fp(x149.asUInt32() & x1.asUInt32());
          args[2][158 * steps + cycle] = x150;
          Fp x151 = x149 - x150;
          Fp x152 = x151 * x3;
          Fp x153 = Fp(x152.asUInt32() & x1.asUInt32());
          args[2][159 * steps + cycle] = x153;
          Fp x154 = x152 - x153;
          Fp x155 = x154 * x3;
          Fp x156 = Fp(x155.asUInt32() & x1.asUInt32());
          args[2][160 * steps + cycle] = x156;
          Fp x157 = x155 - x156;
          Fp x158 = x157 * x3;
          Fp x159 = Fp(x158.asUInt32() & x1.asUInt32());
          args[2][161 * steps + cycle] = x159;
        }
        Fp x160 = args[2][150 * steps + ((cycle - 0) & mask)];
        Fp x161 = args[2][151 * steps + ((cycle - 0) & mask)];
        Fp x162 = x161 * x2;
        Fp x163 = x160 + x162;
        Fp x164 = args[2][152 * steps + ((cycle - 0) & mask)];
        Fp x165 = x164 * x4;
        Fp x166 = x163 + x165;
        Fp x167 = args[2][153 * steps + ((cycle - 0) & mask)];
        Fp x168 = x167 * x5;
        Fp x169 = x166 + x168;
        Fp x170 = args[2][154 * steps + ((cycle - 0) & mask)];
        Fp x171 = x170 * x6;
        Fp x172 = x169 + x171;
        Fp x173 = args[2][155 * steps + ((cycle - 0) & mask)];
        Fp x174 = x173 * x7;
        Fp x175 = x172 + x174;
        Fp x176 = args[2][156 * steps + ((cycle - 0) & mask)];
        Fp x177 = x176 * x8;
        Fp x178 = x175 + x177;
        Fp x179 = args[2][157 * steps + ((cycle - 0) & mask)];
        Fp x180 = x179 * x9;
        Fp x181 = x178 + x180;
        Fp x182 = args[2][158 * steps + ((cycle - 0) & mask)];
        Fp x183 = x182 * x10;
        Fp x184 = x181 + x183;
        Fp x185 = args[2][159 * steps + ((cycle - 0) & mask)];
        Fp x186 = x185 * x11;
        Fp x187 = x184 + x186;
        Fp x188 = args[2][160 * steps + ((cycle - 0) & mask)];
        Fp x189 = x188 * x12;
        Fp x190 = x187 + x189;
        Fp x191 = args[2][161 * steps + ((cycle - 0) & mask)];
        Fp x192 = x191 * x13;
        Fp x193 = x190 + x192;
        Fp x194 = x125 - x193;
        Fp x195 = x194 * x14;
        if (x195 != 0) throw std::runtime_error("eqz failed at: external/risc0/risc0/zkvm/circuit/mem_check.cpp:48");
      }
    }
    Fp x196 = args[2][149 * steps + ((cycle - 0) & mask)];
    Fp x197 = x196 - x15;
    Fp x198 = x196 * x197;
    if (x198 != 0) throw std::runtime_error("eqz failed at: external/risc0/risc0/zkvm/circuit/mem_check.h:28");
    Fp x199 = args[2][150 * steps + ((cycle - 0) & mask)];
    Fp x200 = x199 - x15;
    Fp x201 = x199 * x200;
    Fp x202 = x199 - x0;
    Fp x203 = x201 * x202;
    Fp x204 = x199 - x1;
    Fp x205 = x203 * x204;
    if (x205 != 0) throw std::runtime_error("eqz failed at: external/risc0/risc0/zkvm/circuit/mem_check.h:28");
    Fp x206 = args[2][151 * steps + ((cycle - 0) & mask)];
    Fp x207 = x206 - x15;
    Fp x208 = x206 * x207;
    Fp x209 = x206 - x0;
    Fp x210 = x208 * x209;
    Fp x211 = x206 - x1;
    Fp x212 = x210 * x211;
    if (x212 != 0) throw std::runtime_error("eqz failed at: external/risc0/risc0/zkvm/circuit/mem_check.h:28");
    Fp x213 = args[2][152 * steps + ((cycle - 0) & mask)];
    Fp x214 = x213 - x15;
    Fp x215 = x213 * x214;
    Fp x216 = x213 - x0;
    Fp x217 = x215 * x216;
    Fp x218 = x213 - x1;
    Fp x219 = x217 * x218;
    if (x219 != 0) throw std::runtime_error("eqz failed at: external/risc0/risc0/zkvm/circuit/mem_check.h:28");
    Fp x220 = args[2][153 * steps + ((cycle - 0) & mask)];
    Fp x221 = x220 - x15;
    Fp x222 = x220 * x221;
    Fp x223 = x220 - x0;
    Fp x224 = x222 * x223;
    Fp x225 = x220 - x1;
    Fp x226 = x224 * x225;
    if (x226 != 0) throw std::runtime_error("eqz failed at: external/risc0/risc0/zkvm/circuit/mem_check.h:28");
    Fp x227 = args[2][154 * steps + ((cycle - 0) & mask)];
    Fp x228 = x227 - x15;
    Fp x229 = x227 * x228;
    Fp x230 = x227 - x0;
    Fp x231 = x229 * x230;
    Fp x232 = x227 - x1;
    Fp x233 = x231 * x232;
    if (x233 != 0) throw std::runtime_error("eqz failed at: external/risc0/risc0/zkvm/circuit/mem_check.h:28");
    Fp x234 = args[2][155 * steps + ((cycle - 0) & mask)];
    Fp x235 = x234 - x15;
    Fp x236 = x234 * x235;
    Fp x237 = x234 - x0;
    Fp x238 = x236 * x237;
    Fp x239 = x234 - x1;
    Fp x240 = x238 * x239;
    if (x240 != 0) throw std::runtime_error("eqz failed at: external/risc0/risc0/zkvm/circuit/mem_check.h:28");
    Fp x241 = args[2][156 * steps + ((cycle - 0) & mask)];
    Fp x242 = x241 - x15;
    Fp x243 = x241 * x242;
    Fp x244 = x241 - x0;
    Fp x245 = x243 * x244;
    Fp x246 = x241 - x1;
    Fp x247 = x245 * x246;
    if (x247 != 0) throw std::runtime_error("eqz failed at: external/risc0/risc0/zkvm/circuit/mem_check.h:28");
    Fp x248 = args[2][157 * steps + ((cycle - 0) & mask)];
    Fp x249 = x248 - x15;
    Fp x250 = x248 * x249;
    Fp x251 = x248 - x0;
    Fp x252 = x250 * x251;
    Fp x253 = x248 - x1;
    Fp x254 = x252 * x253;
    if (x254 != 0) throw std::runtime_error("eqz failed at: external/risc0/risc0/zkvm/circuit/mem_check.h:28");
    Fp x255 = args[2][158 * steps + ((cycle - 0) & mask)];
    Fp x256 = x255 - x15;
    Fp x257 = x255 * x256;
    Fp x258 = x255 - x0;
    Fp x259 = x257 * x258;
    Fp x260 = x255 - x1;
    Fp x261 = x259 * x260;
    if (x261 != 0) throw std::runtime_error("eqz failed at: external/risc0/risc0/zkvm/circuit/mem_check.h:28");
    Fp x262 = args[2][159 * steps + ((cycle - 0) & mask)];
    Fp x263 = x262 - x15;
    Fp x264 = x262 * x263;
    Fp x265 = x262 - x0;
    Fp x266 = x264 * x265;
    Fp x267 = x262 - x1;
    Fp x268 = x266 * x267;
    if (x268 != 0) throw std::runtime_error("eqz failed at: external/risc0/risc0/zkvm/circuit/mem_check.h:28");
    Fp x269 = args[2][160 * steps + ((cycle - 0) & mask)];
    Fp x270 = x269 - x15;
    Fp x271 = x269 * x270;
    Fp x272 = x269 - x0;
    Fp x273 = x271 * x272;
    Fp x274 = x269 - x1;
    Fp x275 = x273 * x274;
    if (x275 != 0) throw std::runtime_error("eqz failed at: external/risc0/risc0/zkvm/circuit/mem_check.h:28");
    Fp x276 = args[2][161 * steps + ((cycle - 0) & mask)];
    Fp x277 = x276 - x15;
    Fp x278 = x276 * x277;
    Fp x279 = x276 - x0;
    Fp x280 = x278 * x279;
    Fp x281 = x276 - x1;
    Fp x282 = x280 * x281;
    if (x282 != 0) throw std::runtime_error("eqz failed at: external/risc0/risc0/zkvm/circuit/mem_check.h:28");
  }
  return x15;
}

} // namespace risc0::circuit::rv32im
// clang-format on
