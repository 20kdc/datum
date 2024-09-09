/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

// This project is for testing the possibility of Serde support in Datum.

use std::{cell::Cell, collections::HashMap, env::args, fs::read_to_string};

use datum::{serde::ser::{PlainSerializer, Style}, DatumCharToTokenPipeline, DatumLineNumberTracker, DatumPipe, IntoViaDatumBufPipe};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
enum MyEnum {
    Apple,
    Berry
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(dead_code)]
enum MyComplexEnum {
    Stuff,
    ThingCount(i32),
    TupleVariant(i32, i32),
    StructVariant {
        a: i32
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(dead_code)]
struct MyTupleStruct(i32, i32);

#[derive(Deserialize, Serialize, Debug)]
#[allow(dead_code)]
struct MyUnitStruct;

#[derive(Deserialize, Serialize, Debug)]
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
    let pipeline: DatumCharToTokenPipeline<String> = DatumCharToTokenPipeline::default();
    let line_number = Cell::new(1);
    let pipeline = DatumLineNumberTracker::new(&line_number).compose(pipeline);
    let mut iterator = contents.chars().via_datum_buf_pipe(pipeline);
    let mut tmp = datum::serde::de::RootDeserializer(datum::serde::de::PlainDeserializer::from_iterator(&mut iterator));
    let des = MyExampleDocument::deserialize(&mut tmp);
    if let Err(err) = des {
        println!("At line {}: {:?}", line_number.get(), err);
    } else if let Ok(des) = des {
        for v in &des {
            println!("{:?}", v);
        }
        println!("converting back...");
        let mut text = String::new();
        let mut ser = PlainSerializer::new(&mut text, Style::Indented);
        des.serialize(&mut ser).unwrap();
        println!("{}", text);
    }
}
