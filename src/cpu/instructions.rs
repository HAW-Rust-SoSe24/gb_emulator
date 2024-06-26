use super::registers::{Register16Bit, Register8Bit};

pub mod arithmetic_and_logic;
pub mod bit_shift;
pub mod misc;
pub mod load;
pub mod bit_operations;
pub mod stack_operations;
pub mod jumps_subroutines;

/// The Flag States after an instruction
/// Set: The flag is set
/// Unset: The flag is unset
/// NotAffected: The flag is not affected by the instruction
#[derive(Debug, PartialEq, Clone)]
pub enum FlagState {
    Set,
    Unset,
    NotAffected,
}

/// The condition codes after an instruction
/// zero: The zero flag (Z)
/// subtract: The subtract flag (N)
/// half_carry: The half carry flag (H)
/// carry: The carry flag (C)
#[derive(Debug, PartialEq, Clone)]
pub struct ConditionCodes {
    pub zero: FlagState,
    pub subtract: FlagState,
    pub half_carry: FlagState,
    pub carry: FlagState,
}

/// The result of an instruction
/// cycles: The number of cycles the instruction took
/// bytes: The number of bytes the instruction took
/// condition_codes: The condition codes after the instruction
#[derive(Debug, PartialEq, Clone)]
pub struct InstructionResult {
    pub cycles: u8,
    pub bytes: u8,
    pub condition_codes: ConditionCodes,
}

impl Default for InstructionResult {
    fn default() -> InstructionResult {
        InstructionResult {
            cycles: 0,
            bytes: 0,
            condition_codes: ConditionCodes {
                zero: FlagState::NotAffected,
                subtract: FlagState::NotAffected,
                half_carry: FlagState::NotAffected,
                carry: FlagState::NotAffected,
            },
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum InstructionCondition {
    Zero,
    NotZero,
    Subtract,
    NotSubtract,
    Halfcarry,
    NotHalfcarry,
    Carry,
    NotCarry,
    SkipConditionCodes,
}

#[derive(Debug, PartialEq, Clone)]
pub enum InstParam {
    Register8Bit(Register8Bit),
    Register16Bit(Register16Bit),
    ConditionCodes(InstructionCondition),
    Number8Bit(u8),
    SignedNumber8Bit(i8),
    Number16Bit(u16),
    Offset,
    Unsigned3Bit(u8),
    Boolean(bool),
    None,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Instructions {
    ADD(InstParam),
    #[allow(non_camel_case_types)]
    ADD_HL(InstParam),
    //ADD_SP(InstParam),
    ADC(InstParam),
    SUB(InstParam),
    SBC(InstParam),
    AND(InstParam),
    XOR(InstParam),
    OR(InstParam),
    CP(InstParam),
    INC(InstParam, InstParam),
    DEC(InstParam, InstParam),

    LD(InstParam, InstParam),
    LDH(InstParam, InstParam),
    LDHLIA, // LD (HL+), A
    LDAHLI, // LD A, (HL+)
    LDHLDA, // LD (HL-), A
    LDAHLD, // LD A, (HL-)
    //LD_HL_SP_SIGNED(InstParam),

    BIT(InstParam,InstParam),
    RES(InstParam,InstParam),
    SET(InstParam,InstParam),
    SWAP(InstParam),

    PUSH(InstParam),
    POP(InstParam),

    CALL(InstParam, InstParam),
    JP(InstParam, InstParam),
    JR(InstParam, InstParam),
    RET(InstParam),
    RETI,
    RST(InstParam),

    RL(InstParam),
    RLA(),
    RLC(InstParam),
    RLCA(),
    RR(InstParam),
    RRA(),
    RRC(InstParam),
    RRCA(),
    SLA(InstParam),
    SRL(InstParam),
    SRA(InstParam),

    CCF,
    CPL,
    DAA,
    DI,
    EI,
    HALT,
    NOP,
    SCF,
    STOP, 
    
    INVALID(u8), // Invalid instruction
}