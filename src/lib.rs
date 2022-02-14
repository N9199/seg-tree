//! [![github]](https://github.com/N9199/seg-tree)
//! 
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! 
//! <br>
//! 
//! This library provides simple and easy to use segment trees and some variations of them, by simply implementing certain traits. It also gives some already implemented nodes types, which serve can also serve as examples.
//! 
//! <br>


#![warn(missing_docs)]
/// Module which provides already implemented nodes.
pub mod default;
/// Module which provides every node trait.
pub mod nodes;
/// Module which provides segment tree implementation
pub mod segment_tree;