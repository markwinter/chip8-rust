use crate::chip8::{FONT_START_LOC, REGISTER_VF, SCREEN_HEIGHT, SCREEN_WIDTH, SPRITE_WIDTH};
use rand::Rng;

// Returns VX
fn get_register_x(opcode: u16) -> u8 {
    ((opcode & 0x0F00) >> 8) as u8
}

// Returns VY
fn get_register_y(opcode: u16) -> u8 {
    ((opcode & 0x00F0) >> 4) as u8
}

// 00E0: Clear the screen
// 00EE: Return from subroutine
pub fn op0000(c8: &mut super::Chip8) {
    match c8.current_opcode & 0x00FF {
        0xEE => {
            let val = c8.stack.pop();
            c8.program_counter = val.expect("returning from subroutine but empty stack");
        }
        0xE0 => {
            c8.screen = [0; SCREEN_WIDTH * SCREEN_HEIGHT];
        }
        _ => (),
    }
}

// 1NNN: Jump to NNN
pub fn op1000(c8: &mut super::Chip8) {
    c8.program_counter = c8.current_opcode & 0x0FFF;
}

// 2NNN: Call subroutine at NNN
pub fn op2000(c8: &mut super::Chip8) {
    c8.stack.push(c8.program_counter);
    c8.program_counter = c8.current_opcode & 0x0FFF;
}

// 3XNN: Skip next instruction if VX == NN
pub fn op3000(c8: &mut super::Chip8) {
    let nn = (c8.current_opcode & 0x00FF) as u8;
    let reg_index = get_register_x(c8.current_opcode);

    if nn == c8.registers[reg_index as usize] {
        c8.program_counter += 2;
    }
}

// 4XNN: Skips next instruction if VX != NN
pub fn op4000(c8: &mut super::Chip8) {
    let nn = c8.current_opcode & 0x00FF;
    let reg_index = get_register_x(c8.current_opcode);

    if nn as u8 != c8.registers[reg_index as usize] {
        c8.program_counter += 2;
    }
}

// 5XY0: Skips next instruction if VX equals VY
pub fn op5000(c8: &mut super::Chip8) {
    let reg_index_x = get_register_x(c8.current_opcode);
    let reg_index_y = get_register_y(c8.current_opcode);

    if c8.registers[reg_index_x as usize] == c8.registers[reg_index_y as usize] {
        c8.program_counter += 2;
    }
}

// 6XNN: Sets VX to NN
pub fn op6000(c8: &mut super::Chip8) {
    let reg_index = get_register_x(c8.current_opcode);
    c8.registers[reg_index as usize] = (c8.current_opcode & 0x00FF) as u8;
}

// 7XNN: Adds NN to VX
pub fn op7000(c8: &mut super::Chip8) {
    let reg_index = get_register_x(c8.current_opcode);
    let nn = (c8.current_opcode & 0x00FF) as u8;
    c8.registers[reg_index as usize] = c8.registers[reg_index as usize].wrapping_add(nn);
}

// 8XY0: Set VX = VY
// 8XY1: Set VX = VX|VY
// 8XY2: Set VX = VX&VY
// 8XY3: Set VX = VX^VY
// 8XY4: Set VX += VY
// 8XY5: Set VX -= VY
// 8XY6: Store least significant bit of VX in VF and shift VX right 1
// 8XY7: Set VX = VY - VX. Set VF=0 when there's a borrow, else 1
// 8XYE: Store most significant bit of VX in VF then shift VX left 1
pub fn op8000(c8: &mut super::Chip8) {
    let register_x = get_register_x(c8.current_opcode);
    let register_y = get_register_y(c8.current_opcode);

    match c8.current_opcode & 0x000F {
        0x0 => c8.registers[register_x as usize] = c8.registers[register_y as usize],
        0x1 => c8.registers[register_x as usize] |= c8.registers[register_y as usize],
        0x2 => c8.registers[register_x as usize] &= c8.registers[register_y as usize],
        0x3 => c8.registers[register_x as usize] ^= c8.registers[register_y as usize],
        0x4 => {
            let x = register_x as usize;
            let y = register_y as usize;
            let sum = c8.registers[x] as u16 + c8.registers[y] as u16;
            c8.registers[REGISTER_VF as usize] = if sum > 0xFF { 1 } else { 0 };
            c8.registers[x] = c8.registers[x].wrapping_add(c8.registers[y]);
        }
        0x5 => {
            let x = register_x as usize;
            let y = register_y as usize;
            c8.registers[REGISTER_VF as usize] = if c8.registers[x] >= c8.registers[y] {
                1
            } else {
                0
            };
            c8.registers[x] = c8.registers[x].wrapping_sub(c8.registers[y]);
        }
        0x6 => {
            c8.registers[REGISTER_VF as usize] = c8.registers[register_x as usize] & 0x01;
            c8.registers[register_x as usize] >>= 1;
        }
        0x7 => {
            let x = register_x as usize;
            let y = register_y as usize;
            c8.registers[REGISTER_VF as usize] = if c8.registers[y] >= c8.registers[x] {
                1
            } else {
                0
            };
            c8.registers[x] = c8.registers[y].wrapping_sub(c8.registers[x]);
        }
        0xE => {
            c8.registers[REGISTER_VF as usize] = (c8.registers[register_x as usize] & 0x80) >> 7;
            c8.registers[register_x as usize] <<= 1;
        }
        _ => (),
    }
}

// 9XY0: Skips next instruction if VX doesn't equal VY
pub fn op9000(c8: &mut super::Chip8) {
    let reg_index_x = get_register_x(c8.current_opcode);
    let reg_index_y = get_register_y(c8.current_opcode);

    if c8.registers[reg_index_x as usize] != c8.registers[reg_index_y as usize] {
        c8.program_counter += 2;
    }
}

// ANNN: Sets I to NNN
pub fn opa000(c8: &mut super::Chip8) {
    c8.index_register = c8.current_opcode & 0x0FFF;
}

// BNNN: Jump to the address NNN plus V0
pub fn opb000(c8: &mut super::Chip8) {
    let reg_val = c8.registers[0];
    let addr = c8.current_opcode & 0x0FFF;
    c8.program_counter = reg_val as u16 + addr;
}

// CXNN: Sets VX to the result of a NN & randomNumber
pub fn opc000(c8: &mut super::Chip8) {
    let x = get_register_x(c8.current_opcode) as usize;
    let nn = (c8.current_opcode & 0xFF) as u8;
    let rng: u8 = rand::rng().random_range(0..255);
    c8.registers[x] = nn & rng;
}

// DXYN: Draw at (VX, VY) with width=8, height=N+1
// Each row of 8 pixels is read as bit-coded starting from memory location I
// VF is set to 1 if any screen pixels are flipped from set to unset
pub fn opd000(c8: &mut super::Chip8) {
    let reg_y = get_register_y(c8.current_opcode);
    let reg_x = get_register_x(c8.current_opcode);

    let x_coord = c8.registers[reg_x as usize] as u16;
    let y_coord = c8.registers[reg_y as usize] as u16;

    let width = SPRITE_WIDTH as u16;
    let height = c8.current_opcode & 0x000F;

    let mut flipped = false;

    for row in 0..height {
        let addr = c8.index_register + row;
        let pixels = c8.memory[addr as usize];

        for col in 0..width {
            let sprite_pixel = pixels & (0x80 >> col);

            if sprite_pixel == 0 {
                continue;
            }

            let x = (x_coord + col) as usize % SCREEN_WIDTH;
            let y = (y_coord + row) as usize % SCREEN_HEIGHT;

            let screen_loc = x + SCREEN_WIDTH * y;

            flipped |= c8.screen[screen_loc] != 0;
            c8.screen[screen_loc] ^= 1;
        }
    }

    if flipped {
        c8.registers[REGISTER_VF as usize] = 1;
    } else {
        c8.registers[REGISTER_VF as usize] = 0;
    }
}

// EX9E: Skip next instruction if key stored in VX is pressed
// EXA1: Skip next instruction if key stored in VX isn't pressed
pub fn ope000(c8: &mut super::Chip8) {
    let reg_index = get_register_x(c8.current_opcode);
    let key = c8.registers[reg_index as usize];

    match c8.current_opcode & 0x000F {
        0xE => {
            if c8.keys[key as usize] == 1 {
                c8.program_counter += 2;
            }
        }
        0x1 => {
            if c8.keys[key as usize] == 0 {
                c8.program_counter += 2;
            }
        }
        _ => (),
    }
}

// FX07: Set VX = delay timer
// FX0A: A key press is awaited, then stored in VX (blocking)
// FX15: Set delay timer = VX
// FX18: Set sound timer = VX
// FX1E: Set I += VX
// FX29: Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font.
// FX33: Stores the binary-coded decimal representation of VX,
// with the most significant of three digits at the address in I,
// the middle digit at I plus 1, and the least significant digit at I plus 2
// FX55: Store V0-VX (inclusive) starting at memory address I
// FX65: Fill V0-VX (inclusive) with values from memory address I
pub fn opf000(c8: &mut super::Chip8) {
    let opcode = c8.current_opcode & 0x00FF;
    match opcode {
        0x07 => {
            let register = get_register_x(c8.current_opcode);
            c8.registers[register as usize] = c8.delay_timer;
        }
        0x0A => unimplemented!("opcode unimplemented: {:x}", c8.current_opcode),
        0x15 => {
            let register = get_register_x(c8.current_opcode);
            c8.delay_timer = c8.registers[register as usize];
        }
        0x18 => {
            let register = get_register_x(c8.current_opcode);
            c8.sound_timer = c8.registers[register as usize];
        }
        0x1E => {
            let register = get_register_x(c8.current_opcode);
            c8.index_register = c8
                .index_register
                .wrapping_add(c8.registers[register as usize] as u16);
        }
        0x29 => {
            let register = get_register_x(c8.current_opcode);
            let character = c8.registers[register as usize];
            c8.index_register = (FONT_START_LOC + (5 * character)) as u16;
        }
        0x33 => {
            let register = get_register_x(c8.current_opcode);
            let mut value = c8.registers[register as usize];

            c8.memory[(c8.index_register + 2) as usize] = value % 10;
            value /= 10;

            c8.memory[(c8.index_register + 1) as usize] = value % 10;
            value /= 10;

            c8.memory[c8.index_register as usize] = value % 10;
        }
        0x55 => {
            let register = get_register_x(c8.current_opcode);
            for i in 0..=register {
                c8.memory[(c8.index_register + i as u16) as usize] = c8.registers[i as usize];
            }
        }
        0x65 => {
            let register = get_register_x(c8.current_opcode);
            for i in 0..=register {
                c8.registers[i as usize] = c8.memory[(c8.index_register + i as u16) as usize];
            }
        }
        _ => (),
    }
}
