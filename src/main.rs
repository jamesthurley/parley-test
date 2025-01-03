use image::{Pixel, Rgba, RgbaImage};
use imageproc::{
    drawing::{draw_filled_rect_mut, draw_hollow_rect_mut},
    rect::Rect,
};
use parley::{
    Alignment, FontFamily, FontWeight, Glyph, GlyphRun, PositionedLayoutItem, StyleProperty,
};
use swash::{
    scale::{image::Content, Render, ScaleContext, Scaler, Source, StrikeWith},
    zeno::{Format, Vector},
    FontRef,
};
use taffy::{
    prelude::length, AvailableSpace, Dimension, Display, FlexDirection, NodeId, Size, Style,
    TaffyTree,
};

//const TEXT : &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
const TEXT : &str = "The quick brown fox jumped over the lazy dog. The quick brown fox jumped over the lazy dog. The quick brown fox jumped over the lazy dog. The quick brown fox jumped over the lazy dog. The quick brown fox jumped over the lazy dog. The quick brown fox jumped over the lazy dog. The quick brown fox jumped over the lazy dog. The quick brown fox jumped over the lazy dog. The quick brown fox jumped over the lazy dog. The quick brown fox jumped over the lazy dog. The quick brown fox jumped over the lazy dog. The quick brown fox jumped over the lazy dog.";
//const TEXT : &str = "The quick brown fox jumped over the lazy dog. The quick brown fox jumped over the lazy dog. The quick brown fox jumped over the lazy dog.";

const OPEN_SANS_TTF: &[u8] = include_bytes!("../fonts/OpenSansVariable.ttf");
const OPEN_SANS_ITALIC_TTF: &[u8] = include_bytes!("../fonts/OpenSansItalicVariable.ttf");

const LINE_HEIGHT: f32 = 1.3;

fn main() -> Result<(), taffy::TaffyError> {
    let width = 500;
    let height = 300;

    let mut taffy: TaffyTree<NodeContext> = TaffyTree::new();

    let text_node =
        taffy.new_leaf_with_context(Style { ..Style::default() }, NodeContext::text(TEXT))?;
    let other_node = taffy.new_leaf(Style {
        flex_basis: Dimension::Auto,
        flex_grow: 1.,
        flex_shrink: 1.,
        ..Style::default()
    })?;

    let root = taffy.new_with_children(
        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            size: Size {
                width: Dimension::Percent(1.),
                height: Dimension::Percent(1.),
            },
            padding: taffy::geometry::Rect {
                top: length(10.),
                left: length(10.),
                right: length(10.),
                bottom: length(10.),
            },
            ..Default::default()
        },
        &[text_node, other_node],
    )?;

    let mut swash_scale_context = swash::scale::ScaleContext::new();
    let mut parley_layout_context = parley::LayoutContext::new();
    let mut parley_font_context = parley::FontContext::new();

    parley_font_context
        .collection
        .register_fonts(OPEN_SANS_TTF.into());
    parley_font_context
        .collection
        .register_fonts(OPEN_SANS_ITALIC_TTF.into());

    // Compute layout and print result
    taffy.compute_layout_with_measure(
        root,
        taffy::Size {
            width: AvailableSpace::Definite(width as f32),
            height: AvailableSpace::Definite(height as f32),
        },
        // Note: this closure is a FnMut closure and can be used to borrow external context for the duration of layout
        // For example, you may wish to borrow a global font registry and pass it into your text measuring function
        |known_dimensions, available_space, _node_id, node_context, _style| {
            measure_function(
                known_dimensions,
                available_space,
                node_context,
                &mut parley_font_context,
                &mut parley_layout_context,
            )
        },
    )?;

    taffy.print_tree(root);

    let mut image = RgbaImage::new(width, height);
    // Make image white
    draw_filled_rect_mut(
        &mut image,
        Rect::at(0, 0).of_size(width, height),
        Rgba([255, 255, 255, 255]),
    );

    // Draw red rectangle around text node
    let text_node_layout = taffy.layout(text_node).unwrap();
    println!(
        "Drawing text box at {},{} with size {}x{}",
        text_node_layout.location.x,
        text_node_layout.location.y,
        text_node_layout.size.width,
        text_node_layout.size.height
    );
    draw_hollow_rect_mut(
        &mut image,
        Rect::at(
            text_node_layout.location.x as i32,
            text_node_layout.location.y as i32,
        )
        .of_size(
            (text_node_layout.size.width as u32).max(1),
            (text_node_layout.size.height as u32).max(1),
        ),
        Rgba([255, 0, 0, 255]),
    );

    // Draw blue rectangle around other node
    let other_node_layout = taffy.layout(other_node).unwrap();
    println!(
        "Drawing other box at {},{} with size {}x{}",
        other_node_layout.location.x,
        other_node_layout.location.y,
        other_node_layout.size.width,
        other_node_layout.size.height
    );
    draw_hollow_rect_mut(
        &mut image,
        Rect::at(
            other_node_layout.location.x as i32,
            other_node_layout.location.y as i32,
        )
        .of_size(
            (other_node_layout.size.width as u32).max(1),
            (other_node_layout.size.height as u32).max(1),
        ),
        Rgba([0, 0, 255, 255]),
    );

    draw_override(
        &taffy,
        text_node,
        &mut image,
        &mut parley_font_context,
        &mut parley_layout_context,
        &mut swash_scale_context,
    );

    image.save("output.png").unwrap();

    Ok(())
}

pub(crate) struct TextBlockNodeContext {
    pub text: String,
}

impl TextBlockNodeContext {
    pub fn measure(
        &self,
        known_dimensions: taffy::geometry::Size<Option<f32>>,
        available_space: taffy::geometry::Size<taffy::style::AvailableSpace>,
        font_context: &mut parley::FontContext,
        layout_context: &mut parley::LayoutContext,
    ) -> Size<f32> {
        let width_constraint = known_dimensions.width.or(match available_space.width {
            AvailableSpace::MinContent => Some(0.0),
            AvailableSpace::MaxContent => None,
            AvailableSpace::Definite(width) => Some(width),
        });
        let text = &self.text;

        let layout = prepare_layout(layout_context, font_context, text, width_constraint);

        let width = layout.width();
        let height = layout.height();

        println!("Width constraint is: {:?}", width_constraint);
        println!("Measured text block: width: {}, height: {}", width, height);

        Size { width, height }
    }
}

fn draw_override(
    tree: &TaffyTree<NodeContext>,
    node_id: NodeId,
    image: &mut RgbaImage,
    font_context: &mut parley::FontContext,
    layout_context: &mut parley::LayoutContext,
    scale_context: &mut swash::scale::ScaleContext,
) {
    let node_layout = tree.layout(node_id).unwrap();

    let width_constraint = Some(node_layout.size.width);
    let text = TEXT;

    let layout = prepare_layout(layout_context, font_context, text, width_constraint);

    // Iterate over laid out lines
    for line in layout.lines() {
        // Iterate over GlyphRun's within each line
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(glyph_run) = item {
                render_glyph_run(
                    scale_context,
                    &glyph_run,
                    image,
                    node_layout.location.x,
                    node_layout.location.y,
                );
            }
        }
    }
}

fn prepare_layout(
    layout_context: &mut parley::LayoutContext,
    font_context: &mut parley::FontContext,
    text: &str,
    width_constraint: Option<f32>,
) -> parley::Layout<[u8; 4]> {
    const DISPLAY_SCALE: f32 = 1.0;
    let mut builder = layout_context.ranged_builder(font_context, text, DISPLAY_SCALE);

    builder.push_default(FontFamily::Named("Open Sans".into()));

    // Set default styles that apply to the entire layout
    builder.push_default(StyleProperty::LineHeight(LINE_HEIGHT));
    builder.push_default(StyleProperty::FontSize(16.0));

    // Set a style that applies to the first 4 characters
    builder.push(StyleProperty::FontWeight(FontWeight::new(600.0)), 0..4);

    // Build the builder into a Layout
    let mut layout = builder.build(text);

    // Run line-breaking and alignment on the Layout
    layout.break_all_lines(width_constraint);
    layout.align(width_constraint, Alignment::Start);
    layout
}

fn render_glyph_run(
    context: &mut ScaleContext,
    glyph_run: &GlyphRun<[u8; 4]>,
    image: &mut RgbaImage,
    x: f32,
    y: f32,
) {
    // Resolve properties of the GlyphRun
    let mut run_x = glyph_run.offset() + x;
    let run_y = glyph_run.baseline() + y;
    let style = glyph_run.style();
    let color = style.brush;

    // Get the "Run" from the "GlyphRun"
    let run = glyph_run.run();

    // Resolve properties of the Run
    let font = run.font();
    let font_size = run.font_size();
    let normalized_coords = run.normalized_coords();

    // Convert from parley::Font to swash::FontRef
    let font_ref = FontRef::from_index(font.data.as_ref(), font.index as usize).unwrap();

    // Build a scaler. As the font properties are constant across an entire run of glyphs
    // we can build one scaler for the run and reuse it for each glyph.
    let mut scaler = context
        .builder(font_ref)
        .size(font_size)
        .hint(true)
        .normalized_coords(normalized_coords)
        .build();

    // Iterates over the glyphs in the GlyphRun
    for glyph in glyph_run.glyphs() {
        let glyph_x = run_x + glyph.x;
        let glyph_y = run_y - glyph.y;
        run_x += glyph.advance;

        render_glyph(image, &mut scaler, color, glyph, glyph_x, glyph_y);
    }
}

fn render_glyph(
    image: &mut RgbaImage,
    scaler: &mut Scaler,
    color: [u8; 4],
    glyph: Glyph,
    glyph_x: f32,
    glyph_y: f32,
) {
    // Compute the fractional offset
    // You'll likely want to quantize this in a real renderer
    let offset = Vector::new(glyph_x.fract(), glyph_y.fract());

    // Render the glyph using swash
    let rendered_glyph = Render::new(
        // Select our source order
        &[
            Source::ColorOutline(0),
            Source::ColorBitmap(StrikeWith::BestFit),
            Source::Outline,
        ],
    )
    // Select the simple alpha (non-subpixel) format
    .format(Format::Alpha)
    // Apply the fractional offset
    .offset(offset)
    // Render the image
    .render(scaler, glyph.id)
    .unwrap();

    let glyph_width = rendered_glyph.placement.width;
    let glyph_height = rendered_glyph.placement.height;
    let Ok(glyph_x) = u32::try_from(glyph_x.floor() as i32 + rendered_glyph.placement.left) else {
        return;
    };
    let Ok(glyph_y) = u32::try_from(glyph_y.floor() as i32 - rendered_glyph.placement.top) else {
        return;
    };

    match rendered_glyph.content {
        Content::Mask => {
            let mut i = 0;
            for pixel_y in 0..glyph_height {
                for pixel_x in 0..glyph_width {
                    let x = glyph_x + pixel_x;
                    let y = glyph_y + pixel_y;
                    let alpha = rendered_glyph.data[i];
                    let color = Rgba([color[0], color[1], color[2], alpha]);
                    image.get_pixel_mut(x, y).blend(&color);
                    i += 1;
                }
            }
        }
        Content::SubpixelMask => unimplemented!(),
        Content::Color => {
            let row_size = glyph_width as usize * 4;
            for (pixel_y, row) in rendered_glyph.data.chunks_exact(row_size).enumerate() {
                for (pixel_x, pixel) in row.chunks_exact(4).enumerate() {
                    let x = glyph_x + pixel_x as u32;
                    let y = glyph_y + pixel_y as u32;
                    let color = Rgba(pixel.try_into().expect("Not RGBA"));
                    image.get_pixel_mut(x, y).blend(&color);
                }
            }
        }
    };
}

enum NodeContext {
    Text(TextBlockNodeContext),
}

impl NodeContext {
    /// Constructor for a text node context
    fn text(text: &str) -> Self {
        NodeContext::Text(TextBlockNodeContext {
            text: text.to_string(),
        })
    }
}

fn measure_function(
    known_dimensions: taffy::Size<Option<f32>>,
    available_space: taffy::Size<taffy::AvailableSpace>,
    node_context: Option<&mut NodeContext>,
    font_context: &mut parley::FontContext,
    layout_context: &mut parley::LayoutContext,
) -> Size<f32> {
    if let Size {
        width: Some(width),
        height: Some(height),
    } = known_dimensions
    {
        return Size { width, height };
    }

    match node_context {
        None => Size::ZERO,
        Some(NodeContext::Text(text_context)) => text_context.measure(
            known_dimensions,
            available_space,
            font_context,
            layout_context,
        ),
    }
}
