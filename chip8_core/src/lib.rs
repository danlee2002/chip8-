use rand::Rng;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const START_ADDR: u16 = 0x200;
const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
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

#[derive(Debug)]
pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        let op = self.fetch();
        self.execute(op);
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // to implement
            }
            self.st -= 1;
        }
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            // NOP
            (0, 0, 0, 0) => return,
            // CLS
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }
            // RET
            (0, 0, 0xE, 0xE) => self.reg(),
            // JMP NNN
            (1, _, _, _) => self.jmp_nnn(op),
            // CALL NNN
            (2, _, _, _) => self.call_nnn(op),
            // SKIP V[X] == NN
            (3, _, _, _) => self.skip_vx_eqnn(op, digit2),
            // SKIP V[X] != NN
            (4, _, _, _) => self.skip_vx_neqenn(op, digit2),
            // SKIP V[X] == V[Y]
            (5, _, _, _) => self.skip_vx_eqvy(digit2, digit3),
            // V[X] = NN
            (6, _, _, _) => self.vx_eqnn(op, digit2),
            // V[X] += NN
            (7, _, _, _) => self.vx_plusnn(op, digit2),
            // V[X] = V[Y]
            (8, _, _, 0) => self.vx_eq_vy(digit2, digit3),
            // V[X] |= V[Y]
            (8, _, _, 1) => self.vx_or_vy(digit2, digit3),
            // V[X] &= V[Y]
            (8, _, _, 2) => self.vx_and_vy(digit2, digit3),
            // V[X] ^= V[Y]
            (8, _, _, 3) => self.vx_xor_vy(digit2, digit3),
            // V[X] += V[Y]
            (8, _, _, 4) => self.vx_plus_eqvy(digit2, digit3),
            // V[X] -= V[Y]
            (8, _, _, 5) => self.vx_minus_eqvy(digit2, digit3),
            // V[X] >>= 1
            (8, _, _, 6) => self.vx_bitshiftright(digit2),
            // V[X] = V[Y] - V[X]
            (8, _, _, 7) => self.vx_eqvy_minusvx(digit2, digit3),
            // V[X] <<= 1
            (8, _, _, 0xE) => self.vx_bitshiftleft(digit2),
            // SKIP V[X] != V[Y]
            (9, _, _, 0) => self.skip_eq_vx_neqvy(digit2, digit3),
            // I = NNN
            (0xA, _, _, _) => self.i_eq_nnn(op),
            // JMP V[0] + NNN
            (0xB, _, _, _) => self.jmp_vzero_plusnnn(op),
            // V[X] = rand() & NN
            (0xC, _, _, _) => self.vx_eqrand_and_nnn(op, digit2),
            // DRAW
            (0xD, _, _, _) => self.draw(digit2, digit3, digit4),
            // SKIP KEY PRESS
            (0xE, _, 9, 0xE) => self.skip_keypress(digit2),
            // SKIP KEY RELEASE
            (0xE, _, 0xA, 1) => self.skip_keyrelease(digit2),
            // V[X] = DT
            (0xF, _, 0, 7) => self.vx_eq_delaytimer(digit2),
            // WAIT KEY
            (0xF, _, 0, 0xA) => self.wait(digit2),
            // DT = V[X]
            (0xF, _, 1, 5) => self.delaytimer_eq_vx(digit2),
            // ST = V[X]
            (0xF, _, 1, 8) => self.soundtimer_eq_vx(digit2),
            // I += V[X]
            (0xF, _, 1, 0xE) => self.instruction_plus_eq_vx(digit2),
            // I = FONT
            (0xF, _, 2, 9) => self.i_eq_font(digit2),
            // BCD
            (0xF, _, 3, 3) => self.bcd(digit2),
            // STORE V[0] - V[X]
            (0xF, _, 5, 5) => self.store_v0_vx(digit2),
            // LOAD V[0] - V[X]
            (0xF, _, 6, 5) => self.ld_v0_vx(digit2),
            (_, _, _, _) => unimplemented!("unimplemented opcode: {:#04x}", op),
        }
    }

    // functions for opcodes
    fn ret(&mut self) {
        let ret_addr = self.pop();
        self.pc = ret_addr;
    }

    fn jmp_nnn(&mut self, op: u16) {
        self.pc = op & 0xFFF;
    }

    fn call_nnn(&mut self, op: u16) {
        let nnn = op & 0xFFF;
        self.push(self.pc);
        self.pc = nnn;
    }

    fn skip_vx_eqnn(&mut self, op: u16, x: u16) {
        let nn = (op & 0xFF) as u8;
        if self.v_reg[x as usize] == nn {
            self.pc += 2;
        }
    }

    fn skip_vx_neqenn(&mut self, op: u16, x: u16) {
        let nn = (op & 0xFF) as u8;
        if self.v_reg[x as usize] != nn {
            self.pc += 2;
        }
    }

    fn skip_vx_eqvy(&mut self, x: u16, y: u16) {
        if self.v_reg[x as usize] == self.v_reg[y as usize] {
            self.pc += 2;
        }
    }

    fn vx_eqnn(&mut self, op: u16, x: u16) {
        let nn = (op & 0xFF) as u8;
        self.v_reg[x as usize] = nn;
    }

    fn vx_plusnn(&mut self, op: u16, x: u16) {
        let nn = (op & 0xFF) as u8;
        self.v_reg[x as usize] = self.v_reg[x as usize].wrapping_add(nn);
    }

    fn vx_eq_vy(&mut self, x: u16, y: u16) {
        self.v_reg[x as usize] = self.v_reg[y as usize];
    }

    fn vx_or_vy(&mut self, x: u16, y: u16) {
        self.v_reg[x as usize] |= self.v_reg[y as usize];
    }

    fn vx_and_vy(&mut self, x: u16, y: u16) {
        self.v_reg[x as usize] &= self.v_reg[y as usize];
    }

    fn vx_xor_vy(&mut self, x: u16, y: u16) {
        self.v_reg[x as usize] ^= self.v_reg[y as usize];
    }
    fn vx_plus_eqvy(&mut self, x: u16, y: u16) {
        let (new_vx, carry) = self.v_reg[x as usize].overflowing_add(self.v_reg[y as usize]);
        let new_vf = if carry { 1 } else { 0 };
        self.v_reg[x as usize] = new_vx;
        self.v_reg[0xF] = new_vf;
    }

    fn vx_minus_eqvy(&mut self, x: u16, y: u16) {
        let (new_vx, borrow) = self.v_reg[x as usize].overflowing_sub(self.v_reg[y as usize]);
        let new_vf = if borrow { 0 } else { 1 };
        self.v_reg[x as usize] = new_vx;
        self.v_reg[0xF] = new_vf;
    }

    fn vx_bitshiftright(&mut self, x: u16) {
        let lsb = self.v_reg[x as usize] & 1;
        self.v_reg[x as usize] >>= 1;
        self.v_reg[0xF] = lsb;
    }

    fn vx_eqvy_minusvx(&mut self, x: u16, y: u16) {
        let (new_vx, borrow) = self.v_reg[y as usize].overflowing_sub(self.v_reg[x as usize]);
        let new_vf = if borrow { 0 } else { 1 };
        self.v_reg[x as usize] = new_vx;
        self.v_reg[0xF] = new_vf;
    }

    fn vx_bitshiftleft(&mut self, x: u16) {
        let msb = (self.v_reg[x as usize] >> 7) & 1;
        self.v_reg[x as usize] <<= 1;
        self.v_reg[0xF] = msb;
    }

    fn skip_eq_vx_neqvy(&mut self, x: u16, y: u16) {
        if self.v_reg[x as usize] != self.v_reg[y as usize] {
            self.pc += 2;
        }
    }

    fn i_eq_nnn(&mut self, op: u16) {
        let nnn = op & 0xFFF;
        self.i_reg = nnn;
    }

    fn jmp_vzero_plusnnn(&mut self, op: u16) {
        let nnn = op & 0xFFF;
        self.pc = (self.v_reg[0] as u16) + nnn;
    }

    fn vx_eqrand_and_nnn(&mut self, op: u16, x: u16) {
        let nn = (op & 0xFF) as u8;
        let rng: u8 = rand::thread_rng().gen();
        self.v_reg[x as usize] = rng & nn;
    }

    fn draw(&mut self, x: u16, y: u16, rows: u16) {
        let x_coord = self.v_reg[x as usize] as u16;
        let y_coord = self.v_reg[y as usize] as u16;
        let num_rows = rows;

        let mut flipped = false;
        for y_line in 0..num_rows {
            let addr = self.i_reg + y_line as u16;
            let pixels = self.ram[addr as usize];
            for x_line in 0..8 {
                if (pixels & (0b1000_0000 >> x_line)) != 0 {
                    let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                    let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                    let idx = x + SCREEN_WIDTH * y;
                    flipped |= self.screen[idx];
                    self.screen[idx] ^= true;
                }
            }
        }
        if flipped {
            self.v_reg[0xF] = 1;
        } else {
            self.v_reg[0xF] = 0;
        }
    }

    fn skip_keypress(&mut self, x: u16) {
        let vx: u8 = self.v_reg[x as usize];
        let key = self.keys[vx as usize];
        if key {
            self.pc += 2;
        }
    }

    fn skip_keyrelease(&mut self, x: u16) {
        let vx = self.v_reg[x as usize];
        let key = self.keys[vx as usize];
        if !key {
            self.pc += 2;
        }
    }

    fn vx_eq_delaytimer(&mut self, x: u16) {
        self.v_reg[x as usize] = self.dt;
    }

    fn wait(&mut self, x: u16) {
        let mut pressed = false;
        for i in 0..self.keys.len() {
            if self.keys[i] {
                self.v_reg[x as usize] = i as u8;
                pressed = true;
                break;
            }
        }
        if !pressed {
            self.pc -= 2;
        }
    }

    fn delaytimer_eq_vx(&mut self, x: u16) {
        self.dt = self.v_reg[x as usize];
    }

    fn soundtimer_eq_vx(&mut self, x: u16) {
        self.st = self.v_reg[x as usize];
    }

    fn instruction_plus_eq_vx(&mut self, x: u16) {
        let vx = self.v_reg[x as usize] as u16;
        self.i_reg = self.i_reg.wrapping_add(vx);
    }

    fn i_eq_font(&mut self, x: u16) {
        let c = self.v_reg[x as usize] as u16;
        self.i_reg = c * 5;
    }

    fn bcd(&mut self, x: u16) {
        let vx = self.v_reg[x as usize] as f32;
        let hundreds = (vx / 100.0).floor() as u8;
        let tens = ((vx / 10.0) % 10.0).floor() as u8;
        let ones = (vx % 10.0) as u8;
        self.ram[self.i_reg as usize] = hundreds;
        self.ram[(self.i_reg + 1) as usize] = tens;
        self.ram[(self.i_reg + 2) as usize] = ones;
    }

    fn store_v0_vx(&mut self, x: u16) {
        let x = x as usize;
        let i = self.i_reg as usize;
        for idx in 0..=x {
            self.ram[i + idx] = self.v_reg[idx];
        }
    }

    fn ld_v0_vx(&mut self, x: u16) {
        let x = x as usize;
        let i = self.i_reg as usize;
        for idx in 0..=x {
            self.v_reg[idx] = self.ram[i + idx];
        }
    }
    #[cfg(test)]
    pub fn get_pc(&mut self) -> u16 {
        self.pc
    }
    #[cfg(test)]
    pub fn get_ram(&mut self) -> &[u8] {
        &self.ram
    }
    #[cfg(test)]
    pub fn get_v_reg(&mut self) -> &[u8] {
        &self.v_reg
    }
    #[cfg(test)]
    pub fn get_i_reg(&mut self) -> u16 {
        self.i_reg
    }
    #[cfg(test)]
    pub fn get_sp(&mut self) -> u16 {
        self.sp
    }
    #[cfg(test)]
    pub fn get_stack(&mut self) -> &[u16] {
        &self.stack
    }
    #[cfg(test)]
    pub fn get_keys(&mut self) -> &[bool] {
        &self.keys
    }
    #[cfg(test)]
    pub fn get_dt(&mut self) -> u8 {
        self.dt
    }

    #[cfg(test)]
    pub fn get_st(&mut self) -> u8 {
        self.st
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // tests initalization
    #[test]
    fn test_initialization() {
        let mut emu = Emu::new();
        let mut true_ram = [0; 4096];
        true_ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        assert_eq!(emu.get_pc(), 0x200);
        assert_eq!(emu.get_ram(), true_ram);
        assert_eq!(emu.get_v_reg(), [0; 16]);
        assert_eq!(emu.get_i_reg(), 0);
        assert_eq!(emu.get_sp(), 0);
        assert_eq!(emu.get_stack(), [0; 16]);
        assert_eq!(emu.get_keys(), [false; 16]);
        assert_eq!(emu.get_dt(), 0);
        assert_eq!(emu.get_st(), 0);
    }

    // test pushing onto stack counter
    #[test]
    fn test_push() {
        let mut emu = Emu::new();
        let mut expected = [0; 16];
        for i in 0..16 {
            emu.push(i);
            expected[i as usize] = i as u16;
            assert_eq!(emu.get_sp(), i + 1);
            assert_eq!(emu.get_stack(), expected);
        }
    }

    //tests sp
    #[test]
    fn test_pop() {
        let mut emu = Emu::new();
        let mut expected = [0; 16];
        for i in 0..16 {
            emu.push(i);
            expected[i as usize] = i as u16;
        }
        for i in (0..16).rev() {
            let val = emu.pop();
            assert_eq!(val, i);
            assert_eq!(emu.get_sp(), i);
        }
    }
}
