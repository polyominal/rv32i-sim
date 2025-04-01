//! System call handler

use std::io::Write;
use std::io::{self};

use text_io::scan;

use crate::error::ExecutionError;
use crate::error::SimulatorResult;
use crate::memory::StorageInterface;

/// Handles a system call
pub fn syscall(
    op1: i32,
    op2: i32,
    mem: &mut impl StorageInterface,
) -> SimulatorResult<i32> {
    let call_type = op2;
    let call_arg = op1;

    // Does no change by default
    let mut result: i32 = op1;

    match call_type {
        0 => {
            // Print a string
            let mut address = call_arg as u32;
            loop {
                let ch = mem.get(address, 1, &mut None, &mut None)? as u8;
                if ch == 0 {
                    break;
                }
                print!("{}", ch as char);
                io::stdout().flush()?;
                address += 1;
            }
        }
        1 => {
            // Print a character
            print!("{}", (call_arg as u8) as char);
            io::stdout().flush()?;
        }
        2 => {
            // Print a signed number
            print!("{}", { call_arg });
            io::stdout().flush()?;
        }
        3 => {
            // Exit the program
            // We'll do nothing actually
        }
        4 => {
            // Read a character
            let c: char;
            scan!("{}", c);
            result = c as i32;
        }
        5 => {
            // Read a signed number
            let n: i32;
            scan!("{}", n);
            result = n;
        }
        _ => {
            return Err(ExecutionError::UnknownSystemCall(call_type).into());
        }
    }

    Ok(result)
}
