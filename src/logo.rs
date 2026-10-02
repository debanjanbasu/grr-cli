//! The `--version` banner: the grr crab, rendered as well as the
//! terminal allows.
//!
//! Two renderings of one logo. On a terminal that advertises 24-bit
//! colour, the mascot is drawn as 60x22 cells of U+2580 UPPER HALF BLOCK,
//! each cell carrying the top pixel in the foreground and the bottom
//! pixel in the background — two pixels of vertical detail per character
//! row, which is the most a character cell can express. Everywhere else
//! (pipes, CI, `TERM=dumb`, `NO_COLOR`) the same mascot degrades to the
//! ASCII crab.
//!
//! Both renderings put the bare semver on the FIRST line. clap prefixes
//! it with `grr `, and three consumers depend on that line alone:
//! scripts/benchmark.ts parses it as the measured version and the
//! workflow's version gate compares it against Cargo.toml;
//! scripts/generate-demo.ts matches it against
//! `/^grr \d+\.\d+\.\d+$/m`; the Homebrew formula greps for the version as
//! a substring anywhere in the output. A crab-first layout would parse as
//! "version missing" and fail the gate open forever.
//!
//! The pixel map below is the mascot's idle pose, cropped to its
//! bounding box. site/scripts/generate-mascot.mjs holds the 64x64
//! original (`buildIdleMap`) and the sRGB values in [`rgb`] mirror its
//! PALETTE_LEGEND, so the CLI and the site speak one palette.

use std::borrow::Cow;
use std::fmt::Write as _;
use std::io::IsTerminal;

/// The mascot's idle pose, 43 rows of 60 pixels, cropped to the crab's
/// bounding box. `.` is transparent; every other cell indexes [`rgb`].
const CRAB: [&str; 43] = [
    "...................oooooooooo..oooooooooo...................",
    "...................owwwwwwhho..owwwwwwhho...................",
    "...................owwwwwwhho..owwwwwwhho...................",
    "...................owwwwwwhho..owwwwwwhho...................",
    "...................ohhhhkkhho..ohhhhkkhho...................",
    "...................ohhhhkkhho..ohhhhkkhho...................",
    "...................ohhkkkkhho..ohhkkkkhho...................",
    "...................ohhkkkkhho..ohhkkkkhho...................",
    "...................ohhkkkdhho..ohhkkkdhho...................",
    "...................ophhhhhhhoooophhhhhhho...................",
    ".................whohhhhhhhhoppohhhhhhhho...................",
    "...oooo.........wphooooooooooppoooooooooo............oooo...",
    "..o.mmmoooo....wwphhhpppppppwppppppppppmooo......oooommmoo..",
    ".o.phhhpmmo...pwwwhhhpppppppwppppppppppmmmoo.....omphhhpmoo.",
    "o.mpppppmmo..ppwwwwwwwwwwwwwwpppppwwwwwbmmmoo....ompppppmmoo",
    "oompppppmmo..ppwwwwwwwwwwwwwwpppppwwwwwbbmmmoo...ompppppmm.o",
    "o.mmmmmmmbo..pppwwwwwwwwwwwwpppppppppbwwbbmmmoo..ombmmmmmmoo",
    "oommmmmmmmo..ppppwwwwwwwwwwppppppppppbwwbbbmmmo..ommmmmmmmo.",
    ".ommmmmmmmoooppppppppppppppppppppppppbwwbbbmmmoo.ommmmmmmmo.",
    ".o.dmmmmmo.ooppppppppddppppppppppppppbwwmmbmmmoo.o.mmmmmmdo.",
    "..o.omm.ooobobppppppppppmmppppppppppbbbbbbdddddoooo.omm.oo..",
    "...ooo.o..obobbppppppppppppppppppppbbddbbbdddddoopoooo.o....",
    ".....oo...ooobbbbppppppppppppppppbbbbbbbbbdddddodoo..oo.....",
    "...........oobbbbbrrrrpppppppppbbbbbbbrrrrdddddooo..........",
    "............obbbbbpprrbbbbkbbbbbkbbbbbrrdddddddo............",
    "............ombbbbbbbbbbbbbkkkkkbbbbbbbbbbdddddo............",
    "............oombbbbbbbbbbbbbbbbbbbbbbbbbbmddddoo............",
    "............oommbbbbbbbbbbbbbbbbbbbbbbbbmmddddoo............",
    ".............ommmbbbbbbbbbbbbbbbbbbbbbbmmmddddo.............",
    ".............oommmmbbbbbbbbbbbbbbbbbbmmmmmdddoo.............",
    "..............oommmmmbbbbbbbbbbbbbbmmmmmmmddoo..............",
    "...............oommmmmmmmmmmmmmmmmmmmmmmmmdoo...............",
    "................oommmmmmmmmmmmmmmmmmmmmmmmoo................",
    ".................ooommmmmmmmmdddddddddddooooo...............",
    "................oobooommmmmmmdddddddddooooodo...............",
    "...............ooboo.oooommmmddddddoooooo.odoo..............",
    "..............ooboo..oooooooooooooooodoo..oodoo.............",
    "..............oboo....oopoo........oboo....oopoo............",
    ".............oobo......oopo.......oobo......oopo............",
    "............ooboo.......opoo.....ooboo.......opoo...........",
    "........owwwwooo........ooowwowwwwooo........ooowwwwo.......",
    "........obbbboo..........ooppobbbboo..........ooppppo.......",
    "........oooooo............ooooooooo............oooooo.......",
];

/// The tagline: the crate's `about` line, repeated verbatim so the two
/// can never drift.
const TAGLINE: &str = "Google tools from the terminal, at maximum performance";

/// The plain-text banner: semver, a blank line, the ASCII crab, then the
/// tagline — concatenated at compile time, so a non-colour terminal pays
/// no formatting cost at all.
const ASCII_BANNER: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\n\n",
    // The mascot's idle pose: wide-set glossy eyes, small smile, chunky
    // claws, drawn in four rows of ASCII.
    "      _~^~^~^~_\n",
    "  \\) / (o) (o) \\ (/\n",
    "    '_   \\_/   _'\n",
    "    \\  '-----'  /\n",
    "Google tools from the terminal, at maximum performance",
);

/// The sRGB value of one palette cell, or `None` for `.` (transparent).
///
/// Keys and values are PALETTE_LEGEND from
/// site/scripts/generate-mascot.mjs; every non-`.` cell in [`CRAB`] has
/// an arm, which is what the tests pin down.
fn rgb(cell: char) -> Option<[u8; 3]> {
    Some(match cell {
        'o' => [0x3B, 0x1D, 0x0B],
        's' => [0x7C, 0x2D, 0x12],
        'd' => [0xC2, 0x41, 0x0C],
        'm' => [0xEA, 0x58, 0x0C],
        'b' => [0xF9, 0x73, 0x16],
        'p' => [0xFF, 0x8A, 0x3D],
        'w' => [0xFF, 0xD9, 0xB0],
        'h' => [0xFF, 0xF3, 0xE4],
        'k' => [0x1A, 0x12, 0x0B],
        'r' => [0xEA, 0x43, 0x35],
        _ => return None,
    })
}

/// The whole `--version` banner, in the richest form this terminal
/// supports.
///
/// Returns [`Cow::Borrowed`] — the compile-time string, zero allocation
/// and zero formatting work — for every invocation whose stdout is not a
/// 24-bit-colour terminal. Only an interactive colour terminal pays to
/// render the logo, so piping, CI and the startup benchmark never do.
pub fn version_banner() -> Cow<'static, str> {
    if !graphics_enabled() {
        return Cow::Borrowed(ASCII_BANNER);
    }

    let mut banner = String::with_capacity(ASCII_BANNER.len() + CRAB.len() * 64);
    banner.push_str(env!("CARGO_PKG_VERSION"));
    banner.push_str("\n\n");
    render_truecolor(&mut banner);
    // The logo is a tall block; a blank line keeps the tagline off its
    // last row instead of butting against it.
    banner.push('\n');
    banner.push_str(TAGLINE);
    Cow::Owned(banner)
}

/// Whether this invocation may draw the colour logo.
///
/// `NO_COLOR` wins over everything, including the `CLICOLOR_FORCE`
/// opt-in: the variable's entire contract is "never colour my output",
/// and an environment that sets both has said so twice.
///
/// `CLICOLOR_FORCE` (anything but `0`) is the conventional way to ask for
/// colour through a pipe — `grr --version | less -R` — and it does not
/// disturb the semver line, which stays plain either way. Without it the
/// logo is only drawn for a real terminal, so the ASCII crab is what
/// scripts, CI and Homebrew see.
fn graphics_enabled() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if std::env::var("CLICOLOR_FORCE").is_ok_and(|force| force != "0") {
        return true;
    }
    std::io::stdout().is_terminal() && truecolor_signalled(terminal_signals())
}

/// What the environment says about the terminal's capabilities, as
/// (TERM, COLORTERM, running under Windows Terminal).
fn terminal_signals() -> (Option<String>, Option<String>, bool) {
    let term = std::env::var("TERM").ok().filter(|value| !value.is_empty());
    let colorterm = std::env::var("COLORTERM").ok().filter(|v| !v.is_empty());
    (term, colorterm, windows_terminal())
}

/// Whether those signals amount to "24-bit colour", the plain decision
/// split out from the environment so it can be tested directly.
///
/// `COLORTERM` is the portable answer and the only one iTerm2, VTE
/// terminals, Kitty, WezTerm and recent Windows Terminal all set. `TERM`
/// carries it too on the terminals that predate `COLORTERM`; `direct`
/// is foot and Ghostty, `truecolor` is Kitty's own spelling. Windows
/// Terminal additionally gets a vote from `WT_SESSION`, which it always
/// sets — and which matters because a ConPTY host interprets ANSI without
/// the console mode bit that legacy conhost needs.
fn truecolor_signalled(signals: (Option<String>, Option<String>, bool)) -> bool {
    let (term, colorterm, windows_terminal) = signals;
    // `dumb` is the terminal explicitly saying it draws nothing — the
    // one TERM value that overrides every other signal, including a
    // COLORTERM the user's shell inherited from somewhere else.
    if term.as_deref() == Some("dumb") {
        return false;
    }
    if matches!(colorterm.as_deref(), Some("truecolor" | "24bit")) {
        return true;
    }
    // Foot and Ghostty advertise direct addressing in TERM itself rather
    // than setting COLORTERM.
    if let Some(term) = term.as_deref()
        && (term.contains("direct") || term.contains("truecolor"))
    {
        return true;
    }
    if cfg!(windows) {
        return windows_terminal;
    }
    false
}

/// Whether the process is hosted by Windows Terminal, where ANSI escape
/// sequences are interpreted by the pseudoconsole without a console mode
/// opt-in.
#[cfg(windows)]
fn windows_terminal() -> bool {
    std::env::var_os("WT_SESSION").is_some()
}

/// Not Windows Terminal, and not Windows at all: no console-mode opt-in
/// is available, so only the portable signals in [`truecolor_signalled`]
/// apply.
#[cfg(not(windows))]
fn windows_terminal() -> bool {
    false
}

/// Draw [`CRAB`] into `out` as half-block cells with 24-bit colour.
///
/// Each cell shows two source rows: the foreground paints the upper
/// pixel, the background the lower one, so a character row carries two
/// rows of the mascot. SGR sequences are emitted only when the colour
/// actually changes from the previous cell — a pixel-art row is mostly
/// runs, and the run-length pass keeps the banner from being a
/// kilobyte of nearly repeated escapes.
fn render_truecolor(out: &mut String) {
    let mut foreground: Option<[u8; 3]> = None;
    let mut background: Option<[u8; 3]> = None;

    for pair in CRAB.chunks(2) {
        let top = pair[0].as_bytes();
        let bottom = pair.get(1).map(|row| row.as_bytes());
        for column in 0..CRAB[0].len() {
            let top_pixel = rgb(top[column] as char);
            let bottom_pixel = bottom.and_then(|row| rgb(row[column] as char));

            if foreground != top_pixel {
                foreground = top_pixel;
                set_colour(out, 38, top_pixel);
            }
            if background != bottom_pixel {
                background = bottom_pixel;
                set_colour(out, 48, bottom_pixel);
            }
            // A transparent pixel in both halves has nothing to paint,
            // and a space costs no escape sequence.
            out.push(if top_pixel.is_none() && bottom_pixel.is_none() {
                ' '
            } else {
                UPPER_HALF_BLOCK
            });
        }
        // Reset per row so the newline itself is never coloured.
        out.push_str(RESET);
        out.push('\n');
    }
}

/// Emit one SGR colour change: 24-bit foreground (`layer` 38) or
/// background (`layer` 48), or the default-ink reset when the pixel is
/// transparent. Escapes are written as `\u{1b}` rather than a literal
/// control byte so the source stays pure ASCII on every platform.
fn set_colour(out: &mut String, layer: u8, pixel: Option<[u8; 3]>) {
    match pixel {
        Some([red, green, blue]) => {
            let _ = write!(out, "\u{1b}[{layer};2;{red};{green};{blue}m");
        }
        None => {
            let _ = write!(out, "\u{1b}[{}m", layer + 1);
        }
    }
}

/// U+2580 UPPER HALF BLOCK: foreground on the top half of the cell,
/// background on the bottom half.
const UPPER_HALF_BLOCK: char = '\u{2580}';

/// SGR 0 — back to the terminal's own default ink. Emitted at the end of
/// every logo row so the logo cannot bleed into the tagline.
const RESET: &str = "\u{1b}[0m";

#[cfg(test)]
mod tests {
    use super::*;

    /// Every cell the crab map actually uses must resolve to a colour,
    /// or the logo silently loses pixels.
    #[test]
    fn every_pixel_cell_maps_to_a_colour() {
        for (row_index, row) in CRAB.iter().enumerate() {
            for (column_index, cell) in row.chars().enumerate() {
                if cell != '.' && rgb(cell).is_none() {
                    panic!("CRAB[{row_index}][{column_index}] = {cell:?} has no palette arm");
                }
            }
        }
    }

    /// The map is a fixed-size grid: half-block rendering indexes it
    /// column-wise and pairs rows, so a ragged map would panic or skew.
    #[test]
    fn the_map_is_a_rectangular_grid() {
        // Half-block rendering pairs rows and indexes every row by column,
        // so a ragged map would either panic or skew the logo.
        assert!(!CRAB.is_empty());
        let width = CRAB[0].len();
        for row in CRAB {
            assert_eq!(row.len(), width, "ragged crab row: {row:?}");
        }
        // An odd row count is expected — the crab's last pixel row is
        // paired with nothing — but the renderer must still emit it.
        assert_eq!(CRAB.len().div_ceil(2), 22);
    }

    /// The logo is a tall block of half blocks ending in a reset, one
    /// line per pair of source rows.
    #[test]
    fn the_colour_render_draws_every_row_as_a_reset_terminated_line() {
        let mut rendered = String::new();
        render_truecolor(&mut rendered);

        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines.len(), CRAB.len().div_ceil(2));
        // The crab's widest row is full-width; a narrower one would mean
        // the transparent margins were trimmed by accident.
        let widest = lines[CRAB.len() / 2 - 1];
        assert!(widest.contains(UPPER_HALF_BLOCK));
        for line in lines {
            assert!(line.ends_with(RESET), "row not reset: {line:?}");
            assert!(line.contains(UPPER_HALF_BLOCK));
        }
    }

    /// Colour is decided by signals alone, so each way of saying
    /// "24-bit" is pinned independently of the machine running the test.
    #[test]
    fn truecolor_is_detected_from_the_documented_signals() {
        let absent = (None, None, false);
        assert!(!truecolor_signalled(absent));
        assert!(truecolor_signalled((None, Some("truecolor".into()), false)));
        assert!(truecolor_signalled((None, Some("24bit".into()), false)));
        // Foot and Ghostty carry it in TERM; Kitty, iTerm2 and the VTE
        // terminals carry it in COLORTERM.
        assert!(truecolor_signalled((
            Some("foot-direct".into()),
            None,
            false
        )));
        assert!(!truecolor_signalled((
            Some("xterm-kitty".into()),
            None,
            false
        )));
        assert!(truecolor_signalled((
            Some("xterm-kitty".into()),
            Some("truecolor".into()),
            false
        )));
        // An empty COLORTERM is no signal at all, and TERM=dumb overrides
        // every other signal — it is the terminal saying it draws nothing.
        assert!(!truecolor_signalled((None, Some(String::new()), false)));
        assert!(!truecolor_signalled((
            Some("dumb".into()),
            Some("truecolor".into()),
            false
        )));
    }

    /// A terminal with no truecolor signal renders the ASCII crab: the
    /// semver, the four-row mascot, the tagline.
    #[test]
    fn the_plain_banner_keeps_semver_first_and_the_crab_intact() {
        let banner = version_banner();
        let first_line = banner.lines().next().expect("banner has a first line");
        assert_eq!(first_line, env!("CARGO_PKG_VERSION"));
        assert!(banner.contains("(o) (o)"));
        assert!(banner.contains("\\_/"));
        assert!(banner.ends_with(TAGLINE));
    }
}
