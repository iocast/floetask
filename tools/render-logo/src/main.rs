//! Renders `assets/logo.svg` into the raster icons committed next to it.
//!
//! Run from the workspace root after changing the logo:
//! `cargo run -p render-logo`

use std::fs::File;
use std::path::Path;

use resvg::{tiny_skia, usvg};

const SIZES: [u32; 7] = [16, 24, 32, 48, 64, 128, 256];
const PNG_SIZES: [u32; 2] = [256, 512];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let assets = Path::new("assets");
    let tree = usvg::Tree::from_data(&std::fs::read(assets.join("logo.svg"))?, &usvg::Options::default())?;

    for size in PNG_SIZES {
        render(&tree, size).save_png(assets.join(format!("icon-{size}.png")))?;
    }

    let mut icon = ico::IconDir::new(ico::ResourceType::Icon);
    for size in SIZES {
        let pixmap = render(&tree, size);
        let image = ico::IconImage::from_rgba_data(size, size, unpremultiply(&pixmap));
        icon.add_entry(ico::IconDirEntry::encode(&image)?);
    }
    icon.write(File::create(assets.join("floetask.ico"))?)?;
    println!("wrote icons to {}", assets.display());
    Ok(())
}

fn render(tree: &usvg::Tree, size: u32) -> tiny_skia::Pixmap {
    let mut pixmap = tiny_skia::Pixmap::new(size, size).expect("non-zero size");
    let scale = size as f32 / tree.size().width();
    resvg::render(
        tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    pixmap
}

/// tiny-skia stores premultiplied alpha; ICO wants straight alpha.
fn unpremultiply(pixmap: &tiny_skia::Pixmap) -> Vec<u8> {
    pixmap
        .pixels()
        .iter()
        .flat_map(|pixel| {
            let color = pixel.demultiply();
            [color.red(), color.green(), color.blue(), color.alpha()]
        })
        .collect()
}
