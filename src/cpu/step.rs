




use crate::mmu::MemoryOperations;

use super::{
    instructions::{FlagState, InstParam, InstructionCondition, InstructionResult, Instructions},
    registers::{Register16Bit, Register8Bit},
    CPU,
};

impl CPU {
    // Gets a 8-bit value from the HL register
    fn get_n8_from_hl(&self) -> u8 {
        self.mmu
            .read_byte(self.get_16bit_register(Register16Bit::HL))
    }

    pub fn prepare_and_decode_next_instruction(&mut self) -> Result<Instructions, String> {
        log::debug!(
            "🖱️ Current PC: {:#06X}",
            self.get_16bit_register(Register16Bit::PC)
        );
        let opcode = self.get_next_opcode();
        log::debug!("🤖 Next opcode: {:#02X}", opcode);
        let instruction = self.decode(opcode)?;
        log::debug!("📖 Decoded instruction: {:#?}", instruction);
        self.next_instruction = instruction.clone();
        Ok(instruction)
    }

    /// Does a step (calls function and sets last_step_result),
    /// ensure to first set the next instruction
    /// by decoding it (see `decode.rs`)
    pub fn step(&mut self) -> Result<&InstructionResult, String> {
        if self.check_and_handle_interrupts() {
            self.last_step_result.cycles = 5;
            self.last_step_result.bytes = 0;
            return Ok(&self.last_step_result);
        }

        self.last_step_result = match &self.next_instruction {
            Instructions::ADD(param) => match param {
                InstParam::Register8Bit(register) => self.add_a_r8(*register),
                InstParam::Register16Bit(register) => {
                    if *register == Register16Bit::HL {
                        self.add_a_hl()
                    } else {
                        return Err(format!("ADD with {:?} not implemented", param))
                    }
                }
                InstParam::SignedNumber8Bit(value) => self.add_sp_e8(*value),
                InstParam::Number8Bit(value) => self.add_a_n8(*value),
                _ => return Err(format!("ADD with {:?} not implemented", param)),
            },
            Instructions::ADD_HL(param) => match param { 
                InstParam::Register16Bit(register) => self.add_hl_r16(*register),
                _ => return Err(format!("ADD_HL with {:?} not implemented", param)),
            }
            Instructions::ADC(param) => match param {
                InstParam::Register8Bit(register) => self.adc_a_r8(*register),
                InstParam::Register16Bit(_register) => self.adc_a_hl(),
                InstParam::Number8Bit(value) => self.adc_a_n8(*value),
                _ => return Err(format!("ADD with {:?} not implemented", param)),
            },
            Instructions::INC(param, hl_memory) => match param {
                InstParam::Register8Bit(register) => self.inc(*register),
                InstParam::Register16Bit(register) => match register {
                    Register16Bit::SP => self.inc_sp(),
                    Register16Bit::HL => match hl_memory {
                        InstParam::Boolean(hl_with_memory) => {
                            if *hl_with_memory {
                                self.inc_hl()
                            } else {
                                self.inc_r16(*register)
                            }
                        }
                        _ => return Err(format!("INC with {:?} not implemented", param)),
                    },
                    _ => self.inc_r16(*register),
                },
                _ => return Err(format!("INC with {:?} not implemented", param)),
            },
            Instructions::DEC(param, hl_memory) => match param {
                InstParam::Register8Bit(register) => self.dec_r8(*register),
                InstParam::Register16Bit(register) => match register {
                    Register16Bit::SP => self.dec_sp(),
                    Register16Bit::HL => match hl_memory {
                        InstParam::Boolean(hl_with_memory) => {
                            if *hl_with_memory {
                                self.dec_hl()
                            } else {
                                self.dec_r16(*register)
                            }
                        }
                        _ => return Err(format!("INC with {:?} not implemented", param)),
                    },
                    _ => self.dec_r16(*register),
                },
                _ => return Err(format!("INC with {:?} not implemented", param)),
            },
            Instructions::SUB(param) => match param {
                InstParam::Register8Bit(register) => {
                    self.sub_and_subc(self.get_8bit_register(*register), 1, 1, false)
                }
                InstParam::Register16Bit(_) => {
                    self.sub_and_subc(self.get_n8_from_hl(), 2, 1, false)
                }
                InstParam::Number8Bit(value) => self.sub_and_subc(*value, 2, 2, false),
                _ => return Err(format!("SUB with {:?} not implemented", param)),
            },
            Instructions::SBC(param) => match param {
                InstParam::Register8Bit(register) => {
                    self.sub_and_subc(self.get_8bit_register(*register), 1, 1, true)
                }
                InstParam::Register16Bit(_) => self.sub_and_subc(self.get_n8_from_hl(), 2, 1, true),
                InstParam::Number8Bit(value) => self.sub_and_subc(*value, 2, 2, true),
                _ => return Err(format!("SBC with {:?} not implemented", param)),
            },
            Instructions::CP(param) => match param {
                InstParam::Register8Bit(register) => self.cp_a_r8(*register),
                InstParam::Register16Bit(_) => self.cp_a_hl(),
                InstParam::Number8Bit(value) => self.cp_a_n8(*value),
                _ => return Err(format!("AND with {:?} not implemented", param)),
            },
            Instructions::OR(param) => match param {
                InstParam::Register8Bit(register) => {
                    self.or(self.get_8bit_register(*register), 1, 1)
                }
                InstParam::Register16Bit(_) => self.or(self.get_n8_from_hl(), 2, 1),
                InstParam::Number8Bit(value) => self.or(*value, 2, 2),
                _ => return Err(format!("OR with {:?} not implemented", param)),
            },
            Instructions::XOR(param) => match param {
                InstParam::Register8Bit(register) => {
                    self.xor(self.get_8bit_register(*register), 1, 1)
                }
                InstParam::Register16Bit(_) => self.xor(self.get_n8_from_hl(), 2, 1),
                InstParam::Number8Bit(value) => self.xor(*value, 2, 2),
                _ => return Err(format!("XOR with {:?} not implemented", param)),
            },
            Instructions::AND(param) => match param {
                InstParam::Register8Bit(register) => self.and_a_r8(*register),
                InstParam::Register16Bit(_) => self.and_a_hl(),
                InstParam::Number8Bit(value) => self.and_a_n8(*value),
                _ => return Err(format!("AND with {:?} not implemented", param)),
            },
            Instructions::LDAHLD => self.ld_a_hld(),
            Instructions::LDHLDA => self.ld_hld_a(),
            Instructions::LDAHLI => self.ld_a_hli(),
            Instructions::LDHLIA => self.ld_hli_a(),
            Instructions::PUSH(target) => match target {
                InstParam::Register16Bit(register) => {
                    if *register == Register16Bit::AF {
                        self.push_af()
                    } else {
                        self.push_r16(*register)
                    }
                }
                _ => return Err(format!("PUSH with {:?} not implemented", target)),
            },
            Instructions::POP(target) => match target {
                InstParam::Register16Bit(register) => {
                    if *register == Register16Bit::AF {
                        self.pop_af()
                    } else {
                        self.pop_r16(*register)
                    }
                }
                _ => return Err(format!("PUSH with {:?} not implemented", target)),
            },
            Instructions::BIT(bit, target) => match target {
                InstParam::Register8Bit(register) => match bit {
                    InstParam::Unsigned3Bit(targeted_bit) => {
                        self.bit_u3_r8(*targeted_bit, *register)
                    }
                    _ => return Err(format!("BIT with {:?} not implemented", bit)),
                },
                InstParam::Register16Bit(register) => {
                    if *register == Register16Bit::HL {
                        match bit {
                            InstParam::Unsigned3Bit(targeted_bit) => self.bit_u3_hl(*targeted_bit),
                            _ => return Err(format!("BIT with {:?} not implemented", bit)),
                        }
                    } else {
                        return Err(format!("BIT with {:?} not implemented", target));
                    }
                }
                _ => return Err(format!("BIT with {:?} not implemented", target)),
            },
            Instructions::RES(bit, target) => match bit {
                InstParam::Unsigned3Bit(targeted_bit) => match target {
                    InstParam::Register8Bit(register) => self.res_u3_r8(*targeted_bit, *register),
                    InstParam::Register16Bit(register) => {
                        if *register == Register16Bit::HL {
                            self.res_u3_hl(*targeted_bit)
                        } else {
                            return Err(format!("RES with {:?} not implemented", target));
                        }
                    }
                    _ => return Err(format!("RES with {:?} not implemented", target)),
                },
                _ => return Err(format!("RES with {:?} not implemented", target)),
            },
            Instructions::SET(bit, target) => match bit {
                InstParam::Unsigned3Bit(targeted_bit) => match target {
                    InstParam::Register8Bit(register) => self.set_u3_r8(*targeted_bit, *register),
                    InstParam::Register16Bit(register) => {
                        if *register == Register16Bit::HL {
                            self.set_u3_hl(*targeted_bit)
                        } else {
                            return Err(format!("SET with {:?} not implemented", target));
                        }
                    }
                    _ => return Err(format!("SET with {:?} not implemented", target)),
                },
                _ => return Err(format!("SET with {:?} not implemented", target)),
            },
            Instructions::SWAP(target) => match target {
                InstParam::Register8Bit(register) => self.swap_r8(*register),
                InstParam::Register16Bit(register) => {
                    if *register == Register16Bit::HL {
                        self.swap_hl()
                    } else {
                        return Err(format!("SWAP with {:?} not implemented", target));
                    }
                }
                _ => return Err(format!("SWAP with {:?} not implemented", target)),
            },
            Instructions::RLA() =>self.rl_a(),
            Instructions::RL(target) => match target {
                InstParam::Register8Bit(register) => self.rl_r8(*register),
                InstParam::Register16Bit(Register16Bit::HL) => self.rl_hl(),
                _ => return Err(format!("SWAP with {:?} not implemented", target)),
            },

            Instructions::RLCA() => self.rl_c_a(),//RLCA and RLC A are two different instructions
            Instructions::RLC(target) => match target {
                InstParam::Register8Bit(register) => self.rl_c_r8(*register),
                InstParam::Register16Bit(Register16Bit::HL) => self.rl_c_hl(),
                _ => return Err(format!("RLC with {:?} not implemented", target)),
            },
            Instructions::RRA() => self.rr_a(),
            Instructions::RR(target) => match target {
                InstParam::Register8Bit(register) => self.rr_r8(*register),
                InstParam::Register16Bit(Register16Bit::HL) => self.rr_hl(),
                _ => return Err(format!("SWAP with {:?} not implemented", target)),
            },
            Instructions::RRCA() => self.rr_c_a(),//RRCA and RRC A are two different instructions
            Instructions::RRC(target) => match target {
                InstParam::Register8Bit(register) => self.rr_c_r8(*register),
                InstParam::Register16Bit(Register16Bit::HL) => self.rr_c_hl(),
                _ => return Err(format!("SWAP with {:?} not implemented", target)),
            },
            Instructions::SLA(target) => match target {
                InstParam::Register8Bit(register) => self.sla_r8(*register),
                InstParam::Register16Bit(Register16Bit::HL) => self.sla_hl(),
                _ => return Err(format!("SWAP with {:?} not implemented", target)),
            },
            Instructions::SRA(target) => match target {
                InstParam::Register8Bit(register) => self.sra_r8(*register),
                InstParam::Register16Bit(Register16Bit::HL) => self.sra_hl(),
                _ => return Err(format!("SWAP with {:?} not implemented", target)),
            },
            Instructions::SRL(target) => match target {
                InstParam::Register8Bit(register) => self.srl_r8(*register),
                InstParam::Register16Bit(Register16Bit::HL) => self.srl_hl(),
                _ => return Err(format!("SWAP with {:?} not implemented", target)),
            },
            Instructions::LDH(target, source) => match target {
                InstParam::Number16Bit(target_number) => self.ldh_n16_a(*target_number),
                InstParam::Number8Bit(target_number) => self.ldh_a8_a(*target_number),
                InstParam::Register8Bit(target_register) => {
                    if *target_register == Register8Bit::A {
                        match source {
                            InstParam::Number16Bit(source_number) => self.ldh_a_n16(*source_number),
                            InstParam::Number8Bit(source_number) => self.ldh_a_a8(*source_number),
                            InstParam::Register8Bit(source_register) => {
                                if *source_register == Register8Bit::C {
                                    self.ldh_a_c()
                                } else {
                                    return Err(format!(
                                        "Handling of {:?} not implemented",
                                        source_register
                                    ));
                                }
                            }
                            _ => return Err(format!("Handling of {:?} not implemented", source)),
                        }
                    } else if *target_register == Register8Bit::C {
                        self.ldh_c_a()
                    } else {
                        return Err(format!("Handling of {:?} not implemented", source));
                    }
                }
                _ => return Err(format!("Handling of {:?} not implemented", source)),
            },
            Instructions::LD(target, source) => match target {
                InstParam::Register8Bit(target_register) => {
                    if *target_register == Register8Bit::A {
                        match source {
                            InstParam::Register16Bit(source_register) => {
                                self.ld_a_r16(*source_register)
                            }
                            InstParam::Number16Bit(source_number) => self.ld_a_n16(*source_number),
                            InstParam::Register8Bit(source_register) => {
                                self.ld_r8_r8(*target_register, *source_register)
                            }
                            InstParam::Number8Bit(source_number) => {
                                self.ld_r8_n8(*target_register, *source_number)
                            }
                            _ => return Err(format!("Handling of {:?} not implemented", source)),
                        }
                    } else {
                        match source {
                            InstParam::Register8Bit(source_register) => {
                                self.ld_r8_r8(*target_register, *source_register)
                            }
                            InstParam::Number8Bit(source_number) => {
                                self.ld_r8_n8(*target_register, *source_number)
                            }
                            InstParam::Register16Bit(_source_register) => {
                                self.ld_r8_hl(*target_register)
                            }
                            _ => return Err(format!("Handling of {:?} not implemented", source)),
                        }
                    }
                }
                InstParam::Register16Bit(target_register) => {
                    if *target_register == Register16Bit::SP {
                        match source {
                            InstParam::Register16Bit(_source_register) => self.ld_sp_hl(),
                            InstParam::Number16Bit(source_address) => {
                                self.ld_sp_n16(*source_address)
                            }
                            _ => return Err(format!("LD with {:?} not implemented", source)),
                        }
                    } else if *target_register == Register16Bit::HL {
                        match source {
                            InstParam::Register8Bit(source_register) => {
                                self.ld_hl_r8(*source_register)
                            }
                            InstParam::Number8Bit(source_number) => self.ld_hl_n8(*source_number),
                            InstParam::Number16Bit(source_number) => {
                                self.ld_r16_n16(Register16Bit::HL, *source_number)
                            }
                            InstParam::SignedNumber8Bit(source_number) => {
                                self.ld_hl_sp_plus_e8(*source_number)
                            }
                            _ => return Err(format!("Handling of {:?} not implemented", source)),
                        }
                    } else {
                        match source {
                            InstParam::Number16Bit(source_number) => {
                                self.ld_r16_n16(*target_register, *source_number)
                            }
                            InstParam::Register8Bit(_source_register) => {
                                self.ld_r16_a(*target_register)
                            }
                            _ => return Err(format!("Handling of {:?} not implemented", source)),
                        }
                    }
                }
                InstParam::Number16Bit(number) => match source {
                    InstParam::Register8Bit(_source_register) => self.ld_n16_a(*number),
                    InstParam::Register16Bit(_source_register) => self.ld_n16_sp(*number),
                    _ => {
                        return Err(format!(
                            "LD with n16 address of {:?} not implemented",
                            source
                        ))
                    }
                },
                _ => return Err(format!("Handling of {:?} not implemented", target)),
            },
            Instructions::RET(condition) => match condition {
                InstParam::ConditionCodes(cond) => match cond{
                    InstructionCondition::SkipConditionCodes => self.ret(),
                    _ => self.ret_cc(self.check_condition(cond)),
                }
                _ => self.ret(),
            },
            Instructions::RETI => self.reti(),
            Instructions::CALL(target_or_condition, optional_target) => match target_or_condition {
                InstParam::ConditionCodes(cond) => match optional_target {
                    InstParam::Number16Bit(target_addr) => {
                        if cond == &InstructionCondition::SkipConditionCodes {
                            self.call_n16(*target_addr)
                        } else {
                            self.call_cc_n16(self.check_condition(cond), *target_addr)
                        }
                    }
                    _ => return Err(format!("CALL of {:?} not implemented", optional_target)),
                },
                _ => return Err(format!("CALL of {:?} not implemented", target_or_condition)),
            },
            Instructions::JP(target_or_condition, optional_target) => match target_or_condition {
                InstParam::Number16Bit(target_addr) => self.jp_n16(*target_addr),
                InstParam::ConditionCodes(cond) => match optional_target {
                    InstParam::Register16Bit(target_reg) => {
                        if *target_reg == Register16Bit::HL && *cond == InstructionCondition::SkipConditionCodes {
                            self.jp_hl()
                        } else {
                            return Err(format!("JP to {:?} not implemented", target_reg));
                        }
                    }
                    InstParam::Number16Bit(target_addr) => {
                        self.jp_cc_n16(self.check_condition(cond), *target_addr)
                    }
                    _ => return Err(format!("JP of {:?} not implemented", optional_target)),
                },
                _ => return Err(format!("JP of {:?} not implemented", target_or_condition)),
            },
            Instructions::JR(target_or_condition, optional_target) => match target_or_condition {
                InstParam::SignedNumber8Bit(target_addr) => self.jr_n16(*target_addr),
                InstParam::ConditionCodes(cond) => match optional_target {
                    InstParam::SignedNumber8Bit(target_addr) => {
                        self.jr_cc_n16(self.check_condition(cond), *target_addr)
                    }
                    _ => return Err(format!("JR of {:?} not implemented", optional_target)),
                },
                _ => return Err(format!("CALL of {:?} not implemented", target_or_condition)),
            },
            Instructions::RST(vec) => match vec {
                InstParam::Number8Bit(target_addr) => self.rst_vec(*target_addr),
                _ => return Err(format!("RST of {:?} not implemented", vec)),
            },
            Instructions::CCF => self.ccf(),
            Instructions::CPL => self.cpl(),
            Instructions::DAA => self.daa(),
            Instructions::DI => self.di(),
            Instructions::EI => self.ei(),
            Instructions::HALT => self.halt(),
            Instructions::NOP => self.nop(),
            Instructions::SCF => self.scf(),
            Instructions::STOP => self.stop(),
            _ => {
                return Err(format!(
                    "Handling of {:?} not implemented",
                    self.next_instruction
                ))
            }
        };

        // Move the program counter to the next instruction
        // Depending on the bytes of the last instruction
        match self.next_instruction {
            // We need to NOT update the PC for JP, CALL, RST, RET, RETI
            Instructions::JP(_, _)
            | Instructions::CALL(_, _)
            | Instructions::RST(_)
            | Instructions::RET(_)
            | Instructions::RETI => {}
            _ => {
                let (new_val, overflow) = self.get_16bit_register(Register16Bit::PC).overflowing_add(self.last_step_result.bytes as u16);
                if overflow {
                    log::warn!("Overflow when adding {:?} to PC", self.last_step_result.bytes);
                }
                self.set_16bit_register(
                Register16Bit::PC,
                new_val
                )
        }
        }
        self.update_ime();

        match self.last_step_result.condition_codes.carry {
            FlagState::NotAffected => {}
            FlagState::Set => self.set_carry_flag(),
            FlagState::Unset => self.clear_carry_flag(),
        }

        match self.last_step_result.condition_codes.half_carry {
            FlagState::NotAffected => {}
            FlagState::Set => self.set_half_carry_flag(),
            FlagState::Unset => self.clear_half_carry_flag(),
        }

        match self.last_step_result.condition_codes.subtract {
            FlagState::NotAffected => {}
            FlagState::Set => self.set_subtraction_flag(),
            FlagState::Unset => self.clear_subtraction_flag(),
        }

        match self.last_step_result.condition_codes.zero {
            FlagState::NotAffected => {}
            FlagState::Set => self.set_zero_flag(),
            FlagState::Unset => self.clear_zero_flag(),
        }

        // Check whether we need to increase timer
        let timer_modulo = self.get_timer_modulo();
        let remaining_cycles = self.cycles % timer_modulo;
        if remaining_cycles + self.last_step_result.cycles as u64 >= timer_modulo {
            self.increment_timer();
        }

        // Update the last execution time
        self.last_execution_time = std::time::Instant::now();
        self.cycles += self.last_step_result.cycles as u64;

        // Check whether a DMA routine has been requested
        if self.mmu.IO.is_dma_requested() {
            self.dma_active = true;
            self.dma_current_offset = 0;
            self.mmu.IO.reset_dma_request();
        }

        // Do DMA for cycles used
        for _ in 0..self.last_step_result.cycles {
            self.dma_routine();
        }

        Ok(&self.last_step_result)
    }

    fn update_ime(&mut self) {
        if self.enable_ime == 1 {
            self.ime_flag = true;
            self.enable_ime = 0;
        }
        if self.enable_ime == 2 {
            self.enable_ime = 1;
        }
    }
    fn check_condition(&self, cond: &InstructionCondition) -> bool {
        match cond {
            InstructionCondition::Zero => {
                self.is_zero_flag_set()
            }
            InstructionCondition::NotZero => {
                !self.is_zero_flag_set()
            }
            InstructionCondition::Subtract => {
                self.is_subtraction_flag_set()
            }
            InstructionCondition::NotSubtract => {
                !self.is_subtraction_flag_set()
            }
            InstructionCondition::Halfcarry => {
                self.is_half_carry_flag_set()
            }
            InstructionCondition::NotHalfcarry => {
                !self.is_half_carry_flag_set()
            }
            InstructionCondition::Carry => {
                self.is_carry_flag_set()
            }
            InstructionCondition::NotCarry => {
                !self.is_carry_flag_set()
            }
            InstructionCondition::SkipConditionCodes => true,
        }
    }
}
