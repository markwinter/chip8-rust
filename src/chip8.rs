mod opcodes;

use std::fs;

const FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

const FONT_START_LOC: u8 = 0x50;
const PROGRAM_START_LOC: u16 = 0x200;

const REGISTER_VF: u8 = 15;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const SPRITE_WIDTH: u8 = 8;

#[derive(Debug)]
pub struct Chip8 {
    memory: [u8; 4096],
    registers: [u8; 16],
    index_register: u16,

    stack: [u16; 64],
    sp: u8,

    current_opcode: u16,
    program_counter: u16,

    delay_timer: u8,
    sound_timer: u8,

    pub screen: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
    keys: [u8; 16],
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut mem: [u8; 4096] = [0; 4096];

        for i in 0u8..80u8 {
            mem[(FONT_START_LOC + i) as usize] = FONT_SET[i as usize];
        }

        Chip8 {
            memory: mem,
            registers: [0; 16],
            index_register: 0,

            stack: [0; 64],
            sp: 0,

            current_opcode: 0,
            program_counter: PROGRAM_START_LOC,

            delay_timer: 0,
            sound_timer: 0,

            screen: [0; SCREEN_HEIGHT * SCREEN_WIDTH],
            keys: [0; 16],
        }
    }

    pub fn load_rom(&mut self, filepath: &str) -> Result<(), std::io::Error> {
        let data = fs::read(filepath)?;
        let start = PROGRAM_START_LOC as usize;
        let end = start + data.len();
        self.memory[start..end].copy_from_slice(&data);
        Ok(())
    }

    pub fn step(&mut self) {
        self.current_opcode = (self.memory[self.program_counter as usize] as u16) << 8
            | self.memory[(self.program_counter as usize) + 1] as u16;

        match self.current_opcode & 0xF000 {
            0x0000 => opcodes::op0000(self),
            0x1000 => opcodes::op1000(self),
            0x2000 => opcodes::op2000(self),
            0x3000 => opcodes::op3000(self),
            0x4000 => opcodes::op4000(self),
            0x5000 => opcodes::op5000(self),
            0x6000 => opcodes::op6000(self),
            0x7000 => opcodes::op7000(self),
            0x8000 => opcodes::op8000(self),
            0x9000 => opcodes::op9000(self),
            0xA000 => opcodes::opa000(self),
            0xB000 => opcodes::opb000(self),
            0xC000 => opcodes::opc000(self),
            0xD000 => opcodes::opd000(self),
            0xE000 => opcodes::ope000(self),
            0xF000 => opcodes::opf000(self),
            _ => panic!("unknown opcode found: {:x}", self.current_opcode),
        }

        if self.current_opcode & 0xF000 != 0x1000 && self.current_opcode & 0xF000 != 0x2000 {
            self.program_counter += 2;
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
}
