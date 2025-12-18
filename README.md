# Chip-8

## Emulador em Rust

### Referências

- [Another Chip-8 Emulator (in Rust)](https://r3zz.io/posts/rust-chip8-emulator/#emulators)
- [Building a CHIP-8 Emulator in Rust ](https://dev.to/trish_07/building-a-chip-8-emulator-in-rust-an-advanced-adventure-1kb4)
- [Building a CHIP-8 Emulator (C++)](https://austinmorlan.com/posts/chip8_emulator/)

### O que é um emulador Chip-8?

> At its essence, an emulator is a software or hardware tool that replicates the functions of one system (the emulated system) on another system (the host system). It enables the host to run software, games, or applications designed for the emulated system, allowing users to experience or test software on different platforms without the need for the original hardware.

### Como uma CPU funciona?

> As explained above, the CHIP-8 was not a real physical CPU but instead a virtual machine with its own instruction set, but the same principles apply. For example, consider the CHIP-8 instruction **$7522**. The first byte (**$75**) says **ADD to Register 5** and the second byte (**$22**) is the value to be added to Register 5. So this instruction says **ADD $22 to Register 5**.

> Way back in the day, programmers would write programs in an **assembly language** rather than the high-level languages (like C++) that we use today. Assembly is as low as you can go while still being human readable. An **assembler** would translate their human-readable assembly into the 1s and 0s that the computer could understand.

> Keeping with the earlier example, the assembly program would have **ADD V5, $22**, and the assembler would translate that to **$7522**, which the CHIP-8 interpreter can read.

$22 = 0x22 = 34 = 100010

### Estrutura do emulador

- [Chip-8 Design Specification](https://www.cs.columbia.edu/~sedwards/classes/2016/4840-spring/designs/Chip8.pdf)
#### Memória

- 4KB de RAM (4096 bytes)
	- 0x000 - 0x1FF são reservados (512 bytes)
	- 0x050 - 0x0A0 é o espaço para a fonte padrão (0 até F)
	- 0x200 - 0xFFF disponíveis

![](https://r3zz.io/images/chip8memory.png)


#### Registros

- 16 registros gerais com tamanho de 8 bits (1 byte)
	- *V0, V1, V2 ... VF*
	- VF registra resultados de certas operações

- *I* (Index)- Armazena endereços (16 bits para 4096KB)

- *PC* (Program Counter) - Endereço da instrução (16 bits)
	- Cada instrução incrementa 2 (16 bits ou 2 bytes)

- *SP* (Stack Pointer) - Topo do stack (8 bits)

- *DT* (Delay Timer) - Reduz o valor até 0 num rate de 60Hz (8 bits)

- *ST* (Sound Timer) - Igual DT, mas emite um tom quando não zero (8 bits)


#### Stack

> A stack is a way for a CPU to keep track of the order of execution when it calls into functions. There is an instruction (**CALL**) that will cause the CPU to begin executing instructions in a different region of the program. When the program reaches another instruction (**RET**), it must be able to go back to where it was when it hit the CALL instruction. The stack holds the PC value when the CALL instruction was executed, and the RETURN statement pulls that address from the stack and puts it back into the PC so the CPU will execute it on the next cycle.

- 16 níveis de 16 bits (registra o PC)

```text
$200: CALL $208
$202: JMP $20E
$204: LD V1, $1
$206: RET
$208: LD V3, $3
$20A: CALL $204
$20C: RET
$20E: LD V4, $4
```

```text
$200: CALL $208 -> PC += 2 = $202 | SP = 0 | Put $202 in stack[0] and ++SP = 1    | PC = $208 | Next cycle at PC = $208
$208: LD V3, $3 -> PC += 2 = $20A | SP = 1 | Do not modify stack or SP            | PC = $20A | Next cycle at PC = $20A
$20A: CALL $204 -> PC += 2 = $20C | SP = 1 | Put $20C on stack[1] and ++SP = 2    | PC = $204 | Next cycle at PC = $204
$204: LD V1, $1 -> PC += 2 = $206 | SP = 2 | Do not modify stack or SP            | PC = $206 | Next cycle at PC = $206
$206: RET       -> PC += 2 = $208 | SP = 2 | --SP = 1 and Pull $20C from stack[1] | PC = $20C | Next cycle at PC = $20C
$20C: RET       -> PC += 2 = $20E | SP = 0 | --SP = 0 and Pull $202 from stack[0] | PC = $202 | Next cycle at PC = $202
$202: JMP $20E  -> PC += 2 = $204 | SP = 0 | Do not modify stack or SP            | PC = $204 | Next cycle at PC = $204
$20E: LD V4, $4 -> PC += 2 = $210 | SP = 0 | Do not modify stack or SP            | PC = $210 | Next cycle at PC = $210
```

#### Keyboard

- 16 teclas (0 - F)

```text
Keypad       Keyboard
+-+-+-+-+    +-+-+-+-+
|1|2|3|C|    |1|2|3|4|
+-+-+-+-+    +-+-+-+-+
|4|5|6|D|    |Q|W|E|R|
+-+-+-+-+ => +-+-+-+-+
|7|8|9|E|    |A|S|D|F|
+-+-+-+-+    +-+-+-+-+
|A|0|B|F|    |Z|X|C|V|
+-+-+-+-+    +-+-+-+-+
```

#### Display

- Monocromático 64 x 32
	-  Pixel on/off
	- Sprites dão a volta se fora da tela

![](https://r3zz.io/images/displaydrawing.png)

> The draw instruction iterates over each pixel in a sprite and XORs the sprite pixel with the display pixel.
	- Sprite Pixel Off XOR Display Pixel Off = Display Pixel Off
	- Sprite Pixel Off XOR Display Pixel On = Display Pixel On
	- Sprite Pixel On XOR Display Pixel Off = Display Pixel On
	- Sprite Pixel On XOR Display Pixel On = Display Pixel Off

#### Fonte

- Exemplo - Letra A:

| 1   | 1   | 1   | 1   |
| --- | --- | --- | --- |
| 1   |     |     | 1   |
| 1   | 1   | 1   | 1   |
| 1   |     |     | 1   |
| 1   |     |     | 1   |
```text
1111 0000 = 0xF0
1001 0000 = 0x90
1111 0000 = 0xF0
1001 0000 = 0x90
1001 0000 = 0x90
```

```rust
const FONTSET: [u8; 80] = [
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
```

#### Instruções

> - 34 opcodes - 16 bits
>   - n n n n (0xF000, 0x0F00, 0x00F0, 0x00F)
>   - n = "nibble" (4 bits)
>   - x, y = registros (4 = V4)
>   - kk = valor imediato (8 bits)

- **0nnn - SYS addr**
Jump to a machine code routine at nnn. This instruction is only used on the old computers on which Chip-8 was originally implemented. It is ignored by modern interpreters. This will not be implemented.

- **00E0 - CLS**
Clear the display.

- **00EE - RET**
Return from a subroutine.The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.

- **1nnn - JP addr**
Jump to location nnn. The interpreter sets the program counter to nnn.

- **2nnn - CALL addr**
Call subroutine at nnn. The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.

- **3xkk - SE Vx, byte**
Skip next instruction if Vx = kk. The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.

- **4xkk - SNE Vx, byte**
Skip next instruction if Vx != kk. The interpreter compares register Vx to kk, and if they are not equal,increments the program counter by 2.

- **5xy0 - SE Vx, Vy**
Skip next instruction if Vx = Vy. The interpreter compares register Vx to register Vy, and if they are equal,increments the program counter by 2.

- **6xkk - LD Vx, byte**
Set Vx = kk. The interpreter puts the value kk into register Vx.

- **7xkk - ADD Vx, byte**
Set Vx = Vx + kk. Adds the value kk to the value of register Vx, then stores the result in Vx.

- **8xy0 - LD Vx, Vy**
Set Vx = Vy. Stores the value of register Vy in register Vx.

- **8xy1 - OR Vx, Vy**
Set Vx = Vx OR Vy. Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx. Abitwise OR compares the corresponding bits from two values, and if either bit is 1, then the same bit in theresult is also 1. Otherwise, it is 0.

- **8xy2 - AND Vx, Vy**
Set Vx = Vx AND Vy. Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx. A bitwise AND compares the corresponding bits from two values, and if both bits are 1, then the same bit in the result is also 1. Otherwise, it is 0.

- **8xy3 - XOR Vx, Vy**
Set Vx = Vx XOR Vy. Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx. An exclusive OR compares the corresponding bits from two values, and if the bits are not both the same, then the corresponding bit in the result is set to 1. Otherwise, it is 0.

- **8xy4 - ADD Vx, Vy**
Set Vx = Vx + Vy, set VF = carry. The values of Vx and Vy are added together. If the result is greater than 8 bits (i.e., ¿ 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.

- **8xy5 - SUB Vx, Vy**
Set Vx = Vx - Vy, set VF = NOT borrow. If Vx ¿ Vy, then VF is set to 1, otherwise 0. Then Vy is
subtracted from Vx, and the results stored in Vx.

- **8xy6 - SHR Vx {, Vy}**
Set Vx = Vx SHR 1. If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.

- **8xy7 - SUBN Vx, Vy**
Set Vx = Vy - Vx, set VF = NOT borrow. If Vy ¿ Vx, then VF is set to 1, otherwise 0. Then Vx is
subtracted from Vy, and the results stored in Vx.

- **8xyE - SHL Vx {, Vy}**
Set Vx = Vx SHL 1. If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.

- **9xy0 - SNE Vx, Vy**
Skip next instruction if Vx != Vy. The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.

- **Annn - LD I, addr**
Set I = nnn. The value of register I is set to nnn.

- **Bnnn - JP V0, addr**
Jump to location nnn + V0. The program counter is set to nnn plus the value of V0.

- **Cxkk - RND Vx, byte**
Set Vx = random byte AND kk. The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk. The results are stored in Vx. See instruction 8xy2 for more information on AND.

- **Dxyn - DRW Vx, Vy, nibble**
Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision. The interpreter reads n bytes from memory, starting at the address stored in I. These bytes are then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XOR’d onto the existing screen. If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0. If the sprite is positioned so part of it is outside the coordinates ofthe display, it wraps around to the opposite side of the screen.

- **Ex9E - SKP Vx**
Skip next instruction if key with the value of Vx is pressed. Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, PC is increased by 2.

- **ExA1 - SKNP Vx**
Skip next instruction if key with the value of Vx is not pressed. Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, PC is increased by 2.

- **Fx07 - LD Vx, DT**
Set Vx = delay timer value. The value of DT is placed into Vx.

- **Fx0A - LD Vx, K**
Wait for a key press, store the value of the key in Vx. All execution stops until a key is pressed, then the value of that key is stored in Vx.

- **Fx15 - LD DT, Vx**
Set delay timer = Vx. Delay Timer is set equal to the value of Vx.

- **Fx18 - LD ST, Vx**
Set sound timer = Vx. Sound Timer is set equal to the value of Vx.

- **Fx1E - ADD I, Vx**
Set I = I + Vx. The values of I and Vx are added, and the results are stored in I.

- **Fx29 - LD F, Vx**
Set I = location of sprite for digit Vx. The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx. See section 2.4, Display, for more information on the Chip-8 hexadecimal font. To obtain this value, multiply VX by 5 (all font data stored in first 80 bytes of memory).

- **Fx33 - LD B, Vx**
Store BCD representation of Vx in memory locations I, I+1, and I+2. The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.

- **Fx55 - LD \[I], Vx**
Stores V0 to VX in memory starting at address I. I is then set to I + x + 1.

- **Fx65 - LD Vx, \[I]**
Fills V0 to VX with values from memory starting at address I. I is then set to I + x + 1.

### Implementação

#### Cargo.toml

```toml
[package]
name = "chip8"
version = "0.1.0"
edition = "2024"

[dependencies]
minifb = "0.28"
rand = "0.9.2"
rodio = "0.21"
```
#### Files

```text
chip8-emulator/
│
├── src/
│   ├── audio.rs
│   ├── cpu.rs
│   ├── main.rs
│   └── window.rs
│
├── Cargo.toml
└── README.md
```
- [Audio](src/audio.rs)
- [Window](src/window.rs)
- [CPU](src/cpu.rs)
- [Main](src/main.rs)