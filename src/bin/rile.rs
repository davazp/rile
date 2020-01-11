//! rile is a simple editor written in Rust.
//!

extern crate signal_hook;

use rile::buffer::Buffer;
use rile::context::Context;
use rile::event_loop::event_loop;
use rile::term::{with_raw_mode, Term};
use rile::window::refresh_screen;

use clap::{App, Arg};

const PKG_NAME: &'static str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const PKG_AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
const PKG_DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
const PKG_GIT_COMMIT: Option<&'static str> = option_env!("GIT_COMMIT");

/// The main entry point of the editor.
fn main() {
    let matches = App::new(PKG_NAME)
        .version(
            format!(
                "{} (git: {})",
                PKG_VERSION,
                PKG_GIT_COMMIT.map(|c| &c[0..8]).unwrap_or("unknown")
            )
            .as_ref(),
        )
        .author(PKG_AUTHORS)
        .about(PKG_DESCRIPTION)
        .arg(Arg::with_name("FILE").help("Input file").index(1))
        .get_matches();

    let file_arg = matches.value_of("FILE");

    let mut context = Context::new(if let Some(filename) = file_arg {
        Buffer::from_file(filename)
    } else {
        Buffer::from_string("")
    });

    signal_hook::flag::register(signal_hook::SIGWINCH, context.was_resized.clone()).unwrap();

    let term = &mut Term::new();
    let context = &mut context;

    term.enable_alternative_screen_buffer();

    refresh_screen(term, context);

    with_raw_mode(|| while !event_loop(term, context, |_, _| {}).is_ok() {})
        .expect("Could not initialize the terminal to run in raw mode.");

    term.disable_alternative_screen_buffer();
    term.show_cursor();
    term.flush();
}
