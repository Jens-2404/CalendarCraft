//! Kommandozeile. Stand Phase 0: `cc-cli convert JJJJ-MM-TT`.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [cmd, date] if cmd == "convert" => convert(date),
        _ => {
            eprintln!("Aufruf: cc-cli convert JJJJ-MM-TT");
            ExitCode::from(2)
        }
    }
}

fn convert(input: &str) -> ExitCode {
    let parts: Vec<&str> = input.split('-').collect();
    let parsed = match parts.as_slice() {
        [y, m, d] => y.parse().ok().zip(m.parse().ok()).zip(d.parse().ok()),
        _ => None,
    };
    let Some(((y, m), d)) = parsed else {
        eprintln!("Datum nicht lesbar: {input} (erwartet JJJJ-MM-TT)");
        return ExitCode::from(2);
    };
    match cc_calendar::gregorian_to_hebrew(y, m, d) {
        Ok(h) => {
            println!("{input} = {}. {} {}", h.day, h.month_code, h.year);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
