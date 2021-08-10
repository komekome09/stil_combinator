extern crate combine;

use combine::{Parser, many1};
use combine::parser::char::digit;

fn main() {
    let mut parser = many1::<Vec<_>, _, _>(digit());
    println!("{:?}", parser.parse("123"));
    println!("{:?}", parser.parse("123ABC"));
    println!("{:?}", parser.parse("ABC123"));
}
