// Copyright 2024 Cornell University
// released under BSD 3-Clause License
// author: Kevin Laeufer <laeufer@cornell.edu>

use std::path::Path;
use svlang_sys::*;

fn from_verilog(filename: &str) {
    let tree = syntax::SyntaxTree_fromFile(filename);
    println!("{}", syntax::SyntaxNode_toString(tree.root()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        from_verilog("inputs/coward/datapath_verification_fig1_impl.v");
    }
}
