/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;

use crate::{datum_byte_to_value_pipeline, datum_char_to_token_pipeline, datum_char_to_value_pipeline, DatumAtom, DatumPipe, DatumToken, DatumWriter, ViaDatumPipe};

fn do_roundtrip_test(input: &str, output: &str) {
    let mut dectok1 = datum_char_to_token_pipeline();
    let mut tokenization = Vec::new();
    dectok1.feed_iter_to_vec(&mut tokenization, input.chars(), true).unwrap();
    // ---
    let mut dtparse = datum_char_to_value_pipeline();
    let mut out = Vec::new();
    let mut line_number: usize = 1;
    let dtres = dtparse.feed_iter_to_vec(&mut out, input.chars().inspect(|v| if *v == '\n' { line_number += 1 }), true);
    dtres.expect(&format!("problem at line {}, dump {:?}", line_number, tokenization));
    // so, fun fact, in all the refactors, a bug snuck in where starting any list would enable the parse error flag
    let mut out_str = String::new();
    let mut writer = DatumWriter::default();
    for v in out {
        v.write_to(&mut out_str, &mut writer).unwrap();
        writer.write_newline(&mut out_str).unwrap();
    }
    assert_eq!(out_str, output);
    // --- same again but with bytes
    let mut dtparse = datum_byte_to_value_pipeline();
    let mut out = Vec::new();
    dtparse.feed_iter_to_vec(&mut out, input.bytes(), true).unwrap();
    let mut out_str = String::new();
    let mut writer = DatumWriter::default();
    for v in &out {
        v.write_to(&mut out_str, &mut writer).unwrap();
        writer.write_newline(&mut out_str).unwrap();
    }
    assert_eq!(out_str, output);
    // --- one final time, with feeling: iterator test ---
    assert_eq!(out_str.bytes().via_datum_pipe(datum_byte_to_value_pipeline()).map(|v| v.unwrap()).count(), out.len());
}

fn tokenizer_should_error_eof(input: &str) {
    let mut dectok1 = datum_char_to_token_pipeline();
    let mut ignoredout = Vec::new();
    dectok1.feed_iter_to_vec(&mut ignoredout, input.chars(), false).unwrap();
    assert!(dectok1.eof(&mut |_| {Ok(())}).is_err());
}

fn parser_should_error(input: &str) {
    let mut dtparse = datum_char_to_value_pipeline();
    let mut out = Vec::new();
    assert!(dtparse.feed_iter_to_vec(&mut out, input.chars(), true).is_err());
}


#[test]
fn test_token_write() {
    assert_eq!(&DatumToken::String("Test\\\r\n\t").to_string(), "\"Test\\\\\\r\\n\\t\"");
    assert_eq!(&DatumToken::ID("").to_string(), "#{}#");
    assert_eq!(&DatumToken::ID("test").to_string(), "test");
    assert_eq!(&DatumToken::SpecialID("test").to_string(), "#test");
    let flt: DatumToken<&'static str> = DatumToken::Float(-1.0);
    assert_eq!(&flt.to_string(), "-1.0");
    // make sure above wasn't implemented via rounding or something
    let flt: DatumToken<&'static str> = DatumToken::Float(-1.07915331907856);
    assert_eq!(&flt.to_string(), "-1.07915331907856");
    let int: DatumToken<&'static str> = DatumToken::Integer(1);
    assert_eq!(&int.to_string(), "1");
    let ls: DatumToken<&'static str> = DatumToken::ListStart;
    let le: DatumToken<&'static str> = DatumToken::ListEnd;
    assert_eq!(&ls.to_string(), "(");
    assert_eq!(&le.to_string(), ")");
}

#[test]
fn roundtrip_tests() {
    let niltest: DatumAtom<&str> = DatumAtom::Nil;
    assert_eq!(DatumAtom::default(), niltest);

    let roundtrip_in_file = include_str!("../../doc/roundtrip-input.scm");
    let roundtrip_out_file = include_str!("../../doc/roundtrip-output.scm");
    do_roundtrip_test(roundtrip_in_file, roundtrip_out_file);

    // EOF and such tests
    do_roundtrip_test("", "");
    do_roundtrip_test("-10", "-10\n");
    do_roundtrip_test("-", "-\n");
    do_roundtrip_test("\\-a", "\\-a\n");
    do_roundtrip_test("#t", "#t\n");
    do_roundtrip_test("; line comment\n", "");
    do_roundtrip_test("\n", "");

    tokenizer_should_error_eof("\"a");

    parser_should_error(")");
    parser_should_error("(");
    parser_should_error("#atom_conv_failure");
    parser_should_error("#i0_numeric_conv_failure");

    // writer silly tests
    let mut writer_fmt = String::new();
    let mut writer_fmt_test = DatumWriter::default();
    writer_fmt_test.write_newline(&mut writer_fmt).unwrap();
    writer_fmt_test.indent = 1;
    writer_fmt_test.write_comment(&mut writer_fmt, "lof\nlif\nidk?").unwrap();
    assert_eq!(writer_fmt, "\n\t; lof\n\t; lif\n\t; idk?\n");
}
