/*
The file we write out has the following layout:

     0x0 +----------------------+
         |     ELF64 header     |
    0x40 +----------------------+
         | program header table |
         | PT_LOAD .text        |
         | PT_LOAD .rodata      |
         | PT_LOAD .data + .bss |
    0xe8 +----------------------+
         |  pad to align 4096   |
  0x1000 +----------------------+ <- text_off    \
         |        .text         |                |
         +----------------------+                | This block is loaded contiguously
         |  pad to align 4096   |                | into the process' address space by
 0x??000 +----------------------+ <- rodata_off  | three PT_LOADs, documented below.
         |       .rodata        |                |
         +----------------------+                |
         |  pad to align 4096   |                |
 0x??000 +----------------------+ <- data_off    |
         |        .data         |                |
         +----------------------+                |
         |  pad to align 16     |                |
 0x????0 +----------------------+ <- strtab_off  /
         |       .strtab        |
         +----------------------+
         |  pad to align 16     |
 0x????0 +----------------------+ <- e_shoff
         | section header table |
         |   section: null      |
         |   section: .text     |
         |   section: .rodata   |
         |   section: .data     |
         |   section: .bss      |
         |   section: .strtab   |
         +----------------------+

Once the binary is loaded, our memory layout looks like the following, where ... stands for
the padding between sections as described above.

LOAD_BASE
   |             LOAD_BASE + (rodata_off - text_off)
   |               |               LOAD_BASE + (data_off - text_off)
   v               V                 v
   +-------+-------+---------+-------+-------+-------+------+-------+
   | .text |  ...  | .rodata |  ...  | .data |  ...  | .bss |  ...  |
   +-------+-------+---------+-------+-------+-------+------+-------+
    =============== ================= ==============================
         X + R              R                     R + W

This is achieved via the above three PT_LOADs, setting flags X + R, R, and R + W respectively.
Note that a single load handles both .data and .bss because we simply set the size in memory
for the load larger than the size in the ELF, which achieves a zero-initialized .bss section.

We're mapping the file position text_off to the memory location LOAD_BASE, and then mapping file
bytes into memory bytes continuously thereafter. Thus, the byte at file position x goes to the
memory address LOAD_BASE + (x - text_off).

Our loaded sections have the following alignments for the user:

   section   alignment
  ---------|-----------
   .text   | 4096
   .rodata | 4096
   .data   | 4096
   .bss    | 16
*/

use std::io::prelude::*;
use std::os::unix::fs::PermissionsExt;

pub enum SectionName {
    Instructions, ROData, Data, BSS,
}

pub struct RelocationRef {
    section: SectionName,
    offset: u64,
}

pub struct RelocationEntry {
    section: SectionName,
}

#[derive(Default)]
pub struct Assembler {
    pub instructions: Vec<u8>,
    pub rodata: Vec<u8>,
    pub data: Vec<u8>,
    pub bss_size: u64,

}

static PF_X: u32 = 0x1;
static PF_W: u32 = 0x2;
static PF_R: u32 = 0x4;

static SHF_WRITE:     u64 = 0x1;
static SHF_ALLOC:     u64 = 0x2;
static SHF_EXECINSTR: u64 = 0x4;
static SHF_STRINGS:   u64 = 0x20;

static SHT_PROGBITS: u32 = 1;
static SHT_STRTAB:   u32 = 3;
static SHT_NOBITS:   u32 = 8;

static LOAD_BASE: u64 = 0x400000;

fn round_up_to_next_multiple(x: u64, modulus: u64) -> u64 {
    return x + modulus - 1 - (x + modulus - 1) % modulus;
}

type Register = u8;

pub enum RegisterSize {
    Bits8, Bits16, Bits32, Bits64,
}

pub static REG_RAX: Register = 0;
pub static REG_RCX: Register = 1;
pub static REG_RDX: Register = 2;
pub static REG_RBX: Register = 3;
pub static REG_RSP: Register = 4;
pub static REG_RBP: Register = 5;
pub static REG_RSI: Register = 6;
pub static REG_RDI: Register = 7;
pub static REG_R8:  Register = 8;
pub static REG_R9:  Register = 9;
pub static REG_R10: Register = 10;
pub static REG_R11: Register = 11;
pub static REG_R12: Register = 12;
pub static REG_R13: Register = 13;
pub static REG_R14: Register = 14;
pub static REG_R15: Register = 15;

impl Assembler {
    pub fn op_lit(&mut self, size: RegisterSize, reg: Register, literal: u64) {        
        // TODO
    }

    pub fn op_mov(&mut self, size: RegisterSize, dst: Register, src: Register) {
        // TODO
    }

    pub fn op_syscall(&mut self) {
        self.instructions.extend(b"\x0f\x05");
    }

    pub fn write_to_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        fn write_datum<T: Sized>(f: &mut std::io::BufWriter<std::fs::File>, x: T) -> std::io::Result<()> {
            let data: &[u8] = unsafe {
                std::slice::from_raw_parts(
                    (&x as *const T) as *const u8,
                    std::mem::size_of::<T>(),
                )
            };
            f.write(data)?;
            Ok(())
        }
        
        fn write_pt_load(
            f: &mut std::io::BufWriter<std::fs::File>,
            p_flags: u32,
            p_offset: u64,
            p_vaddr: u64,
            p_filesz: u64,
            p_memsz: u64,
            p_align: u64,
        ) -> std::io::Result<()> {
            write_datum(f, 1        as u32)?; // 4: p_type (1 = PT_LOAD)
            write_datum(f, p_flags  as u32)?; // 4: p_flags
            write_datum(f, p_offset as u64)?; // 8: p_offset
            write_datum(f, p_vaddr  as u64)?; // 8: p_vaddr
            write_datum(f, p_vaddr  as u64)?; // 8: p_paddr
            write_datum(f, p_filesz as u64)?; // 8: p_filesz
            write_datum(f, p_memsz  as u64)?; // 8: p_memsz
            write_datum(f, p_align  as u64)?; // 8: p_align
            Ok(())
        }
        
        fn write_section_header_table_entry(
            f: &mut std::io::BufWriter<std::fs::File>,
            sh_name: u32,
            sh_type: u32,
            sh_flags: u64,
            sh_addr: u64,
            sh_offset: u64,
            sh_size: u64,
            sh_addralign: u64,
        ) -> std::io::Result<()> {
            write_datum(f, sh_name      as u32)?; // 4: sh_name
            write_datum(f, sh_type      as u32)?; // 4: sh_type
            write_datum(f, sh_flags     as u64)?; // 8: sh_flags
            write_datum(f, sh_addr      as u64)?; // 8: sh_addr
            write_datum(f, sh_offset    as u64)?; // 8: sh_offset
            write_datum(f, sh_size      as u64)?; // 8: sh_size
            write_datum(f, 0            as u32)?; // 4: sh_link
            write_datum(f, 0            as u32)?; // 4: sh_info
            write_datum(f, sh_addralign as u64)?; // 8: sh_addralign
            write_datum(f, 0            as u64)?; // 8: sh_entsize
            Ok(())
        }

        let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);

        let strtab = "\0.text\0.rodata\0.data\0.bss\0.strtab\0";

        // Compute the byte offsets of the key points in the file.
        let text_off:   u64 = 0x1000;
        let text_len:   u64 = self.instructions.len() as u64;
        let rodata_off: u64 = round_up_to_next_multiple(text_off + text_len, 4096);
        let rodata_len: u64 = self.rodata.len() as u64;
        let data_off:   u64 = round_up_to_next_multiple(rodata_off + rodata_len, 4096);
        let data_len:   u64 = self.data.len() as u64;
        let strtab_off: u64 = round_up_to_next_multiple(data_off + data_len, 16);
        let strtab_len: u64 = strtab.len() as u64;
        let e_shoff:    u64 = round_up_to_next_multiple(strtab_off + strtab_len, 16);

        // We use a single R+W PT_LOAD to make both the .data and .bss sections.
        // The in-memory result of the load should look like:
        //   +-------+-----------+------+-----------+
        //   | .data | pad to 16 | .bss | pad to 16 |
        //   +-------+-----------+------+-----------+
        // Here we compute the total length of the above.
        let total_rw_load_size = round_up_to_next_multiple(
            round_up_to_next_multiple(data_len, 16) + self.bss_size, 16,
        );

        // Write the ELF64 header.
        f.write(b"\x7fELF")?;                   // 4: e_ident[EI_MAG]
        f.write(b"\x02")?;                      // 1: e_ident[EI_CLASS]      (2 = 64-bit)
        f.write(b"\x01")?;                      // 1: e_ident[EI_DATA]       (1 = little-endian)
        f.write(b"\x01")?;                      // 1: e_ident[EI_VERSION]
        f.write(b"\0")?;                        // 1: e_ident[EI_OSABI]      (0 = System V)
        f.write(b"\0")?;                        // 1: e_ident[EI_ABIVERSION]
        f.write(b"\0\0\0\0\0\0\0")?;            // 7: padding
        write_datum(&mut f, 2         as u16)?; // 2: e_type                 (2 = ET_EXEC)
        write_datum(&mut f, 0x3e      as u16)?; // 2: e_machine              (0x3e = amd64)
        write_datum(&mut f, 1         as u32)?; // 4: e_version
        write_datum(&mut f, LOAD_BASE as u64)?; // 8: e_entry
        write_datum(&mut f, 0x40      as u64)?; // 8: e_phoff
        write_datum(&mut f, e_shoff   as u64)?; // 8: e_shoff
        write_datum(&mut f, 0         as u32)?; // 4: e_flags
        write_datum(&mut f, 0x40      as u16)?; // 2: e_ehsize
        write_datum(&mut f, 0x38      as u16)?; // 2: e_phentsize
        write_datum(&mut f, 3         as u16)?; // 2: e_phnum
        write_datum(&mut f, 0x40      as u16)?; // 2: e_shentsize
        write_datum(&mut f, 6         as u16)?; // 2: e_shnum
        write_datum(&mut f, 5         as u16)?; // 2: e_shstrndx

        // Write the program header table.
        // Entry 1: Loads our .text section.
        write_pt_load(
            &mut f,
            PF_X | PF_R,
            text_off,
            LOAD_BASE,
            text_len,
            text_len,
            4096,
        )?;
        // Entry 2: Loads our .rodata section.
        write_pt_load(
            &mut f,
            PF_R,
            rodata_off,
            LOAD_BASE + (rodata_off - text_off),
            rodata_len,
            rodata_len,
            4096,
        )?;
        // Entry 3: Loads our .data and .bss sections.
        write_pt_load(
            &mut f,
            PF_R | PF_W,
            data_off,
            LOAD_BASE + (data_off - text_off),
            data_len,
            total_rw_load_size,
            4096,
        )?;

        // Pad to our target code address.
        for _ in (0x40 + 3 * 0x38)..text_off {
            f.write(b"\0")?;
        }
        f.write(&self.instructions)?;

        // Pad to our target rodata address.
        for _ in (text_off + text_len)..rodata_off {
            f.write(b"\0")?;
        }
        f.write(&self.rodata)?;

        // Pad to our target data address.
        for _ in (rodata_off + rodata_len)..data_off {
            f.write(b"\0")?;
        }
        f.write(&self.data)?;

        // Write the .strtab strings.
        for _ in (data_off + data_len)..strtab_off {
            f.write(b"\0")?;
        }
        f.write(strtab.as_bytes())?;

        // Pad to our section header table address.
        for _ in (strtab_off + strtab_len)..e_shoff {
            f.write(b"\0")?;
        }

        // Write the required null section entry.
        write_section_header_table_entry(&mut f, 0, 0, 0, 0, 0, 0, 0)?;
        write_section_header_table_entry(
            &mut f,
            strtab.find(".text").unwrap() as u32,
            SHT_PROGBITS,
            SHF_ALLOC | SHF_EXECINSTR,
            LOAD_BASE,
            text_off,
            text_len,
            4096,
        )?;
        write_section_header_table_entry(
            &mut f,
            strtab.find(".rodata").unwrap() as u32,
            SHT_PROGBITS,
            SHF_ALLOC,
            LOAD_BASE + (rodata_off - text_off),
            rodata_off,
            rodata_len,
            4096,
        )?;
        write_section_header_table_entry(
            &mut f,
            strtab.find(".data").unwrap() as u32,
            SHT_PROGBITS,
            SHF_ALLOC | SHF_WRITE,
            LOAD_BASE + (data_off - text_off),
            data_off,
            data_len,
            4096,
        )?;
        // The concept of "bss_off" doesn't really make sense, because it's not actually data
        // that's present in the ELF file. However, I use strtab_off here because it corresponds
        // to the byte offset that makes the most sense. (See the diagram at the top of this file.)
        let bss_off = strtab_off;
        write_section_header_table_entry(
            &mut f,
            strtab.find(".bss").unwrap() as u32,
            SHT_NOBITS,
            SHF_ALLOC | SHF_WRITE,
            LOAD_BASE + (bss_off - text_off),
            bss_off, // I think this value is arbitrary and doesn't matter?
            self.bss_size,
            16,
        )?;
        write_section_header_table_entry(
            &mut f,
            strtab.find(".strtab").unwrap() as u32,
            SHT_STRTAB,
            SHF_STRINGS,
            0,
            strtab_off,
            strtab_len,
            1,
        )?;

        // Close the file.
        f.flush()?;
        std::mem::drop(f);

        // Ugh, is there no easy way to do this from the existing fd in Rust?
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
        Ok(())
    }
}
