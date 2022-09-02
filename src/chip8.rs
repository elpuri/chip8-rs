use std::ops::{BitAnd, BitXor, BitOr};

use rand::RngCore;

pub struct Chip8<'program> {
    memory: [u8; Chip8::MEM_SIZE],
    stack: [usize; Chip8::STACK_SIZE],
    program: &'program[u8],
    display: [u8; Chip8::DISPLAY_SIZE],
    keys: [u8; 16],
    reg: [u8; 16],
    i: usize,
    sp: usize,
    pc: usize,
    delay_timer: u8,
    sound_timer: u8,
    waiting_for_key: Option<u8>
}

const FONT: &[u8] = &[
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

impl <'program> Chip8<'program> {
    pub const DISPLAY_WIDTH: usize = 64;
    pub const DISPLAY_HEIGHT: usize = 32;
    pub const DISPLAY_SIZE: usize = (Chip8::DISPLAY_WIDTH * Chip8::DISPLAY_HEIGHT) as usize;
    const STACK_SIZE: usize = 16;
    const MEM_SIZE: usize = 4096;
    const RESET_VECTOR: usize = 0x200;

    pub fn new(program: &'program[u8]) -> Self {
        let memory = [0; Chip8::MEM_SIZE];

        let mut c8 = Chip8 {
            reg: [0; 16],
            i: 0,
            pc: 0,
            sp: 0,
            memory,
            program,
            keys: [0; 16],
            stack: [0; Chip8::STACK_SIZE],
            display: [0; (Chip8::DISPLAY_WIDTH * Chip8::DISPLAY_HEIGHT) as usize],
            delay_timer: 0,
            sound_timer: 0,
            waiting_for_key: None
        };
        c8.reset();
        c8
    }

    pub fn reset(&mut self) {
        self.pc = Chip8::RESET_VECTOR;
        self.reg = [0; 16];
        self.sp = 0;
        self.keys = [0; 16];
        self.memory[Chip8::RESET_VECTOR..Chip8::RESET_VECTOR + self.program.len()].copy_from_slice(self.program);
        self.memory[0..FONT.len()].copy_from_slice(FONT);
    }

    pub fn tick_60hz(&mut self) {
        self.delay_timer = self.delay_timer.wrapping_sub(1);
    }

    pub fn set_key_state(&mut self, key: u8, pressed: bool) {
        if key >= 16 {
            panic!("Invalid key");
        }

        if self.waiting_for_key == Some(key) {
            self.waiting_for_key = None;
        }

        self.keys[key as usize] = if pressed { 255 } else { 0 };
    }

    pub fn pixels(&self) -> &[u8] {
        &self.display
    }

    pub fn step(&mut self, count: usize) {
        if self.waiting_for_key.is_some() {
            return;
        }

        let mem = self.memory;
        for _c in 0..count {
            let pc = self.pc;
            let instr: u16 = (mem[pc] as u16) << 8 | mem[pc + 1] as u16;

            let n3 = instr as usize >> 12 & 0xf;
            let n2 = instr as usize >> 8  & 0xf;
            let n1 = instr as usize >> 4  & 0xf;
            let n0 = instr as usize >> 0  & 0xf;
            let b0 = (instr & 0xff) as u8;

            match (n3, n2, n1, n0) {
                // Clear display
                (0, 0, 0xe, 0x0) => {
                    self.display = [0; Chip8::DISPLAY_SIZE];
                    self.pc += 2;
                }

                // Return
                (0, 0, 0xe, 0xe) => {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp];
                }

                // Jump
                (1, ..) => {
                    self.pc = (instr & 0xfff) as usize;
                }

                // Call
                (2, ..) => {
                    self.stack[self.sp] = self.pc + 2;
                    self.sp += 1;
                    self.pc = (instr & 0xfff) as usize;

                }

                // Skip next instruction if reg equal to immediate
                (3, ..) => {
                    let eq = self.reg[n2] == b0;
                    self.pc += if eq { 4 } else { 2 };
                }

                // Skip next instruction if reg not equal to immediate
                (4, ..) => {
                    let neq = self.reg[n2] != b0;
                    self.pc += if neq { 4 } else { 2 };
                }

                // Skip next instruction if given regs are equal
                (5, ..) => {
                    let eq = self.reg[n1] == self.reg[n2];
                    self.pc += if eq { 4 } else { 2 };
                }

                // Load immediate to reg
                (6, ..) => {
                    self.reg[n2] = b0;
                    self.pc += 2;
                }

                // Add immediate to reg
                (7, ..) => {
                    self.reg[n2] = u8::wrapping_add(self.reg[n2], b0);
                    self.pc += 2;
                }

                // Reg to reg ALU ops
                (8, ..) => {
                    let dst = self.reg[n2];
                    let src = self.reg[n1];
                    self.reg[n2] = match n0 {
                        0 => src,               // LD
                        1 => dst.bitor(src),    // OR
                        2 => dst.bitand(src),   // AND
                        3 => dst.bitxor(src),   // XOR
                        4 => {  // ADD
                            let r: u16 = src as u16 + dst as u16;
                            self.reg[15] = if r & 0xff00 != 0 { 1 } else { 0 };   // carry
                            (r & 0xff) as u8
                        },
                        5 => {  // SUB
                            self.reg[15] = if src > dst { 1 } else { 0 };   // overflow
                            dst.wrapping_sub(src)
                        },
                        6 => {  // SHR
                            self.reg[15] = if dst & 1 != 0 { 1 } else { 0 };     // lsb
                            dst >> 1
                        },
                        7 => {  // SUBN
                            self.reg[15] = if dst > src { 1 } else { 0 };   // overflow
                            src.wrapping_sub(dst)
                        },
                        0xe => {  // SHL
                            self.reg[15] = if dst & 0x80 != 0 { 1 } else { 0 };     // msb
                            dst << 1
                        },

                        _=> {
                            panic!("{:#04x} is not an instruction", instr);
                        }
                    };
                    self.pc += 2;
                },

                // Skip next instruction if given regs are not equal
                (9, ..) => {
                    let neq = self.reg[n1] != self.reg[n2];
                    self.pc += if neq { 4 } else { 2 };
                },

                // Load immediate to I
                (0xa, ..) => {
                    self.i = (instr & 0xfff) as usize;
                    self.pc += 2;
                },

                // Jump to gp0 + immediate
                (0xb, ..) => {
                    self.pc = ((instr & 0xfff) + self.reg[0] as u16) as usize;
                },

                // RND
                (0xc, ..) => {
                    self.reg[n2] = rand::thread_rng().next_u32() as u8 & b0;
                    self.pc += 2;
                },

                // Draw
                (0xd, ..) => {
                    // I = sprite ptr
                    let sprite_ptr = self.i;
                    let ox = self.reg[n2] as usize;
                    let oy = self.reg[n1] as usize;
                    let sprite_height = n0;
                    let mut collision = false;

                    for y in 0..sprite_height {
                        let sprite_line = self.memory[sprite_ptr + y];
                        for x in 0..8 {
                            let sx = (ox + x) & 63;
                            let sy = (oy + y) & 31;
                            let px = if (sprite_line >> (7 - x)) & 0x01 == 1 { 255 }  else { 0 };
                            let prev = self.xor_pixel(sx, sy, px);
                            collision |= px != 0 && prev != 0;
                        }
                    }

                    self.reg[15] = if collision { 1 } else { 0 };
                    self.pc += 2;
                },

                // Skip next instruction if key pressed
                (0xe, _, 0x9, 0xe) => {
                    let pressed = self.keys[self.reg[n2] as usize] != 0;
                    self.pc += if pressed { 4 } else { 2 };
                },

                // Skip next instruction if key not pressed
                (0xe, _, 0xa, 0x1) => {
                    let not_pressed = self.keys[self.reg[n2] as usize] == 0;
                    self.pc += if not_pressed { 4 } else { 2 };
                },

                // Store registers to [i]
                (0xf, _, 5, 5) => {
                    for i in 0..=n2 {
                        self.memory[self.i] = self.reg[i];
                        self.i += 1;
                    }
                    self.pc += 2;
                }

                (0xf, _, 0x6, 0x5) => {
                    for i in 0..=n2 {
                        self.reg[i] = self.memory[self.i];
                        self.i += 1;
                    }
                    self.pc += 2;
                }

                (0xf, _, 0x0, 0x7) => {
                    self.reg[n2 as usize] = self.delay_timer;
                    self.pc += 2;
                },

                (0xf, _, 0x0, 0xa) => {
                    self.waiting_for_key = Some(self.reg[n2 as usize]);
                    self.pc += 2;
                },

                (0xf, _, 0x1, 0x5) => {
                    self.delay_timer = self.reg[n2 as usize];
                    self.pc += 2;
                },

                (0xf, _, 0x1, 0xe) => {
                    self.i += self.reg[n2] as usize;
                    self.pc += 2;
                }

                (0xf, _, 0x1, 0x8) => {
                    let v = self.reg[n2];
                    self.sound_timer = v;
                    self.pc += 2;
                }

                (0xf, _, 0x2, 0x9) => {
                    // font is at 0x0000 in memory
                    let digit = self.reg[n2] as usize;
                    self.i = digit * 5;
                    self.pc += 2;
                }
                _ => todo!("Unimplemented CPU instruction {:#04x}", instr)
            }

        }
    }

    fn xor_pixel(&mut self, x: usize, y: usize, px: u8) -> u8 {
        let pixel = &mut self.display[y * Chip8::DISPLAY_WIDTH + x];
        let old_val = *pixel;
        *pixel = px ^ *pixel;
        old_val
    }

}

pub fn dump_display(c8: &Chip8) {
    for y in 0..Chip8::DISPLAY_HEIGHT {
        let mut line = String::with_capacity(Chip8::DISPLAY_WIDTH);
        for x in 0..Chip8::DISPLAY_WIDTH {
            line.push(if c8.display[y * Chip8::DISPLAY_WIDTH + x] == 0 { '.' } else { 'x' });
        }
        println!("{line}");
    }
}

pub fn dump_machine_state(c8: &Chip8) {
    println!("pc: ${:#06X}, instr: {:#06X}, i: {:#06X}, regs {:02x?}",
        c8.pc,
        ((c8.memory[c8.pc] as u16) << 8) | c8.memory[c8.pc + 1] as u16,
        c8.i,
        c8.reg
    );

}


#[cfg(test)]
mod tests;
