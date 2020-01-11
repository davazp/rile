//! test-term
//!
//! You can use this binary as a way to debug the support for some
//! specific terminal.
//!

use clap::{App, AppSettings, Arg, SubCommand};

use rile::term::{read_key_timeout, with_raw_mode, ErasePart, Term};
use rile::Color;
use rile::Key;

fn check_system_color(term: &mut Term) {
    for n in 0..255 {
        term.reset_attr();
        term.bg(n);
        term.write(&format!("{} {}", n, Color::name_from_code(n)));
        term.erase_line(ErasePart::ToEnd);
        term.write("\n");
        term.flush()
    }
}

fn check_truecolor(term: &mut Term) {
    for r in (0..255).step_by(10) {
        for g in (0..255).step_by(10) {
            for b in (0..255).step_by(10) {
                let approx = Color::from_rgb(r, g, b).to_256_code();

                let block = "                    ";
                term.reset_attr();
                term.rgb_bg(r, g, b);
                term.flush();
                term.write(block);

                term.bg(approx);
                term.write(block);

                term.reset_attr();
                term.write("\n");
                term.flush();
            }
            term.write("\n");
        }
        term.write("\n\n");
    }
}

fn check_input() {
    println!("Reading and printing keys. Press 'q' to exit.\n");

    let _ = with_raw_mode(|| loop {
        if let Some(key) = read_key_timeout() {
            print!("{} ({})\r\n", key, key.to_code());

            if key == Key::parse("q").unwrap() {
                break;
            }
        }
    });
}

fn main() {
    let matches = App::new("test-term")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(SubCommand::with_name("input"))
        .subcommand(
            SubCommand::with_name("color").arg(
                Arg::with_name("list-system-colors")
                    .long("--list-system-color")
                    .help("List 256 system colors"),
            ),
        )
        .get_matches();

    let mut term = Term::new();

    match matches.subcommand() {
        ("input", _) => check_input(),
        ("color", Some(submatches)) => {
            if submatches.is_present("list-system-colors") {
                check_system_color(&mut term);
            } else {
                check_truecolor(&mut term);
            }
            term.reset_attr();
            term.erase_line(ErasePart::ToEnd);
            term.flush();
        }
        _ => unreachable!(),
    }
}
