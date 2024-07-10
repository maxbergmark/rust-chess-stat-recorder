#![warn(
    // missing_docs,
    // unreachable_pub,
    keyword_idents,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    future_incompatible,
    nonstandard_style,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
)]

mod enums;
mod error;
mod game;
mod game_data;
mod game_player_data;
mod parser;
mod plotting;
mod ui;
mod util;
mod validator;

pub use error::{Error, Result};

#[tokio::main]
async fn main() -> Result<()> {
    parser::run_all_files().await
}
