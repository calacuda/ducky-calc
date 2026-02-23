#![no_std]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

// use colored::Color;
// use fern::colors::{Color, ColoredLevelConfig};

pub mod ast;
pub mod coms_proto;

// #[cfg(test)]
// mod tests {
//     // use std::sync::mpsc::channel;
//
//     use super::*;
//     use ast::DuckyScript;
//     use pest::Parser;
//
//     fn parse(source: &str, length: usize) {
//         let script = DuckyScript::get_len(source);
//
//         assert!(script.is_ok(), "parsing script resulted in an error");
//         assert_eq!(script.unwrap(), length, "parsed the wrong number of tokens")
//     }
//
//     #[test]
//     fn generic_parser() {
//         parse("LED_R\nDELAY 2000\nGUI r\nSTRINGLN notepad\nDELAY 5000\nLED_G\nSTRING H\nSHIFT ello, World!\nENTER\nREM foobar\nLED_OFF\nSHIFT UP", 12);
//         parse("LED_R\nDELAY 2000\nGUI r\nSTRINGLN notepad\nDELAY 5000\nLED_G\nSTRING H\nSHIFT ello, World!\nENTER\nREM foobar\nLED_OFF\nSHIFT UP\n", 12);
//         parse("\nLED_R\nDELAY 2000\nGUI r\nSTRINGLN notepad\nDELAY 5000\nLED_G\nSTRING H\nSHIFT ello, World!\nENTER\nREM foobar\nLED_OFF\nSHIFT UP", 12);
//         parse("\nLED_R\nDELAY 2000\nGUI r\nSTRINGLN notepad\nDELAY 5000\nLED_G\nSTRING H\nSHIFT ello, World!\nENTER\nREM foobar\nLED_OFF\nSHIFT UP\n", 12);
//         parse("INJECT_MOD GUI", 1);
//
//         // assert!(false);
//     }
//
//     #[test]
//     fn blocks_and_multiline() {
//         parse("\nSTRINGLN\n\tline1\nEND_STRINGLN", 1);
//         parse("\nSTRINGLN\nline1\nEND_STRINGLN\n", 1);
//         parse("\nREM_BLOCK\nline1\nline2\nEND_REM\n", 1);
//         parse("\nREM_BLOCK\nline1\nline2\nEND_REM\nREM foobar", 2);
//         // parse("\nGUI\n");
//
//         // assert!(false);
//     }
//
//     // #[test]
//     // fn output_test() {
//     //     // let (tx, _rx) = channel();
//     //     let res =
//     //         DuckyScript::new("/dev/not-a-tty").from_source("\nSTRINGLN\n\tline1\nEND_STRINGLN");
//     //     // parse("\nSTRINGLN\nline1\nEND_STRINGLN\n", 1);
//     //     assert!(res.is_ok(), "parsing script resulted in an error: {res:?}");
//     //
//     //     // assert!(false);
//     // }
// }
