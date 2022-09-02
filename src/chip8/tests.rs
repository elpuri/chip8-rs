use crate::chip8;
use chip8::{Chip8};

use super::dump_display;

#[test]
fn test_reset() {
    let fake_program = [0, 1, 2, 3];
    let mut c8 = Chip8::new(&fake_program);

    c8.pc = 1;
    c8.reg = [255; 16];
    c8.reset();

    assert_eq!(c8.pc, Chip8::RESET_VECTOR);
    for reg in c8.reg.iter() {
        assert_eq!(*reg, 0);
    }

    let program_range = Chip8::RESET_VECTOR as usize..Chip8::RESET_VECTOR as usize+fake_program.len();
    for i in 0..Chip8::MEM_SIZE {
        let byte = c8.memory[i];
        if program_range.contains(&i) {
            let a = i - Chip8::RESET_VECTOR;
            assert_eq!(a, byte as usize);
        } else {
            assert_eq!(0, byte);
        }
    }
}

#[test]
fn test_instr_ld_immediate_to_gp() {
    let test_program: &[u8] = &[
        0x60, 0x0,
        0x61, 0x1,
        0x62, 0x2,
        0x63, 0x3,
        0x64, 0x4,
        0x65, 0x5,
        0x66, 0x6,
        0x67, 0x7,
        0x68, 0x8,
        0x69, 0x9,
        0x6a, 0xa,
        0x6b, 0xb,
        0x6c, 0xc,
        0x6d, 0xd,
        0x6e, 0xe,
        0x6f, 0xf,
    ];
    let mut c8 = Chip8::new(test_program);
    c8.reg = [255; 16];
    let pc_before = c8.pc;
    c8.step(16);
    assert_eq!(c8.pc, pc_before + test_program.len());

    for i in 0..16 {
        assert_eq!(c8.reg[i] as usize, i);
    }
}

#[test]
fn test_instr_clear_display() {
    let test_program: &[u8] = &[
        0x00, 0xe0
    ];
    let mut c8 = Chip8::new(test_program);
    c8.display = [255; Chip8::DISPLAY_SIZE];

    let pc_before = c8.pc;
    c8.step(1);
    assert_eq!(c8.pc, pc_before + test_program.len());
    for i in 0..Chip8::DISPLAY_SIZE {
        assert_eq!(c8.display[i], 0);
    }
}

#[test]
fn test_instr_jump() {
    let test_program: &[u8] = &[
        0x1a, 0xbc
    ];
    let mut c8 = Chip8::new(test_program);
    c8.step(1);
    assert_eq!(c8.pc, 0xabc);
}

#[test]
fn test_instr_call() {
    let test_program: &[u8] = &[
        0x2a, 0xbc
    ];
    let mut c8 = Chip8::new(test_program);
    let pc_before = c8.pc;
    c8.step(1);
    assert_eq!(c8.pc, 0xabc);
    assert_eq!(c8.sp, 1);
    assert_eq!(c8.stack[0], pc_before + 2);
    assert_eq!(c8.pc, 0xabc);
}

#[test]
#[should_panic]
fn test_instr_stack_underflow() {
    let test_program: &[u8] = &[
        0x00, 0xee   // RET
    ];
    let mut c8 = Chip8::new(test_program);
    assert_eq!(c8.sp, 0);
    c8.step(1);
    assert!(false);     // should never get here
}

#[test]
#[should_panic]
fn test_instr_stack_overflow() {
    let test_program: &[u8] = &[
        0x22, 0x02,
        0x22, 0x04,
        0x22, 0x06,
        0x22, 0x08,
        0x22, 0x0a,
        0x22, 0x0c,
        0x22, 0x0e,
        0x22, 0x10,
        0x22, 0x12,
        0x22, 0x14,
        0x22, 0x16,
        0x22, 0x18,
        0x22, 0x1a,
        0x22, 0x1c,
        0x22, 0x1e,
        0x22, 0x20,
        0x22, 0x22,     // too much
    ];
    let mut c8 = Chip8::new(test_program);
    assert_eq!(c8.sp, 0);
    for i in 0..Chip8::STACK_SIZE {
        c8.step(1);
        assert_eq!(c8.sp, i + 1)
    }

    c8.step(1);         // panic
    assert!(false);     // we should never get here
}

#[test]
fn test_instr_skip_next_if_equals_immediate() {
    let test_program: &[u8] = &[
        0x3f, 0xff,   // skip next if gp0 == FF
    ];

    let mut c8 = Chip8::new(test_program);
    let pc = c8.pc;
    c8.reg[0xf] = 0xff;
    c8.step(1);
    assert_eq!(c8.pc, pc + 4);

    c8.reset();
    c8.reg[0xf] = 0x0;
    c8.step(1);
    assert_eq!(c8.pc, pc + 2);
}

#[test]
fn test_instr_skip_next_if_not_equals_immediate() {
    let test_program: &[u8] = &[
        0x4f, 0xff,   // skip next if gp0 != FF
    ];

    let mut c8 = Chip8::new(test_program);
    let pc = c8.pc;
    c8.reg[0xf] = 0x00;
    c8.step(1);
    assert_eq!(c8.pc, pc + 4);

    c8.reset();
    c8.reg[0xf] = 0xff;
    c8.step(1);
    assert_eq!(c8.pc, pc + 2);
}

#[test]
fn test_instr_skip_next_if_not_equals_register() {
    let test_program: &[u8] = &[
        0x51, 0x20,   // skip next if gp1 == gp2
    ];

    let mut c8 = Chip8::new(test_program);
    let pc = c8.pc;
    c8.reg[0x1] = 0x55;
    c8.reg[0x2] = 0x55;
    c8.step(1);
    assert_eq!(c8.pc, pc + 4);

    c8.reset();
    assert_eq!(pc, c8.pc);
    c8.reg[0x1] = 0x55;
    c8.reg[0x2] = 0x44;
    c8.step(1);
    assert_eq!(c8.pc, pc + 2);
}

#[test]
fn test_instr_add_immediate() {
    let test_program: &[u8] = &[
        0x71, 0x34,   // add 0x34 to gp1
    ];

    let mut c8 = Chip8::new(test_program);
    let pc = c8.pc;
    c8.reg[1] = 0x12;
    c8.step(1);
    assert_eq!(c8.reg[1], 0x12 + 0x34);
    assert_eq!(c8.pc, pc + 2);

    // Test wrap-around
    let test_program: &[u8] = &[
        0x70, 0x02,   // skip next if gp0 == FF
    ];

    let mut c8 = Chip8::new(test_program);
    let _pc = c8.pc;
    c8.reg[0] = 0xff;
    c8.step(1);
    assert_eq!(c8.reg[0], 1);
}

#[test]
fn test_instr_alu_nop() {
    let test_program: &[u8] = &[
        0x81, 0x00,   // move gp0 to gp1
    ];

    let mut c8 = Chip8::new(test_program);
    let pc = c8.pc;
    c8.reg[0] = 0x12;
    c8.reg[1] = 0x34;
    c8.step(1);
    assert_eq!(c8.reg[0], 0x12);
    assert_eq!(c8.reg[1], 0x12);
    assert_eq!(c8.pc, pc + 2);
}

#[test]
fn test_instr_alu_and() {
    let test_program: &[u8] = &[
        0x81, 0x02,   // and gp0 with gp1, store in gp1
    ];

    let mut c8 = Chip8::new(test_program);
    let pc = c8.pc;
    c8.reg[0] = 0xaa;
    c8.reg[1] = 0xf0;
    c8.step(1);
    assert_eq!(c8.reg[0], 0xaa);
    assert_eq!(c8.reg[1], 0xa0);
    assert_eq!(c8.pc, pc + 2);
}

#[test]
fn test_instr_alu_or() {
    let test_program: &[u8] = &[
        0x81, 0x01,   // or gp0 with gp1, store in gp1
    ];

    let mut c8 = Chip8::new(test_program);
    let pc = c8.pc;
    c8.reg[0] = 0xaa;
    c8.reg[1] = 0x55;
    c8.step(1);
    assert_eq!(c8.reg[0], 0xaa);
    assert_eq!(c8.reg[1], 0xff);
    assert_eq!(c8.pc, pc + 2);
}

#[test]
fn test_instr_alu_xor() {
    let test_program: &[u8] = &[
        0x81, 0x03,   // xor gp0 with gp1, store in gp1
    ];

    let mut c8 = Chip8::new(test_program);
    let pc = c8.pc;
    c8.reg[0] = 0xaa;
    c8.reg[1] = 0x0f;
    c8.step(1);
    assert_eq!(c8.reg[0], 0xaa);
    assert_eq!(c8.reg[1], 0xa5);
    assert_eq!(c8.pc, pc + 2);
}

#[test]
fn test_instr_alu_add() {
    let test_program: &[u8] = &[
        0x81, 0x04,   // xor gp0 with gp1, store in gp1
    ];

    let mut c8 = Chip8::new(test_program);
    let pc = c8.pc;
    c8.reg[0] = 0x12;
    c8.reg[1] = 0x3;
    c8.step(1);
    assert_eq!(c8.reg[0], 0x12);
    assert_eq!(c8.reg[1], 0x12 + 0x3);
    assert_eq!(c8.pc, pc + 2);
    assert_eq!(c8.reg[15], 0);

    c8.reset();
    c8.reg[0] = 0xff;
    c8.reg[1] = 0x3;
    c8.step(1);
    assert_eq!(c8.reg[0], 0xff);
    assert_eq!(c8.reg[1], 0x2);
    assert_eq!(c8.pc, pc + 2);
    assert_eq!(c8.reg[15], 1);

}

#[test]
fn test_instr_alu_sub() {
    let test_program: &[u8] = &[
        0x81, 0x05,   // gp1 = gp1 - gp0
    ];

    let mut c8 = Chip8::new(test_program);
    let pc = c8.pc;
    c8.reg[0] = 2;
    c8.reg[1] = 5;
    c8.step(1);
    assert_eq!(c8.reg[0], 2);
    assert_eq!(c8.reg[1], 5 - 2);
    assert_eq!(c8.pc, pc + 2);
    assert_eq!(c8.reg[15], 0);

    c8.reset();
    c8.reg[0] = 5;
    c8.reg[1] = 2;
    c8.step(1);
    assert_eq!(c8.reg[0], 5);
    assert_eq!(c8.reg[1], 0xfd);
    assert_eq!(c8.pc, pc + 2);
    assert_eq!(c8.reg[15], 1);

}

#[test]
fn test_instr_alu_shift_right() {
    let test_program: &[u8] = &[
        0x80, 0x06,   // gp0 = gp0 >> 1
    ];

    let mut c8 = Chip8::new(test_program);
    let pc = c8.pc;
    c8.reg[0] = 2;
    c8.step(1);
    assert_eq!(c8.reg[0], 1);
    assert_eq!(c8.pc, pc + 2);
    assert_eq!(c8.reg[15], 0);

    c8.reset();
    c8.reg[0] = 0x81;
    c8.step(1);
    assert_eq!(c8.reg[0], 0x40);
    assert_eq!(c8.pc, pc + 2);
    assert_eq!(c8.reg[15], 1);

}

#[test]
fn test_instr_alu_subn() {
    let test_program: &[u8] = &[
        0x81, 0x07,   // gp1 = gp0 - gp1
    ];

    let mut c8 = Chip8::new(test_program);
    let pc = c8.pc;
    c8.reg[0] = 5;
    c8.reg[1] = 2;
    c8.step(1);
    assert_eq!(c8.reg[0], 5);
    assert_eq!(c8.reg[1], 5 - 2);
    assert_eq!(c8.pc, pc + 2);
    assert_eq!(c8.reg[15], 0);

    c8.reset();
    c8.reg[0] = 2;
    c8.reg[1] = 5;
    c8.step(1);
    assert_eq!(c8.reg[0], 2);
    assert_eq!(c8.reg[1], 0xfd);
    assert_eq!(c8.pc, pc + 2);
    assert_eq!(c8.reg[15], 1);

}

#[test]
fn test_instr_rnd() {
    let test_program: &[u8] = &[
        0xc0, 0xf0,
        0xc0, 0x0f,
        0xc1, 0xff,
        0xc1, 0xff,
        0xc1, 0xff,
        0xc1, 0xff,
    ];

    let mut c8 = Chip8::new(test_program);
    let pc = c8.pc;
    c8.reg[0] = 0xff;
    c8.step(1);
    assert_eq!(c8.pc, pc + 2);
    assert_eq!(c8.reg[0] & 0xf, 0);
    c8.reg[0] = 0xff;
    c8.step(1);
    assert_eq!(c8.reg[0] & 0xf0, 0);

    // make sure there's no more than 3 consequtive same numbers..
    c8.step(1);
    let r = c8.reg[1];
    for i in 1..4 {
        c8.step(1);
        if r != c8.reg[i] {
            break;
        }
        if i == 3 {
            assert!(false);
        }
    }
}

#[test]
fn test_instr_draw() {
    let test_program: &[u8] = &[
        0xd0, 0x11,
        0xd0, 0x11,
        0xd0, 0x18
    ];

    let mut c8 = Chip8::new(test_program);
    let pc = c8.pc;
    c8.i = 0x300;
    c8.memory[0x300..0x308].copy_from_slice(&[0xff; 8]);

    // Light up 8 top left pixels
    c8.step(1);

    for i in 0..8 {
        assert_eq!(c8.display[i], 255);
    }

    for i in 0..8 {
        assert_eq!(c8.display[Chip8::DISPLAY_WIDTH + i], 0);
    }

    assert_eq!(c8.pc, pc + 2);
    assert_eq!(c8.display[9], 0);
    assert_eq!(c8.reg[15], 0);      // no collision

    c8.step(1);
    assert_eq!(c8.reg[15], 1);      // should have collision
    for i in 0..9 {
        assert_eq!(c8.display[i], 0);
    }

    for i in 0..8 {
        assert_eq!(c8.display[Chip8::DISPLAY_WIDTH + i], 0);
    }

    // Test wrapping

    c8.display = [0; Chip8::DISPLAY_SIZE];
    c8.reg[0] = 61;
    c8.reg[1] = 30;

    c8.step(1);
    assert_eq!(c8.reg[15], 0);
//    dump_display(&c8);
    assert_rect(&c8.display, (0, 0, 5, 6), 255);
    assert_rect(&c8.display, (61, 0, 3, 6), 255);
    assert_rect(&c8.display, (61, 30, 3, 2), 255);
    assert_rect(&c8.display, (0, 30, 5, 2), 255);


}

#[test]
fn test_instr_draw_orientation() {
    let test_program: &[u8] = &[
        0xd0, 0x11,
    ];

    let mut c8 = Chip8::new(test_program);
    let _pc = c8.pc;
    c8.i = 0x300;
    c8.memory[0x300] = 0x0f;

    // Top left should be 00001111
    c8.step(1);

    dump_display(&c8);
    assert_rect(&c8.display, (0, 0, 4, 2), 0);
    assert_rect(&c8.display, (4, 0, 4, 1), 255);
    assert_rect(&c8.display, (4, 1, 4, 1), 0);
}

#[test]
fn test_instr_st_regs() {
    let test_program: &[u8] = &[
        0xf3, 0x55,
    ];

    let mut c8 = Chip8::new(test_program);
    let pc = c8.pc;
    c8.i = 0x300;
    for i in 0..4 {
        c8.reg[i] = i as u8;
    }

    c8.memory[0x300..0x308].copy_from_slice(&[0xff; 8]);
    c8.step(1);
    assert_eq!(c8.pc, pc + 2);
    assert_eq!(c8.i, 0x300 + 4);
    for i in 0..4 {
        assert_eq!(c8.memory[0x300 + i], i as u8);
    }

    assert_eq!(c8.memory[0x300 + 4], 255);
}



#[test]
fn test_instr_ld_regs() {
    let test_program: &[u8] = &[
        0xf3, 0x65,
    ];

    let mut c8 = Chip8::new(test_program);
    let pc = c8.pc;
    c8.i = 0x300;
    for i in 0..4 {
        c8.memory[0x300 + i] = i as u8;
    }

    c8.reg = [0xff; 16];
    c8.step(1);
    assert_eq!(c8.pc, pc + 2);
    assert_eq!(c8.i, 0x300 + 4);
    for i in 0..4 {
        assert_eq!(c8.reg[i], i as u8);
    }

    assert_eq!(c8.reg[4], 0xff);
}

#[test]
fn test_instr_add_reg_to_i() {
    let test_program: &[u8] = &[
        0xfe, 0x1e,   // i = i + gp14
    ];

    let mut c8 = Chip8::new(test_program);
    let pc = c8.pc;
    c8.i = 0x123;
    c8.reg[14] = 2;
    c8.step(1);
    assert_eq!(c8.reg[14], 2);
    assert_eq!(c8.pc, pc + 2);
    assert_eq!(c8.i, 0x123 + 2);
}

#[test]
fn test_instr_skip_next_if_key_not_pressed() {
    let test_program: &[u8] = &[
        0xe0, 0xa1,   // skip next if keys[gp1]
    ];

    let mut c8 = Chip8::new(test_program);
    let pc = c8.pc;
    c8.reg[0x0] = 'a' as u8;
    c8.step(1);
    assert_eq!(c8.pc, pc + 4);

    c8.reset();
    c8.reg[0x0] = 'a' as u8;
    c8.keys['a' as usize] = 255;
    c8.step(1);
    assert_eq!(c8.pc, pc + 2);
}

#[test]
fn test_instr_skip_next_if_key_pressed() {
    let test_program: &[u8] = &[
        0xe0, 0x9e,   // skip next if keys[gp1]
    ];

    let mut c8 = Chip8::new(test_program);
    let pc = c8.pc;
    c8.reg[0x0] = 'a' as u8;
    c8.step(1);
    assert_eq!(c8.pc, pc + 2);

    c8.reset();
    c8.reg[0x0] = 'a' as u8;
    c8.keys['a' as usize] = 255;
    c8.step(1);
    assert_eq!(c8.pc, pc + 4);
}



fn assert_rect(d: &[u8], rect: (usize, usize, usize, usize), v: u8) {
    for y in rect.1 .. rect.1 + rect.3 {
        for x in rect.0 .. rect.0 + rect.2 {
            assert_eq!(d[y * Chip8::DISPLAY_WIDTH + x], v);
        }
    }
}