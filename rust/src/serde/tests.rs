/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use std::{collections::HashMap, fmt::Debug};

use serde::{de::Visitor, Deserialize, Serialize};

use crate::{
    datum_char_to_token_pipeline,
    serde::de::{PlainDeserializer, RootDeserializer},
    DatumAtom, IntoViaDatumPipe,
};

use crate::serde::ser::{RootSerializer, Style};

use super::ser::PlainSerializer;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
struct Substruct {
    a: i32,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
enum VeryDetailedEnum {
    UnitVariant,
    StructVariant { a: i32 },
    NewtypeVariant(i32),
    NewtypeVecVariant(Vec<i32>),
    NewtypeStructVariant(Substruct),
    TupleVariant(i32, i32),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MyExampleStruct {
    test1: String,
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
    NewtypeMap(HashMap<String, String>),
}

fn test_serializes_to<'a, V: Debug + PartialEq + Serialize + Deserialize<'a>>(text: &str, v: &V) {
    let mut out = String::new();
    v.serialize(&mut PlainSerializer::new(&mut out, Style::SpacingOnly))
        .unwrap();
    assert_eq!(&out, text);
    // verify no panics/etc.
    let mut delme = String::new();
    v.serialize(&mut PlainSerializer::new(&mut delme, Style::Indented))
        .unwrap();
    // check these deserialize properly
    test_deserializes_to(&out, v);
    test_deserializes_to(&delme, v);
}
fn test_root_serializes_to<'a, V: Debug + PartialEq + Serialize + Deserialize<'a>>(
    text: &str,
    v: &V,
) {
    let mut out = String::new();
    v.serialize(&mut RootSerializer(PlainSerializer::new(
        &mut out,
        Style::SpacingOnly,
    )))
    .unwrap();
    assert_eq!(&out, text);
    // verify no panics/etc.
    let mut delme = String::new();
    v.serialize(&mut RootSerializer(PlainSerializer::new(
        &mut delme,
        Style::Indented,
    )))
    .unwrap();
    // check these deserialize properly
    test_root_deserializes_to(&out, v);
    test_root_deserializes_to(&delme, v);
}
fn test_nt_serializes_to<'a, V: Debug + PartialEq + Serialize + Deserialize<'a>>(
    text: &str,
    v: &V,
) {
    test_serializes_to(&format!("({})", text), v);
    test_root_serializes_to(text, v);
}
fn test_root_serializes_to_indented<V: Serialize>(text: &str, v: V) {
    let mut out = String::new();
    v.serialize(&mut RootSerializer(PlainSerializer::new(
        &mut out,
        Style::Indented,
    )))
    .unwrap();
    assert_eq!(&out, text);
}

fn test_deserializes_to<'a, V: Deserialize<'a> + Debug + PartialEq>(source: &str, v: &V) {
    let mut it = source
        .chars()
        .via_datum_pipe(datum_char_to_token_pipeline());
    let mut pd = PlainDeserializer::from_iterator(&mut it);
    assert!(pd.has_next_token().unwrap());
    let v2 = V::deserialize(&mut pd).unwrap();
    assert!(!pd.has_next_token().unwrap());
    assert_eq!(v, &v2);
}

fn test_root_deserializes_to<'a, V: Deserialize<'a> + Debug + PartialEq>(source: &str, v: &V) {
    let mut it = source
        .chars()
        .via_datum_pipe(datum_char_to_token_pipeline());
    let mut pd = RootDeserializer(PlainDeserializer::from_iterator(&mut it));
    assert!(pd.0.has_next_token().unwrap());
    let v2 = V::deserialize(&mut pd).unwrap();
    assert!(!pd.0.has_next_token().unwrap());
    assert_eq!(v, &v2);
}

fn test_atom_deserializes_to<'a, V: Deserialize<'a> + Debug + PartialEq>(
    atom: DatumAtom<&str>,
    v: V,
) {
    let mut text = String::new();
    atom.write(&mut text).unwrap();
    test_deserializes_to(&text, &v)
}

fn test_deserialization_fails<'a, V: Deserialize<'a> + Debug + PartialEq>(source: &str, _v: V) {
    let mut it = source
        .chars()
        .via_datum_pipe(datum_char_to_token_pipeline());
    let mut pd = PlainDeserializer::from_iterator(&mut it);
    _ = V::deserialize(&mut pd).unwrap_err();
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct MyExampleNewtypeStructHonest;
impl<'de> Visitor<'de> for MyExampleNewtypeStructHonest {
    type Value = MyExampleNewtypeStructHonest;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("example text")
    }
    fn visit_newtype_struct<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        let _ = bool::deserialize(deserializer).unwrap();
        Ok(MyExampleNewtypeStructHonest)
    }
}
impl<'de> Deserialize<'de> for MyExampleNewtypeStructHonest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_newtype_struct(
            "MyExampleNewtypeStructHonest",
            MyExampleNewtypeStructHonest,
        )
    }
}

#[test]
fn test_deserializing() {
    test_atom_deserializes_to(DatumAtom::String("Hello!"), "Hello!".to_string());
    test_atom_deserializes_to(DatumAtom::Symbol("Hello!"), "Hello!".to_string());
    test_atom_deserializes_to(DatumAtom::Integer(0), 0);
    test_atom_deserializes_to(DatumAtom::Integer(0xFF), 0xFF as u8);
    test_atom_deserializes_to(DatumAtom::Integer(0xFFFF), 0xFFFF as u16);
    test_atom_deserializes_to(DatumAtom::Integer(0xFFFFFFFF), 0xFFFFFFFF as u32);
    // sadly, Serde won't cast this
    // test_atom_deserializes_to(DatumAtom::Integer(-0x8000000000000000), 0xFFFFFFFFFFFFFFFF as u64);
    test_atom_deserializes_to(
        DatumAtom::Integer(-0x8000000000000000),
        -0x8000000000000000 as i64,
    );
    test_atom_deserializes_to(DatumAtom::Float(0.123), 0.123 as f32);
    test_atom_deserializes_to(DatumAtom::Float(0.123), 0.123 as f64);
    test_atom_deserializes_to(DatumAtom::Boolean(false), false);
    test_atom_deserializes_to(DatumAtom::Boolean(true), true);
    test_atom_deserializes_to(DatumAtom::Boolean(true), Some(true));
    test_atom_deserializes_to(DatumAtom::Boolean(false), Some(false));
    test_atom_deserializes_to(DatumAtom::Nil, ());
    test_atom_deserializes_to(DatumAtom::Nil, None as Option<bool>);
    test_atom_deserializes_to(DatumAtom::Boolean(true), MyExampleNewtypeStructHonest);
    test_deserializes_to("(#t #t #f)", &vec![true, true, false]);
    let mut example_map: HashMap<String, String> = HashMap::new();
    example_map.insert("edgar".to_string(), "computer".to_string());
    example_map.insert("marker".to_string(), "cones".to_string());
    test_deserializes_to("(edgar computer marker cones)", &example_map.clone());
    test_deserializes_to(
        "(test1 test1val)",
        &MyExampleStruct {
            test1: "test1val".to_string(),
        },
    );
    test_deserializes_to("()", &MyExampleUnitStruct);
    test_deserializes_to("UnitVariant", &VeryDetailedEnum::UnitVariant);
    test_deserializes_to("(UnitVariant)", &VeryDetailedEnum::UnitVariant);
    test_deserializes_to("(NewtypeVariant 0)", &VeryDetailedEnum::NewtypeVariant(0));
    test_root_deserializes_to("NewtypeVariant 0", &VeryDetailedEnum::NewtypeVariant(0));
    test_deserializes_to(
        "(NewtypeVecVariant 0 1 2)",
        &VeryDetailedEnum::NewtypeVecVariant(vec![0, 1, 2]),
    );
    test_root_deserializes_to(
        "NewtypeVecVariant 0 1 2",
        &VeryDetailedEnum::NewtypeVecVariant(vec![0, 1, 2]),
    );
    test_deserializes_to(
        "(NewtypeStructVariant a 0)",
        &VeryDetailedEnum::NewtypeStructVariant(Substruct { a: 0 }),
    );
    test_root_deserializes_to(
        "NewtypeStructVariant a 0",
        &VeryDetailedEnum::NewtypeStructVariant(Substruct { a: 0 }),
    );
    test_deserializes_to("(TupleVariant 0 1)", &VeryDetailedEnum::TupleVariant(0, 1));
    test_root_deserializes_to("TupleVariant 0 1", &VeryDetailedEnum::TupleVariant(0, 1));
    test_deserializes_to(
        "(StructVariant a 2)",
        &VeryDetailedEnum::StructVariant { a: 2 },
    );
    test_root_deserializes_to(
        "StructVariant a 2",
        &VeryDetailedEnum::StructVariant { a: 2 },
    );

    // these deserializations are known to be impossible, on purpose
    test_deserialization_fails("", false);
    test_deserialization_fails(")", false);
    test_deserialization_fails("1", true);
    test_deserialization_fails("(", vec![true]);
    test_deserialization_fails("1", VeryDetailedEnum::UnitVariant);
    test_deserialization_fails("", VeryDetailedEnum::UnitVariant);
    test_deserialization_fails("", None as Option<String>);
    test_deserialization_fails("", ());
    test_deserialization_fails("", example_map.clone());
    test_deserialization_fails("1", example_map.clone());
    // fails during atomization
    test_deserialization_fails("#", true);
    // ok, so this is a really tricky case.
    // basically, Serde will early-abort once it's "done" with the tuple.
    // so you have to have something like, say, expect_list_end after visitor.visit_seq
    // and that requires a specific error handler for if a list end does not appear
    // presumably, the idea was:
    // * self-describing formats can keep track of that they aren't "supposed to have" ended yet in some way (i.e. a 'is it okay to end now' flag in the Deserializer, controlled by the SeqAccess which has the actual state)
    // * in Postcard-like formats, the schema either matches (in which case this behaviour is fine) or it doesn't (in which case it's "your fault")
    test_deserialization_fails("(#t #t #t)", (true, true));
    // This should trigger the hold-and-any path; moreover it should not panic
    test_deserialization_fails("\"\"", 0 as u64);
}

#[test]
fn test_serializing() {
    // primitives
    test_serializes_to(
        "(0 0 0 0 0 0 0 0)",
        &(
            0 as u8, 0 as u16, 0 as u32, 0 as u64, 0 as i8, 0 as i16, 0 as i32, 0 as i64,
        ),
    );
    test_serializes_to("(0.0 0.0)", &(0.0f64, 0.0f32));
    test_serializes_to("\"9\"", &'9');
    // continue...
    test_serializes_to("(#t #t #f)", &vec![true, true, false]);
    let mut example_map: HashMap<String, String> = HashMap::new();
    example_map.insert("marker".to_string(), "cones".to_string());
    // ordering can be weird b/c hashmap, so
    test_serializes_to("(\"marker\" \"cones\")", &example_map.clone());
    test_serializes_to(
        "(test1 \"test1val\")",
        &MyExampleStruct {
            test1: "test1val".to_string(),
        },
    );
    test_serializes_to("()", &MyExampleUnitStruct);
    test_serializes_to("(1 2 3)", &(1, 2, 3));
    test_serializes_to("UnitVariant", &VeryDetailedEnum::UnitVariant);
    test_serializes_to("(NewtypeVariant 0)", &VeryDetailedEnum::NewtypeVariant(0));
    test_serializes_to("(TupleVariant 0 1)", &VeryDetailedEnum::TupleVariant(0, 1));
    test_serializes_to(
        "(StructVariant a 2)",
        &VeryDetailedEnum::StructVariant { a: 2 },
    );

    test_nt_serializes_to("NewtypeUnit ()", &Doc::NewtypeUnit(()));
    test_nt_serializes_to("NewtypeU64 -1", &Doc::NewtypeU64(0xFFFFFFFFFFFFFFFF));
    // None at root level is invalid
    test_serializes_to("(NewtypeOption #nil)", &Doc::NewtypeOption(None));
    test_nt_serializes_to("NewtypeOption ()", &Doc::NewtypeOption(Some(())));
    test_nt_serializes_to(
        "NewtypeEnum UnitVariant",
        &Doc::NewtypeEnum(VeryDetailedEnum::UnitVariant),
    );
    test_nt_serializes_to(
        "NewtypeEnum StructVariant a 0",
        &Doc::NewtypeEnum(VeryDetailedEnum::StructVariant { a: 0 }),
    );
    test_nt_serializes_to(
        "NewtypeEnum NewtypeVariant 0",
        &Doc::NewtypeEnum(VeryDetailedEnum::NewtypeVariant(0)),
    );
    test_nt_serializes_to(
        "NewtypeEnum TupleVariant 0 0",
        &Doc::NewtypeEnum(VeryDetailedEnum::TupleVariant(0, 0)),
    );
    test_nt_serializes_to("NewtypeVec 1 2 3", &Doc::NewtypeVec(vec![1, 2, 3]));
    test_nt_serializes_to("NewtypeTuple 1 2", &Doc::NewtypeTuple((1, 2)));
    test_nt_serializes_to(
        "NewtypeTupleStruct 1 2",
        &Doc::NewtypeTupleStruct(MyExampleTupleStruct(1, 2)),
    );
    test_nt_serializes_to(
        "NewtypeStruct test1 \"\"",
        &Doc::NewtypeStruct(MyExampleStruct {
            test1: "".to_string(),
        }),
    );
    test_nt_serializes_to(
        "NewtypeMap \"marker\" \"cones\"",
        &Doc::NewtypeMap(example_map.clone()),
    );

    test_root_serializes_to("1 2 3", &(1, 2, 3));
    test_root_serializes_to_indented(
        "CheckStructIndent\n0\n(\n\ttest1 \"\"\n)\n(3 4)\n",
        Doc::CheckStructIndent(
            0,
            MyExampleStruct {
                test1: "".to_string(),
            },
            MyExampleTupleStruct(3, 4),
        ),
    );
}
