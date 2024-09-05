/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use std::{collections::HashMap, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::serde::{de_tests::{test_deserializes_to, test_root_deserializes_to}, ser::{RootSerializer, Style}};

use super::ser::PlainSerializer;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum VeryDetailedEnum {
    UnitVariant,
    StructVariant {
        a: i32
    },
    NewtypeVariant(i32),
    TupleVariant(i32, i32)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MyExampleStruct {
    test1: String
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MyExampleUnitStruct;
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MyExampleTupleStruct(i32, i32);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Doc {
    NewtypeUnit(()),
    NewtypeU64(u64),
    NewtypeOption(Option<()>),
    NewtypeEnum(VeryDetailedEnum),
    NewtypeVec(Vec<i32>),
    NewtypeTuple((i32, i32)),
    CheckStructIndent(i32, MyExampleStruct, MyExampleTupleStruct),
    NewtypeTupleStruct(MyExampleTupleStruct),
    NewtypeStruct(MyExampleStruct),
    NewtypeMap(HashMap<String, String>)
}

fn test_serializes_to<'a, V: Debug + PartialEq + Serialize + Deserialize<'a>>(text: &str, v: &V) {
    let mut out = String::new();
    v.serialize(&mut PlainSerializer::new(&mut out, Style::SpacingOnly)).unwrap();
    assert_eq!(&out, text);
    // verify no panics/etc.
    let mut delme = String::new();
    v.serialize(&mut PlainSerializer::new(&mut delme, Style::Indented)).unwrap();
    // check these deserialize properly
    test_deserializes_to(&out, v);
    test_deserializes_to(&delme, v);
}
fn test_root_serializes_to<'a, V: Debug + PartialEq + Serialize + Deserialize<'a>>(text: &str, v: &V) {
    let mut out = String::new();
    v.serialize(&mut RootSerializer(PlainSerializer::new(&mut out, Style::SpacingOnly))).unwrap();
    assert_eq!(&out, text);
    // verify no panics/etc.
    let mut delme = String::new();
    v.serialize(&mut RootSerializer(PlainSerializer::new(&mut delme, Style::Indented))).unwrap();
    // check these deserialize properly
    test_root_deserializes_to(&out, v);
    test_root_deserializes_to(&delme, v);
}
fn test_nt_serializes_to<'a, V: Debug + PartialEq + Serialize + Deserialize<'a>>(text: &str, v: &V) {
    test_serializes_to(&format!("({})", text), v);
    test_root_serializes_to(text, v);
}
fn test_root_serializes_to_indented<V: Serialize>(text: &str, v: V) {
    let mut out = String::new();
    v.serialize(&mut RootSerializer(PlainSerializer::new(&mut out, Style::Indented))).unwrap();
    assert_eq!(&out, text);
}

#[test]
fn test_serializing() {
    // primitives
    test_serializes_to("(0 0 0 0 0 0 0 0)", &(0 as u8, 0 as u16, 0 as u32, 0 as u64, 0 as i8, 0 as i16, 0 as i32, 0 as i64));
    test_serializes_to("(0.0 0.0)", &(0.0f64, 0.0f32));
    test_serializes_to("\"9\"", &'9');
    // continue...
    test_serializes_to("(#t #t #f)", &vec![true, true, false]);
    let mut example_map: HashMap<String, String> = HashMap::new();
    example_map.insert("marker".to_string(), "cones".to_string());
    // ordering can be weird b/c hashmap, so
    test_serializes_to("(\"marker\" \"cones\")", &example_map.clone());
    test_serializes_to("(test1 \"test1val\")", &MyExampleStruct {
        test1: "test1val".to_string()
    });
    test_serializes_to("()", &MyExampleUnitStruct);
    test_serializes_to("(1 2 3)", &(1, 2, 3));
    test_serializes_to("UnitVariant", &VeryDetailedEnum::UnitVariant);
    test_serializes_to("(NewtypeVariant 0)", &VeryDetailedEnum::NewtypeVariant(0));
    test_serializes_to("(TupleVariant 0 1)", &VeryDetailedEnum::TupleVariant(0, 1));
    test_serializes_to("(StructVariant a 2)", &VeryDetailedEnum::StructVariant { a: 2 });

    test_nt_serializes_to("NewtypeUnit ()", &Doc::NewtypeUnit(()));
    test_nt_serializes_to("NewtypeU64 -1", &Doc::NewtypeU64(0xFFFFFFFFFFFFFFFF));
    // None at root level is invalid
    test_serializes_to("(NewtypeOption #nil)", &Doc::NewtypeOption(None));
    test_nt_serializes_to("NewtypeOption ()", &Doc::NewtypeOption(Some(())));
    test_nt_serializes_to("NewtypeEnum UnitVariant", &Doc::NewtypeEnum(VeryDetailedEnum::UnitVariant));
    test_nt_serializes_to("NewtypeEnum StructVariant a 0", &Doc::NewtypeEnum(VeryDetailedEnum::StructVariant { a: 0 }));
    test_nt_serializes_to("NewtypeEnum NewtypeVariant 0", &Doc::NewtypeEnum(VeryDetailedEnum::NewtypeVariant(0)));
    test_nt_serializes_to("NewtypeEnum TupleVariant 0 0", &Doc::NewtypeEnum(VeryDetailedEnum::TupleVariant(0, 0)));
    test_nt_serializes_to("NewtypeVec 1 2 3", &Doc::NewtypeVec(vec![1, 2, 3]));
    test_nt_serializes_to("NewtypeTuple 1 2", &Doc::NewtypeTuple((1, 2)));
    test_nt_serializes_to("NewtypeTupleStruct 1 2", &Doc::NewtypeTupleStruct(MyExampleTupleStruct(1, 2)));
    test_nt_serializes_to("NewtypeStruct test1 \"\"", &Doc::NewtypeStruct(MyExampleStruct { test1: "".to_string() }));
    test_nt_serializes_to("NewtypeMap \"marker\" \"cones\"", &Doc::NewtypeMap(example_map.clone()));

    test_root_serializes_to("1 2 3", &(1, 2, 3));
    test_root_serializes_to_indented("CheckStructIndent\n0\n(\n\ttest1 \"\"\n)\n(3 4)\n", Doc::CheckStructIndent(0, MyExampleStruct { test1: "".to_string() }, MyExampleTupleStruct(3, 4)));
}
