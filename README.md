# metanito

A small, fast CLI to **inspect and strip EXIF metadata** from images.

**Everything runs locally. Nothing is uploaded anywhere.** 

## Why this exists

Photos from phones and cameras carry embedded metadata (EXIF): often the
**GPS coordinates** where the shot was taken, the exact timestamp, and the
device make/model. When you share the *original file* by email, chat as a
file attachment, a personal site, a repo , that metadata rides along.

Note the honest part of the threat model: most large social platforms
(Instagram, Facebook, X) already strip EXIF on upload. The real exposure is
sharing the original file directly. That is what this tool is for.

## Usage

```sh
# see what metadata is in a file
metanito inspect photo.jpg

# write a cleaned copy (defaults to photo_clean.jpg)
metanito strip photo.jpg

# or choose the output name
metanito strip photo.jpg safe.jpg
```

After stripping, the tool re-reads the output and verifies no EXIF fields
remain — a security tool should prove its own claim.

## Build

```sh
cargo build --release
# binary at target/release/metanito
```

## Scope (v1)

- `inspect` — works on any container the reader supports (JPEG, PNG, TIFF,
  WebP, HEIF, ...).
- `strip` — **JPEG only** for now. Removes the EXIF segment and the ICC
  profile, losslessly (pixels untouched).

## Roadmap

- [ ] PNG and WebP stripping
- [ ] Remove XMP and IPTC blocks too, not just EXIF (these can also carry
      location/author data)
- [ ] Batch mode (a whole folder)
- [ ] `--dry-run` that reports what *would* be removed
- [ ] Optional file-manager / pre-commit-hook integration

## License

MIT 
