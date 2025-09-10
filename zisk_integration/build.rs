use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // Create a custom linker script for ZisK
    let linker_script = r#"
MEMORY
{
    RAM : ORIGIN = 0xa0000000, LENGTH = 0x20000000
    ROM : ORIGIN = 0x80000000, LENGTH = 0x20000000
}

SECTIONS
{
    .text : {
        *(.text .text.*)
    } > ROM
    
    .rodata : {
        *(.rodata .rodata.*)
    } > ROM
    
    .data : {
        *(.data .data.*)
    } > RAM
    
    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    } > RAM
    
    /DISCARD/ : {
        *(.comment)
        *(.note*)
    }
}
"#;
    
    fs::write(out_dir.join("linker.ld"), linker_script).unwrap();
    
    println!("cargo:rustc-link-arg=-T{}", out_dir.join("linker.ld").display());
}
