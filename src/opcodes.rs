#![allow(dead_code)]
pub type OpCode = u16;
pub const SPL: OpCode = 0x0000;
pub const SET: OpCode = 0x0001;
pub const ADD: OpCode = 0x0002;
pub const SUB: OpCode = 0x0003;
pub const MUL: OpCode = 0x0004;
pub const MLI: OpCode = 0x0005;
pub const DIV: OpCode = 0x0006;
pub const DVI: OpCode = 0x0007;
pub const MOD: OpCode = 0x0008;
pub const MDI: OpCode = 0x0009;
pub const AND: OpCode = 0x000A;
pub const BOR: OpCode = 0x000B;
pub const XOR: OpCode = 0x000C;
pub const SHR: OpCode = 0x000D;
pub const ASR: OpCode = 0x000E;
pub const SHL: OpCode = 0x000F;
pub const IFB: OpCode = 0x0010;
pub const IFC: OpCode = 0x0011;
pub const IFE: OpCode = 0x0012;
pub const IFN: OpCode = 0x0013;
pub const IFG: OpCode = 0x0014;
pub const IFA: OpCode = 0x0015;
pub const IFL: OpCode = 0x0016;
pub const IFU: OpCode = 0x0017;
pub const ADX: OpCode = 0x001A;
pub const SBX: OpCode = 0x001B;
pub const STI: OpCode = 0x001E;
pub const STD: OpCode = 0x001F;

// Special codes
pub const JSR: OpCode = 0x0001;
pub const INT: OpCode = 0x0008;
pub const IAG: OpCode = 0x0009;
pub const IAS: OpCode = 0x000A;
pub const RFI: OpCode = 0x000B;
pub const IAQ: OpCode = 0x000C;
pub const HWN: OpCode = 0x0010;
pub const HWQ: OpCode = 0x0011;
pub const HWI: OpCode = 0x0012;
