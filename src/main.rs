// metanito — inspect and strip EXIF metadata from images.
//
// Everything happens locally. Nothing is uploaded anywhere.
//
// Usage:
//   metanito inspect <image>          show the metadata in a file
//   metanito strip <image> [output]   write a copy with EXIF removed
//
// v1 scope: inspect works on any container kamadak-exif can read
// (JPEG, PNG, TIFF, WebP, HEIF...). strip handles JPEG only for now.

use std::env;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process;

use img_parts::{ImageEXIF, ImageICC};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        print_usage(&args[0]);
        process::exit(2);
    }

    let command = args[1].as_str();
    let path = Path::new(&args[2]);

    let result = match command {
        "inspect" => inspect(path),
        "strip" => {
            let output = args
                .get(3)
                .map(PathBuf::from)
                .unwrap_or_else(|| default_output(path));
            strip(path, &output)
        }
        other => {
            eprintln!("Unknown command: {other}\n");
            print_usage(&args[0]);
            process::exit(2);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

fn print_usage(prog: &str) {
    eprintln!("metanito - inspect and strip EXIF metadata (local-only)\n");
    eprintln!("Usage:");
    eprintln!("  {prog} inspect <image>");
    eprintln!("  {prog} strip <image> [output]");
}

/// Read and print every EXIF field in the file.
fn inspect(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(path)?;
    let mut bufreader = BufReader::new(&file);

    match exif::Reader::new().read_from_container(&mut bufreader) {
        Ok(exif) => {
            let mut count = 0;
            for field in exif.fields() {
                println!(
                    "{:<30} {}",
                    field.tag.to_string(),
                    field.display_value().with_unit(&exif)
                );
                count += 1;
            }
            if count == 0 {
                println!("No EXIF fields found.");
            } else {
                println!("\n{count} EXIF field(s) found.");
            }
        }
        // No EXIF block at all — not an error, just nothing to show.
        Err(exif::Error::NotFound(_)) => {
            println!("No EXIF metadata found in this file.");
        }
        Err(e) => return Err(Box::new(e)),
    }
    Ok(())
}

/// Write a copy of the image with EXIF (and the ICC profile) removed.
fn strip(path: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let input = fs::read(path)?;

    match detect_format(&input) {
        Format::Jpeg => {
            let mut jpeg = img_parts::jpeg::Jpeg::from_bytes(input.into())?;
            jpeg.set_exif(None); // drop the EXIF segment (this is where GPS lives)
            jpeg.set_icc_profile(None); // drop the colour profile too, for good measure

            let mut out = Vec::new();
            jpeg.encoder().write_to(&mut out)?;
            fs::write(output, &out)?;

            println!("Wrote cleaned image to {}", output.display());
            verify(output)?;
            Ok(())
        }
        Format::Png => Err(
            "Detected a PNG. strip handles JPEG only in v1 — \
             PNG/WebP support is the next increment. (inspect works on it.)"
                .into(),
        ),
        Format::Unsupported => Err(
            "Unsupported format. This tool strips JPEG for now."
                .into(),
        ),
    }
}

/// Re-read the output and confirm no EXIF fields survived.
fn verify(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(path)?;
    let mut bufreader = BufReader::new(&file);

    match exif::Reader::new().read_from_container(&mut bufreader) {
        Ok(exif) => {
            let n = exif.fields().count();
            if n == 0 {
                println!("Verified: no EXIF fields remain.");
            } else {
                // Shouldn't happen after a successful strip — flag loudly if it does.
                println!("WARNING: {n} EXIF field(s) still present after strip!");
            }
        }
        Err(exif::Error::NotFound(_)) => {
            println!("Verified: no EXIF metadata remains.");
        }
        Err(e) => return Err(Box::new(e)),
    }
    Ok(())
}

enum Format {
    Jpeg,
    Png,
    Unsupported,
}

/// Identify the format by its magic bytes rather than trusting the extension.
fn detect_format(data: &[u8]) -> Format {
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Format::Jpeg
    } else if data.starts_with(&[0x89, b'P', b'N', b'G']) {
        Format::Png
    } else {
        Format::Unsupported
    }
}

/// Build a default output path: photo.jpg -> photo_clean.jpg
fn default_output(path: &Path) -> PathBuf {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("jpg");
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!("{stem}_clean.{ext}"))
}
