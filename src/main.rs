use skia_safe::textlayout::{
    FontCollection, ParagraphBuilder, ParagraphStyle, TextStyle, TypefaceFontProvider,
};
use skia_safe::{op, scalar, Color, Data, Font, FontMgr, FontStyle, ISize, Surface, Typeface};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

// == Execution ==

fn main() {
    all_with_slant();
    alternating_slant();
    twice_alternating_slant();
    alternating_slant_within_word();
    alternating_weight();
    alternating_size();
}

// == Scenarios ==

// Works as expected.
fn all_without_slant() {
    let (surface, _regular_style, mut paragraph_builder) = setup_paragraph_with_word_spacing();

    paragraph_builder.add_text("a b c d e f g");

    resolve(paragraph_builder, surface, "all_without_slant");
}

// Works as expected
fn all_with_slant() {
    let (surface, regular_style, mut paragraph_builder) = setup_paragraph_with_word_spacing();

    let mut italic_style = regular_style.clone();
    italic_style.set_font_style(FontStyle::italic());
    paragraph_builder.push_style(&italic_style);
    paragraph_builder.add_text("a b c d e f g");

    resolve(paragraph_builder, surface, "all_without_slant");
}

// BUG: No word spacing applied between 'b' and 'c'
fn alternating_slant() {
    let (surface, regular_style, mut paragraph_builder) = setup_paragraph_with_word_spacing();

    paragraph_builder.add_text("a b ");

    let mut italic_style = regular_style.clone();
    italic_style.set_font_style(FontStyle::italic());
    paragraph_builder.push_style(&italic_style);
    paragraph_builder.add_text(" c d e f g");

    resolve(paragraph_builder, surface, "alternating_slant_with_spacing");
}

// BUG: No word spacing applied between 'b' and 'c', and again not between 'e' and 'f'
fn twice_alternating_slant() {
    let (surface, regular_style, mut paragraph_builder) = setup_paragraph_with_word_spacing();

    paragraph_builder.add_text("a b ");

    let mut italic_style = regular_style.clone();
    italic_style.set_font_style(FontStyle::italic());
    paragraph_builder.push_style(&italic_style);
    paragraph_builder.add_text(" c d e ");

    paragraph_builder.push_style(&regular_style);
    paragraph_builder.add_text("f g");

    resolve(paragraph_builder, surface, "twice_alternating_slant");
}

// Works fine if the slant change happens within a word, not at a word boundary.
fn alternating_slant_within_word() {
    let (surface, regular_style, mut paragraph_builder) = setup_paragraph_with_word_spacing();

    paragraph_builder.add_text("Firs");

    let mut italic_style = regular_style.clone();
    italic_style.set_font_style(FontStyle::italic());
    paragraph_builder.push_style(&italic_style);
    paragraph_builder.add_text("t second");

    resolve(paragraph_builder, surface, "alternating_slant_within_word");
}

// BUG: Also affects font weight; again no word spacing applied between 'b' and 'c'
fn alternating_weight() {
    let (surface, regular_style, mut paragraph_builder) = setup_paragraph_with_word_spacing();

    paragraph_builder.add_text("a b ");

    let mut italic_style = regular_style.clone();
    italic_style.set_font_style(FontStyle::bold());
    paragraph_builder.push_style(&italic_style);
    paragraph_builder.add_text(" c d e f g");

    resolve(paragraph_builder, surface, "alternating_weight");
}

// BUG: Also affects font size; again no word spacing applied between 'b' and 'c'
fn alternating_size() {
    let (surface, regular_style, mut paragraph_builder) = setup_paragraph_with_word_spacing();

    paragraph_builder.add_text("a b ");

    let mut larger_style = regular_style.clone();
    larger_style.set_font_size(32.0);
    paragraph_builder.push_style(&larger_style);
    paragraph_builder.add_text(" c d e f g");

    resolve(paragraph_builder, surface, "alternating_size");
}

// == Helpers ==

fn setup_paragraph_with_word_spacing() -> (Surface, TextStyle, ParagraphBuilder) {
    let surface = Surface::new_raster_n32_premul(ISize::new(320, 240)).unwrap();
    let mut style = ParagraphStyle::new();

    let mut regular_style = TextStyle::new();
    regular_style.set_color(Color::from_rgb(0, 0, 0));
    regular_style.set_font_size(24.0);
    regular_style.set_font_style(FontStyle::normal());
    regular_style.set_font_families(&vec!["Open Sans"]);
    regular_style.set_word_spacing(40.0);
    style.set_text_style(&regular_style);

    let mut typeface_provider = TypefaceFontProvider::new();
    let open_sans =
        Typeface::from_data(data_from_file_path(Path::new("OpenSans-Regular.ttf")), None).unwrap();
    typeface_provider.register_typeface(open_sans, Some("Open Sans"));
    let mut font_collection = FontCollection::new();
    font_collection.set_asset_font_manager(Some(typeface_provider.clone().into()));
    font_collection.set_default_font_manager(Some(FontMgr::default()), None);

    let paragraph_builder = ParagraphBuilder::new(&style, font_collection);

    (surface, regular_style, paragraph_builder)
}

fn resolve(mut paragraph_builder: ParagraphBuilder, mut surface: Surface, test_name: &str) {
    let mut paragraph = paragraph_builder.build();
    paragraph.layout(500.0);

    let point = skia_safe::Point::new(0.0, 0.0);
    surface.canvas().clear(Color::from_rgb(255, 255, 255));
    paragraph.paint(surface.canvas(), point);

    save_png(&mut surface, format!("output/{}.png", test_name).as_str());
}

fn data_from_file_path(file_path: &Path) -> Data {
    let mut file = File::open(file_path).unwrap();
    let mut bytes = vec![];
    file.read_to_end(&mut bytes).unwrap();
    Data::new_copy(&bytes.as_slice())
}

fn save_png(surface: &mut Surface, path: &str) -> bool {
    let mut bytes: Vec<u8> = vec![];
    let image_info = surface.image_info();
    {
        let mut encoder = png::Encoder::new(
            &mut bytes,
            image_info.width() as u32,
            image_info.height() as u32,
        );
        encoder.set_color(png::ColorType::RGBA);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().expect("failed to write file header");

        let mut dst_pixels = vec![0; image_info.height() as usize * image_info.min_row_bytes()];
        let pixels_read = surface.read_pixels(
            &image_info,
            &mut dst_pixels,
            image_info.min_row_bytes(),
            (0, 0),
        );
        if !pixels_read {
            println!("failed to read pixels");
        }
        let result = writer.write_image_data(dst_pixels.as_slice());
        if let Err(reason) = result {
            println!("failed to write image data: {}", reason);
        }
    }
    let data = skia_safe::Data::new_copy(&bytes);

    let mut file = std::fs::File::create(path).expect("failed to create the file");
    file.write_all(data.as_bytes())
        .expect("failed to write data to the file");

    return true;
}
