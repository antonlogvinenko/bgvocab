#![feature(decl_macro)]
#[macro_use]
extern crate rocket;
//use rocket::*;

use std::collections::BTreeMap;

use rocket::{response::content::Json};
use vocab_lib::{get_ru_vocabulary, VocabWord};

#[get("/hello?<page>&<size>")]
fn hello(page: Option<usize>, size: Option<usize>) -> Json<String> {
    let mut p: usize = page.unwrap_or(1);
    let mut s: usize = size.unwrap_or(10);

    let vocab = get_ru_vocabulary().unwrap();
    let batch_vocab: BTreeMap<VocabWord, Vec<String>> = vocab
        .into_iter()
        .skip(p * s)
        .take(s)
        .collect();

    // println!(">>>> {:?}", batch_vocab);
    Json(format!("{:?}", ""))

//         "{
//     'status': 'success',
//     'message': 'Hello API!'
//   }",
//     )
}

fn main() {
    rocket::ignite().mount("/api", routes![hello]).launch();
}
