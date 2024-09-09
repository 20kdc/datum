/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

//! Tests! Sent into a separate directory so they can be filtered from cloc results.

extern crate std;

use core::hash::Hash;
use std::convert::TryFrom;
use std::hash::{DefaultHasher, Hasher};

use crate::{
    DatumByteToTokenPipeline, DatumChar, DatumCharClass, DatumDecoder, DatumParser,
    DatumUTF8Decoder, IntoViaDatumBufPipe,
};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::{
    datum_byte_to_value_pipeline, datum_char_to_token_pipeline, datum_char_to_value_pipeline,
    DatumAtom, DatumPipe, DatumToken, DatumWriter, IntoViaDatumPipe,
};

fn do_roundtrip_test(input: &str, output: &str) {
    let mut dectok1 = datum_char_to_token_pipeline();
    let mut tokenization = Vec::new();
    dectok1
        .feed_iter_to_vec(&mut tokenization, input.chars(), true)
        .unwrap();
    // ---
    let mut dtparse = datum_char_to_value_pipeline();
    let mut out = Vec::new();
    let mut line_number: usize = 1;
    let dtres = dtparse.feed_iter_to_vec(
        &mut out,
        input.chars().inspect(|v| {
            if *v == '\n' {
                line_number += 1
            }
        }),
        true,
    );
    dtres.unwrap_or_else(|_| panic!("problem at line {}, dump {:?}", line_number, tokenization));
    // so, fun fact, in all the refactors, a bug snuck in where starting any list would enable the parse error flag
    let mut out_str = String::new();
    let mut writer = DatumWriter::default();
    let mut hasher = DefaultHasher::default();
    for v in out {
        v.hash(&mut hasher);
        v.write_to(&mut out_str, &mut writer).unwrap();
        writer.write_newline(&mut out_str).unwrap();
    }
    let hash1 = hasher.finish();
    assert_eq!(out_str, output);
    // --- same again but with bytes
    let mut dtparse = datum_byte_to_value_pipeline();
    let mut out = Vec::new();
    dtparse
        .feed_iter_to_vec(&mut out, input.bytes(), true)
        .unwrap();
    let mut out_str = String::new();
    let mut writer = DatumWriter::default();
    let mut hasher = DefaultHasher::default();
    for v in &out {
        v.hash(&mut hasher);
        v.write_to(&mut out_str, &mut writer).unwrap();
        writer.write_newline(&mut out_str).unwrap();
    }
    let hash2 = hasher.finish();
    assert_eq!(out_str, output);
    assert_eq!(hash1, hash2);
    // --- iterator test ---
    assert_eq!(
        out_str
            .bytes()
            .via_datum_pipe(datum_byte_to_value_pipeline())
            .map(|v| v.unwrap())
            .count(),
        out.len()
    );
    // --- iterator test 2 ---
    let i2pipeline: DatumByteToTokenPipeline<String> = Default::default();
    let i2pipeline = i2pipeline.compose(DatumParser::default());
    // must compile
    _ = out_str.bytes().via_datum_buf_pipe(i2pipeline.clone());
    assert_eq!(
        out_str
            .bytes()
            .via_datum_buf_pipe(i2pipeline)
            .map(|v| v.unwrap())
            .count(),
        out.len()
    );
}

fn tokenizer_should_error_eof(input: &str) {
    let mut dectok1 = datum_char_to_token_pipeline();
    let mut ignoredout = Vec::new();
    dectok1
        .feed_iter_to_vec(&mut ignoredout, input.chars(), false)
        .unwrap();
    assert!(dectok1.feed(0, None, &mut |_, _| { Ok(()) }).is_err());
}

fn parser_should_error(input: &str) {
    let mut dtparse = datum_char_to_value_pipeline();
    let mut out = Vec::new();
    assert!(dtparse
        .feed_iter_to_vec(&mut out, input.chars(), true)
        .is_err());
}

#[test]
fn test_token_write() {
    assert_eq!(
        &DatumToken::String(0, "Test\\\r\n\t").to_string(),
        "\"Test\\\\\\r\\n\\t\""
    );
    assert_eq!(&DatumToken::Symbol(0, "").to_string(), "#{}#");
    assert_eq!(&DatumToken::Symbol(0, "test").to_string(), "test");
    assert_eq!(&DatumToken::SpecialID(0, "test").to_string(), "#test");
    let flt: DatumToken<&'static str> = DatumToken::Float(0, -1.0);
    assert_eq!(&flt.to_string(), "-1.0");
    // make sure above wasn't implemented via rounding or something
    let flt: DatumToken<&'static str> = DatumToken::Float(0, -1.07915331907856);
    assert_eq!(&flt.to_string(), "-1.07915331907856");
    let int: DatumToken<&'static str> = DatumToken::Integer(0, 1);
    assert_eq!(&int.to_string(), "1");
    let ls: DatumToken<&'static str> = DatumToken::ListStart(0);
    let le: DatumToken<&'static str> = DatumToken::ListEnd(0);
    assert_eq!(&ls.to_string(), "(");
    assert_eq!(&le.to_string(), ")");
}

#[test]
fn roundtrip_tests() {
    let niltest: DatumAtom<&str> = DatumAtom::Nil;
    assert_eq!(DatumAtom::default(), niltest);

    let roundtrip_in_file = include_str!("../../../doc/roundtrip-input.scm");
    let roundtrip_out_file = include_str!("../../../doc/roundtrip-output.scm");
    do_roundtrip_test(roundtrip_in_file, roundtrip_out_file);

    // EOF and such tests
    do_roundtrip_test("", "");
    do_roundtrip_test("-10", "-10\n");
    do_roundtrip_test("-", "-\n");
    do_roundtrip_test("\\-a", "\\-a\n");
    do_roundtrip_test("#t", "#t\n");
    do_roundtrip_test("; line comment\n", "");
    do_roundtrip_test("\n", "");

    // Hexints
    do_roundtrip_test("#x100", "256\n");
    parser_should_error("#xZ");

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
    writer_fmt_test
        .write_comment(&mut writer_fmt, "lof\nlif\nidk?")
        .unwrap();
    assert_eq!(writer_fmt, "\n\t; lof\n\t; lif\n\t; idk?\n");
}

fn decoder_test(input: &str, output: &str, out_class: DatumCharClass) {
    let mut decoder = DatumDecoder::default();
    let mut output_iterator = output.chars();
    for v in input.chars() {
        decoder
            .feed(0, Some(v), &mut |_, c| {
                assert_eq!(c.char(), output_iterator.next().expect("early output end"));
                assert_eq!(c.class(), out_class);
                Ok(())
            })
            .unwrap();
    }
    assert_eq!(decoder, DatumDecoder::default());
    decoder.feed(0, None, &mut |_, _| Ok(())).unwrap();
    assert_eq!(output_iterator.next(), None);
}

/// Tests the decoder for a single character.
/// This verifies all sorts of useful properties, especially that the starting offset is always returned.
/// This is important for no-alloc no-fixed-buffer parsing.
fn decoder_test_char(input: &str, output: &str, out_class: DatumCharClass) {
    let mut decoder = DatumDecoder::default();
    let mut output_iterator = output.chars();
    for (k, v) in input.chars().enumerate() {
        decoder
            .feed((100 + k) as u64, Some(v), &mut |at, c| {
                assert_eq!(at, 100);
                assert_eq!(c.char(), output_iterator.next().expect("early output end"));
                assert_eq!(c.class(), out_class);
                Ok(())
            })
            .unwrap();
    }
    assert_eq!(decoder, DatumDecoder::default());
    decoder.feed(0, None, &mut |_, _| Ok(())).unwrap();
    assert_eq!(output_iterator.next(), None);
}

fn decoder_should_fail(input: &str) {
    let mut decoder = DatumDecoder::default();
    for v in input.chars() {
        let res = decoder.feed(0, Some(v), &mut |_, _| Ok(()));
        if res.is_err() {
            return;
        }
    }
    panic!("Decoder was supposed to fail!!! tc: {}", input);
}

fn decoder_should_not_allow_eof(input: &str) {
    let mut decoder = DatumDecoder::default();
    for v in input.chars() {
        decoder.feed(0, Some(v), &mut |_, _| Ok(())).unwrap();
    }
    assert!(decoder.feed(0, None, &mut |_, _| { Ok(()) }).is_err());
}

#[test]
fn decoder_results_test() {
    let mut decoder = DatumDecoder::default();
    decoder
        .feed(0, Some('\\'), &mut |_, _| panic!("NO"))
        .unwrap();
    decoder
        .feed(0, Some('x'), &mut |_, _| panic!("NO"))
        .unwrap();
    decoder
        .feed(0, Some('1'), &mut |_, _| panic!("NO"))
        .unwrap();
    decoder
        .feed(0, Some('0'), &mut |_, _| panic!("NO"))
        .unwrap();
    decoder
        .feed(0, Some('F'), &mut |_, _| panic!("NO"))
        .unwrap();
    decoder
        .feed(0, Some('F'), &mut |_, _| panic!("NO"))
        .unwrap();
    decoder
        .feed(0, Some('F'), &mut |_, _| panic!("NO"))
        .unwrap();
    decoder
        .feed(0, Some('F'), &mut |_, _| panic!("NO"))
        .unwrap();
    let out = [DatumChar::content('\u{10FFFF}'), DatumChar::content('a')];
    let mut tmp = Vec::new();
    decoder
        .feed_iter_to_vec(&mut tmp, [';', 'a'], true)
        .unwrap();
    assert_eq!(tmp, out);
}

#[test]
fn all_decoder_test_cases() {
    // -- also see byte_decoder.rs:byte_decoder_tests
    decoder_test(
        "thequickbrownfoxjumpsoverthelazydog",
        "thequickbrownfoxjumpsoverthelazydog",
        DatumCharClass::Content,
    );
    decoder_test(
        "THEQUICKBROWNFOXJUMPSOVERTHELAZYDOG",
        "THEQUICKBROWNFOXJUMPSOVERTHELAZYDOG",
        DatumCharClass::Content,
    );
    decoder_test(
        "!£$%^&*_+=[]{}~@:?/>.<,|",
        "!£$%^&*_+=[]{}~@:?/>.<,|",
        DatumCharClass::Content,
    );
    // a few simple sanity checks
    decoder_test_char("\\n", "\n", DatumCharClass::Content);
    decoder_test_char("\\r", "\r", DatumCharClass::Content);
    decoder_test_char("\\t", "\t", DatumCharClass::Content);
    decoder_test_char("\n", "\n", DatumCharClass::Newline);
    decoder_test_char(";", ";", DatumCharClass::LineComment);
    decoder_test_char("\"", "\"", DatumCharClass::String);
    decoder_test_char("(", "(", DatumCharClass::ListStart);
    decoder_test_char(")", ")", DatumCharClass::ListEnd);
    decoder_test_char("#", "#", DatumCharClass::SpecialID);
    decoder_test_char("\\;", ";", DatumCharClass::Content);
    // Hex escape check
    decoder_test_char("\\x0A;", "\n", DatumCharClass::Content);
    // UTF-8 encoding check
    decoder_test_char("\\xB9;", "¹", DatumCharClass::Content);
    decoder_test_char("\\x10FFff;", "\u{10FFFF}", DatumCharClass::Content);
    decoder_test_char("\u{10FFFF}", "\u{10FFFF}", DatumCharClass::Content);
    // --

    // failure tests
    decoder_should_fail("\\\n");
    decoder_should_fail("\\x-");
    decoder_should_fail("\\xFFFFFF;");
    decoder_should_not_allow_eof("\\");
    decoder_should_not_allow_eof("\\x");
    decoder_should_not_allow_eof("\\xA");
}

fn byte_decoder_should_fail(input: &[u8]) {
    let mut decoder = DatumUTF8Decoder::default();
    for v in input {
        if decoder.feed(0, Some(*v), &mut |_, _| Ok(())).is_err() {
            return;
        }
    }
    panic!("Decoder was supposed to fail!!!");
}

fn byte_decoder_should_not_allow_eof(input: &[u8]) {
    let mut decoder = DatumUTF8Decoder::default();
    for v in input {
        decoder.feed(0, Some(*v), &mut |_, _| Ok(())).unwrap();
    }
    assert!(decoder.feed(0, None, &mut |_, _| { Ok(()) }).is_err());
}

#[test]
fn byte_decoder_tests() {
    // failure tests
    // random continuation
    byte_decoder_should_fail(&[0x80]);
    // start of sequence, but nothing else
    byte_decoder_should_not_allow_eof(&[0xC2]);
    // it just keeps going and going!
    byte_decoder_should_fail(&[0xC2, 0x80, 0x80, 0x80, 0x80]);
    // interrupted 'characters'
    byte_decoder_should_fail(&[0xC2, 0xC2]);
}

#[test]
fn these_cannot_be_atomized() {
    let tkn: DatumToken<&str> = DatumToken::ListStart(0);
    DatumAtom::try_from(tkn).unwrap_err();
    let tkn: DatumToken<&str> = DatumToken::ListEnd(0);
    DatumAtom::try_from(tkn).unwrap_err();
}
