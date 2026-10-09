use std::{env, fs::File, io::BufWriter, path::PathBuf};

use formats::SpriteDocument;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .ok_or("usage: pxs-to-png INPUT.pxs OUTPUT.png")?,
    );
    let output = PathBuf::from(
        args.next()
            .ok_or("usage: pxs-to-png INPUT.pxs OUTPUT.png")?,
    );
    if args.next().is_some() {
        return Err("usage: pxs-to-png INPUT.pxs OUTPUT.png".into());
    }

    let document = SpriteDocument::decode(&std::fs::read(input)?)?;
    let mut rgba = Vec::with_capacity(document.pixels.pixels().len() * 4);
    for &index in document.pixels.pixels() {
        if index == 0 {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            let [red, green, blue] = api::Color::ALL[usize::from(index - 1)].rgb();
            rgba.extend_from_slice(&[red, green, blue, 255]);
        }
    }

    let file = BufWriter::new(File::create(output)?);
    let mut encoder = png::Encoder::new(file, document.size.x, document.size.y);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(&rgba)?;
    Ok(())
}
