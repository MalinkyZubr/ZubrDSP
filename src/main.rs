mod foo;
use crate::foo::{MyStruct, Another};

fn main() {
    println!("Hello, world!");
    let _ms = MyStruct{};
    let _a: Another = Another{};
}
