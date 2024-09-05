/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use std::collections::HashMap;

use serde::Serialize;

use crate::serde::ser::{RootSerializer, Style};

use super::ser::PlainSerializer;

#[derive(Serialize)]
enum VeryDetailedEnum {
    UnitVariant,
    StructVariant {
        a: i32
    },
    NewtypeVariant(i32),
    TupleVariant(i32, i32)
}

#[derive(Serialize)]
struct MyExampleStruct {
    test1: String
}
#[derive(Serialize)]
struct MyExampleUnitStruct;

#[derive(Serialize)]
enum MyExampleEnumDocument {
    NewtypeVec(Vec<i32>)
}

fn test_serializes_to<V: Serialize>(text: &str, v: V) {
    let mut out = String::new();
    v.serialize(&mut PlainSerializer::new(&mut out, Style::SpacingOnly)).unwrap();
    assert_eq!(&out, text);
}
fn test_root_serializes_to<V: Serialize>(text: &str, v: V) {
    let mut out = String::new();
    v.serialize(&mut RootSerializer(PlainSerializer::new(&mut out, Style::SpacingOnly))).unwrap();
    assert_eq!(&out, text);
}

#[test]
fn test_serializing() {
    test_serializes_to("(#t #t #f)", vec![true, true, false]);
    let mut example_map: HashMap<String, String> = HashMap::new();
    example_map.insert("marker".to_string(), "cones".to_string());
    // ordering can be weird b/c hashmap, so
    test_serializes_to("(\"marker\" \"cones\")", example_map.clone());
    test_serializes_to("(test1 \"test1val\")", MyExampleStruct {
        test1: "test1val".to_string()
    });
    test_serializes_to("()", MyExampleUnitStruct);
    test_serializes_to("UnitVariant", VeryDetailedEnum::UnitVariant);
    test_serializes_to("(NewtypeVariant 0)", VeryDetailedEnum::NewtypeVariant(0));
    test_serializes_to("(TupleVariant 0 1)", VeryDetailedEnum::TupleVariant(0, 1));
    test_serializes_to("(StructVariant a 2)", VeryDetailedEnum::StructVariant { a: 2 });
    test_root_serializes_to("NewtypeVec 1 2 3", MyExampleEnumDocument::NewtypeVec(vec![1, 2, 3]));
}
