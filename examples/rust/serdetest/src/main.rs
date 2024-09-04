/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

// This project is for testing the possibility of Serde support in Datum.

use std::{collections::HashMap, env::args, fs::read_to_string};

use datum::{DatumCharToTokenPipeline, IntoViaDatumPipe};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
enum MyEnum {
    Apple,
    Berry
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
enum MyComplexEnum {
    Stuff,
    ThingCount(i32)
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct MyTupleStruct(i32, i32);

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct MyUnitStruct;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct MyExampleType {
    pub wobble: i32,
    pub myenum: MyEnum,
    pub tuple: (i32, i32),
    pub tuple_struct: MyTupleStruct,
    pub unit_struct: MyUnitStruct,
    pub myvec: Vec<MyComplexEnum>
}

type MyExampleDocument = HashMap<String, MyExampleType>;

fn main() {
    let mut args = args();
    _ = args.next();
    let filename = args.next().expect("filename should be given");
    assert_eq!(args.next(), None);
    let contents = read_to_string(filename).expect("filename should be readable");
    // this is slowly being worked on to be less and less alloc-using, but I think we've hit a brick wall
    let custom: DatumCharToTokenPipeline<String> = DatumCharToTokenPipeline::default();
    let mut iterator = contents.chars().via_datum_pipe(custom);
    let mut tmp = datum::de::MapRootDeserializer(datum::de::PlainDeserializer::from_iterator(&mut iterator));
    let vec: MyExampleDocument = MyExampleDocument::deserialize(&mut tmp).unwrap();
    for v in vec {
        println!("{:?}", v);
    }
}
