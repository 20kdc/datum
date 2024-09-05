/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use std::{collections::HashMap, fmt::Debug};

use serde::{de::Visitor, Deserialize};

use crate::{
    datum_char_to_token_pipeline, serde::de::{PlainDeserializer, RootDeserializer}, DatumAtom, IntoViaDatumPipe,
};

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
struct Substruct {
    a: i32
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
enum VeryDetailedEnum {
    UnitVariant,
    StructVariant {
        a: i32
    },
    NewtypeVariant(i32),
    NewtypeVecVariant(Vec<i32>),
    NewtypeStructVariant(Substruct),
    TupleVariant(i32, i32)
}

pub(crate) fn test_deserializes_to<'a, V: Deserialize<'a> + Debug + PartialEq>(
    source: &str,
    v: &V,
) {
    let mut it = source.chars().via_datum_pipe(datum_char_to_token_pipeline());
    let mut pd = PlainDeserializer::from_iterator(&mut it);
    assert!(pd.has_next_token().unwrap());
    let v2 = V::deserialize(&mut pd).unwrap();
    assert!(!pd.has_next_token().unwrap());
    assert_eq!(v, &v2);
}

pub(crate) fn test_root_deserializes_to<'a, V: Deserialize<'a> + Debug + PartialEq>(
    source: &str,
    v: &V,
) {
    let mut it = source.chars().via_datum_pipe(datum_char_to_token_pipeline());
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

fn test_deserialization_fails<'a, V: Deserialize<'a> + Debug + PartialEq>(
    source: &str,
    _v: V,
) {
    let mut it = source.chars().via_datum_pipe(datum_char_to_token_pipeline());
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
    fn visit_newtype_struct<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        let _ = bool::deserialize(deserializer).unwrap();
        Ok(MyExampleNewtypeStructHonest)
    }
}
impl<'de> Deserialize<'de> for MyExampleNewtypeStructHonest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_newtype_struct("MyExampleNewtypeStructHonest", MyExampleNewtypeStructHonest)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct MyExampleStruct;

impl<'de> Visitor<'de> for MyExampleStruct {
    type Value = MyExampleStruct;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("example text")
    }
    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let k: String = map.next_key().unwrap().unwrap();
        assert_eq!(k, "test1");
        let v: String = map.next_value().unwrap();
        assert_eq!(v, "test1val");
        let kopt: Option<String> = map.next_key().unwrap();
        assert_eq!(kopt, None);
        Ok(MyExampleStruct)
    }
}
impl<'de> Deserialize<'de> for MyExampleStruct {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_struct("MyExampleStruct", &["test1"], MyExampleStruct)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct MyExampleUnitStruct;

impl<'de> Visitor<'de> for MyExampleUnitStruct {
    type Value = MyExampleUnitStruct;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("example text")
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
        Ok(MyExampleUnitStruct)
    }
}
impl<'de> Deserialize<'de> for MyExampleUnitStruct {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_unit_struct("MyExampleUnitStruct", MyExampleUnitStruct)
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
    test_atom_deserializes_to(DatumAtom::Integer(-0x8000000000000000), -0x8000000000000000 as i64);
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
    test_deserializes_to("(test1 test1val)", &MyExampleStruct);
    test_deserializes_to("()", &MyExampleUnitStruct);
    test_deserializes_to("UnitVariant", &VeryDetailedEnum::UnitVariant);
    test_deserializes_to("(UnitVariant)", &VeryDetailedEnum::UnitVariant);
    test_deserializes_to("(NewtypeVariant 0)", &VeryDetailedEnum::NewtypeVariant(0));
    test_root_deserializes_to("NewtypeVariant 0", &VeryDetailedEnum::NewtypeVariant(0));
    test_deserializes_to("(NewtypeVecVariant 0 1 2)", &VeryDetailedEnum::NewtypeVecVariant(vec![0, 1, 2]));
    test_root_deserializes_to("NewtypeVecVariant 0 1 2", &VeryDetailedEnum::NewtypeVecVariant(vec![0, 1, 2]));
    test_deserializes_to("(NewtypeStructVariant a 0)", &VeryDetailedEnum::NewtypeStructVariant(Substruct { a: 0 }));
    test_root_deserializes_to("NewtypeStructVariant a 0", &VeryDetailedEnum::NewtypeStructVariant(Substruct { a: 0 }));
    test_deserializes_to("(TupleVariant 0 1)", &VeryDetailedEnum::TupleVariant(0, 1));
    test_root_deserializes_to("TupleVariant 0 1", &VeryDetailedEnum::TupleVariant(0, 1));
    test_deserializes_to("(StructVariant a 2)", &VeryDetailedEnum::StructVariant { a: 2 });
    test_root_deserializes_to("StructVariant a 2", &VeryDetailedEnum::StructVariant { a: 2 });

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
