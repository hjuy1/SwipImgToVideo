#![allow(unused)]
use std::{collections::HashMap, fmt::Debug};

pub const RESOURCE: &str = "../resources";

pub fn debug_print<T: Debug>(s: T) {
    #[cfg(debug_assertions)]
    dbg!(s);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        println!("{:?}", std::fs::canonicalize(RESOURCE));
    }
}
