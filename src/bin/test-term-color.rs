use rile::term::{ErasePart, Term};
use rile::Color;

use clap::{App, Arg};

fn check_system_color(term: &mut Term) {
    for n in 0..255 {
        term.csi("m");
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
                term.csi("m");
                term.rgb_bg(r, g, b);
                term.flush();
                term.write(block);

                term.bg(approx);
                term.write(block);

                term.csi("m");
                term.write("\n");
                term.flush();
            }
            term.write("\n");
        }
        term.write("\n\n");
    }
}

fn main() {
    let matches = App::new("test-term-color")
        .arg(
            Arg::with_name("list-system-colors")
                .long("--list-system-color")
                .help("List 256 system colors"),
        )
        .get_matches();

    let mut term = Term::new();

    if matches.is_present("list-system-colors") {
        check_system_color(&mut term);
    } else {
        check_truecolor(&mut term);
    }

    term.csi("m");
    term.erase_line(ErasePart::ToEnd);
    term.flush();
}
