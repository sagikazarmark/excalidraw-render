//! Scalars that no family face and not Xiaolai maps (Greek, arrows, subscript
//! digits) are drawn from the bundled generic-family fallbacks the way the
//! reference's browser draws them from `sans-serif`: Liberation Sans first,
//! then Cascadia Code. Nothing is replaced by `?`, no `font.missing-glyph` is
//! reported, and the SVG embeds a subset of each face it used. A scalar no
//! bundled face maps still reports `font.missing-glyph`.

use excalidraw_render::{DiagnosticCode, OutputFormat, SceneDocument};

use crate::common::pdf_inspect::{declares_base_font, pdf_text};
use crate::common::render::render;
use crate::common::svg::{
    font_family_attribute, CASCADIA_FAMILIES, LIBERATION_FAMILIES, NUNITO_FAMILIES,
};

/// Latin in Nunito, then `→` and `α` (both in Liberation Sans) and the
/// subscript digits `₁ ₂ ₃` (only in Cascadia Code).
const TEXT: &str = "a \u{2192} b, \u{3b1}\u{2081} \u{2082} \u{2083}";

fn text_scene(font_family: i32, text: &str) -> SceneDocument {
    let source = format!(
        "{{\"type\":\"excalidraw\",\"version\":2,\"elements\":[{{\"id\":\"text\",\"type\":\"text\",\"width\":300,\"height\":25,\"fontSize\":20,\"fontFamily\":{font_family},\"text\":{},\"updated\":1}}],\"appState\":{{}},\"files\":{{}}}}",
        serde_json::to_string(text).expect("test text serializes")
    );
    SceneDocument::parse(source.as_bytes()).expect("text scene is admitted")
}

#[test]
fn every_format_draws_arrows_greek_and_subscripts_without_a_diagnostic() {
    let scene = text_scene(6, TEXT);
    for format in [OutputFormat::Svg, OutputFormat::Png, OutputFormat::Pdf] {
        let outcome = render(&scene, format);
        assert!(
            outcome.report().is_empty(),
            "{format:?} reports no missing glyph: {:?}",
            outcome.report().diagnostics()
        );
    }
}

#[test]
fn svg_sets_each_scalar_from_the_first_fallback_that_maps_it_and_embeds_subsets() {
    let outcome = render(&text_scene(6, TEXT), OutputFormat::Svg);
    let svg = std::str::from_utf8(outcome.artifact().bytes()).expect("SVG is UTF-8");

    // The stored text travels beside the glyphs; nothing became `?`.
    assert!(svg.contains(&format!("data-source-text=\"{TEXT}\"")));
    assert!(!svg.contains(">?</tspan>"));

    let nunito = format!("<tspan {}", font_family_attribute(NUNITO_FAMILIES));
    let liberation = format!("<tspan {}", font_family_attribute(LIBERATION_FAMILIES));
    let cascadia = format!("<tspan {}", font_family_attribute(CASCADIA_FAMILIES));
    // Nunito keeps the Latin, Liberation Sans takes `→` and `α`, and only
    // Cascadia Code maps the subscript digits.
    assert!(svg.contains(&nunito));
    assert!(svg.contains(&format!("{liberation}")));
    assert!(svg.contains(&cascadia));
    for (family, scalar) in [
        (&liberation, '\u{2192}'),
        (&liberation, '\u{3b1}'),
        (&cascadia, '\u{2081}'),
        (&cascadia, '\u{2082}'),
        (&cascadia, '\u{2083}'),
    ] {
        let mut runs = svg.split(family.as_str()).skip(1);
        assert!(
            runs.any(|run| run
                .split("</tspan>")
                .next()
                .is_some_and(|content| content.contains(scalar))),
            "{scalar:?} is shaped by {family}"
        );
    }

    // Each face a run uses has its own derived subset in the SVG.
    for face in ["Nunito", "ExcalidrawRender-9", "ExcalidrawRender-3"] {
        assert!(
            svg.contains(&format!("@font-face{{font-family:'{face}'")),
            "{face} subset is embedded"
        );
    }
}

#[test]
fn pdf_embeds_the_fallback_faces_it_used() {
    let outcome = render(&text_scene(6, TEXT), OutputFormat::Pdf);
    let pdf = pdf_text(outcome.artifact().bytes());
    assert!(declares_base_font(&pdf, "Nunito"));
    assert!(declares_base_font(&pdf, "LiberationSans"));
    assert!(declares_base_font(&pdf, "CascadiaCode"));
}

#[test]
fn a_scalar_no_bundled_face_maps_is_still_missing() {
    // Linear B U+10000 is in no bundled face, so it stays the replacement and
    // is reported once, while the arrow beside it is not.
    let outcome = render(&text_scene(6, "\u{2192}\u{10000}"), OutputFormat::Svg);
    let diagnostics = outcome.report().diagnostics();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].code(), DiagnosticCode::FONT_MISSING_GLYPH);
}
