//! SVG generation from render commands.

use crate::render::{Rect, RenderCommand, RenderTree};

/// Options for SVG generation.
#[derive(Debug, Clone)]
pub struct SvgOptions {
    /// Include XML declaration
    pub xml_declaration: bool,
    /// Pretty-print output
    pub pretty: bool,
    /// Decimal precision for coordinates
    pub precision: usize,
}

impl Default for SvgOptions {
    fn default() -> Self {
        Self {
            xml_declaration: true,
            pretty: false,
            precision: 2,
        }
    }
}

/// Generate SVG string from render tree.
pub fn generate_svg(tree: &RenderTree, options: &SvgOptions) -> String {
    let mut svg = SvgBuilder::new(tree.width, tree.height, options);

    for command in &tree.commands {
        svg.render_command(command);
    }

    svg.finish()
}

struct SvgBuilder<'a> {
    output: String,
    options: &'a SvgOptions,
    clip_id_counter: u32,
}

impl<'a> SvgBuilder<'a> {
    fn new(width: f32, height: f32, options: &'a SvgOptions) -> Self {
        let mut output = String::new();

        if options.xml_declaration {
            output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        }

        output.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" \
             xmlns:xlink=\"http://www.w3.org/1999/xlink\" \
             width=\"{:.p$}\" height=\"{:.p$}\" \
             viewBox=\"0 0 {:.p$} {:.p$}\">",
            width,
            height,
            width,
            height,
            p = options.precision
        ));

        Self {
            output,
            options,
            clip_id_counter: 0,
        }
    }

    fn render_command(&mut self, cmd: &RenderCommand) {
        match cmd {
            RenderCommand::FillRect {
                rect,
                color,
                border_radius,
            } => {
                self.render_fill_rect(rect, color, border_radius);
            }

            RenderCommand::StrokeRect {
                rect,
                color,
                width,
                border_radius,
            } => {
                self.render_stroke_rect(rect, color, *width, border_radius);
            }

            RenderCommand::Text {
                font_family,
                font_size,
                font_weight,
                color,
                lines,
                ..
            } => {
                self.render_text(font_family, *font_size, *font_weight, color, lines);
            }

            RenderCommand::TextPath { path_data, color } => {
                self.render_text_path(path_data, color);
            }

            RenderCommand::Image {
                rect,
                src,
                border_radius,
            } => {
                self.render_image(rect, src, border_radius);
            }

            RenderCommand::PushClip { rect, border_radius } => {
                self.push_clip(rect, border_radius);
            }

            RenderCommand::PopClip => {
                self.pop_clip();
            }

            RenderCommand::PushOpacity { opacity } => {
                self.push_opacity(*opacity);
            }

            RenderCommand::PopOpacity => {
                self.pop_opacity();
            }
        }
    }

    fn render_fill_rect(
        &mut self,
        rect: &Rect,
        color: &crate::element::Color,
        border_radius: &[f32; 4],
    ) {
        if color.a == 0 {
            return;
        }

        let p = self.options.precision;
        let has_radius = border_radius.iter().any(|&r| r > 0.0);

        if has_radius && !all_same(border_radius) {
            // Different corner radii - use path
            self.output.push_str(&format!(
                "<path d=\"{}\" fill=\"{}\"/>",
                rounded_rect_path(rect, border_radius, p),
                color.to_css()
            ));
        } else if has_radius {
            // Same radius on all corners
            self.output.push_str(&format!(
                "<rect x=\"{:.p$}\" y=\"{:.p$}\" width=\"{:.p$}\" height=\"{:.p$}\" \
                 rx=\"{:.p$}\" fill=\"{}\"/>",
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                border_radius[0],
                color.to_css(),
                p = p
            ));
        } else {
            // No radius
            self.output.push_str(&format!(
                "<rect x=\"{:.p$}\" y=\"{:.p$}\" width=\"{:.p$}\" height=\"{:.p$}\" \
                 fill=\"{}\"/>",
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                color.to_css(),
                p = p
            ));
        }
    }

    fn render_stroke_rect(
        &mut self,
        rect: &Rect,
        color: &crate::element::Color,
        stroke_width: f32,
        border_radius: &[f32; 4],
    ) {
        if color.a == 0 || stroke_width <= 0.0 {
            return;
        }

        let p = self.options.precision;
        let has_radius = border_radius.iter().any(|&r| r > 0.0);

        // Inset the rect by half stroke width for proper border rendering
        let inset = stroke_width / 2.0;
        let inner_rect = Rect {
            x: rect.x + inset,
            y: rect.y + inset,
            width: rect.width - stroke_width,
            height: rect.height - stroke_width,
        };

        if has_radius && !all_same(border_radius) {
            self.output.push_str(&format!(
                "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.p$}\"/>",
                rounded_rect_path(&inner_rect, border_radius, p),
                color.to_css(),
                stroke_width,
                p = p
            ));
        } else if has_radius {
            self.output.push_str(&format!(
                "<rect x=\"{:.p$}\" y=\"{:.p$}\" width=\"{:.p$}\" height=\"{:.p$}\" \
                 rx=\"{:.p$}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.p$}\"/>",
                inner_rect.x,
                inner_rect.y,
                inner_rect.width,
                inner_rect.height,
                border_radius[0],
                color.to_css(),
                stroke_width,
                p = p
            ));
        } else {
            self.output.push_str(&format!(
                "<rect x=\"{:.p$}\" y=\"{:.p$}\" width=\"{:.p$}\" height=\"{:.p$}\" \
                 fill=\"none\" stroke=\"{}\" stroke-width=\"{:.p$}\"/>",
                inner_rect.x,
                inner_rect.y,
                inner_rect.width,
                inner_rect.height,
                color.to_css(),
                stroke_width,
                p = p
            ));
        }
    }

    fn render_text(
        &mut self,
        font_family: &str,
        font_size: f32,
        font_weight: u16,
        color: &crate::element::Color,
        lines: &[crate::render::TextLineRender],
    ) {
        let p = self.options.precision;

        if lines.is_empty() {
            return;
        }

        if lines.len() == 1 {
            let line = &lines[0];
            self.output.push_str(&format!(
                "<text x=\"{:.p$}\" y=\"{:.p$}\" \
                 fill=\"{}\" \
                 font-family=\"{}\" \
                 font-size=\"{:.p$}\" \
                 font-weight=\"{}\">{}</text>",
                line.x,
                line.y,
                color.to_css(),
                escape_xml(font_family),
                font_size,
                font_weight,
                escape_xml(&line.text),
                p = p
            ));
        } else {
            self.output.push_str(&format!(
                "<text fill=\"{}\" \
                 font-family=\"{}\" \
                 font-size=\"{:.p$}\" \
                 font-weight=\"{}\">",
                color.to_css(),
                escape_xml(font_family),
                font_size,
                font_weight,
                p = p
            ));
            for line in lines {
                self.output.push_str(&format!(
                    "<tspan x=\"{:.p$}\" y=\"{:.p$}\">{}</tspan>",
                    line.x,
                    line.y,
                    escape_xml(&line.text),
                    p = p
                ));
            }
            self.output.push_str("</text>");
        }
    }

    fn render_text_path(&mut self, path_data: &str, color: &crate::element::Color) {
        self.output.push_str(&format!(
            "<path d=\"{}\" fill=\"{}\"/>",
            path_data,
            color.to_css()
        ));
    }

    fn render_image(&mut self, rect: &Rect, src: &str, border_radius: &[f32; 4]) {
        let p = self.options.precision;
        let has_radius = border_radius.iter().any(|&r| r > 0.0);

        if has_radius {
            // Need to clip the image
            let clip_id = self.clip_id_counter;
            self.clip_id_counter += 1;

            self.output.push_str("<defs>");
            self.output
                .push_str(&format!("<clipPath id=\"img-clip-{}\">", clip_id));

            if all_same(border_radius) {
                self.output.push_str(&format!(
                    "<rect x=\"{:.p$}\" y=\"{:.p$}\" width=\"{:.p$}\" height=\"{:.p$}\" rx=\"{:.p$}\"/>",
                    rect.x, rect.y, rect.width, rect.height, border_radius[0],
                    p = p
                ));
            } else {
                self.output.push_str(&format!(
                    "<path d=\"{}\"/>",
                    rounded_rect_path(rect, border_radius, p)
                ));
            }

            self.output.push_str("</clipPath></defs>");

            self.output.push_str(&format!(
                "<image x=\"{:.p$}\" y=\"{:.p$}\" width=\"{:.p$}\" height=\"{:.p$}\" \
                 xlink:href=\"{}\" clip-path=\"url(#img-clip-{})\"/>",
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                escape_xml(src),
                clip_id,
                p = p
            ));
        } else {
            self.output.push_str(&format!(
                "<image x=\"{:.p$}\" y=\"{:.p$}\" width=\"{:.p$}\" height=\"{:.p$}\" \
                 xlink:href=\"{}\"/>",
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                escape_xml(src),
                p = p
            ));
        }
    }

    fn push_clip(&mut self, rect: &Rect, border_radius: &[f32; 4]) {
        let clip_id = self.clip_id_counter;
        self.clip_id_counter += 1;
        let p = self.options.precision;

        self.output.push_str("<defs>");
        self.output
            .push_str(&format!("<clipPath id=\"clip-{}\">", clip_id));

        if border_radius.iter().any(|&r| r > 0.0) {
            if all_same(border_radius) {
                self.output.push_str(&format!(
                    "<rect x=\"{:.p$}\" y=\"{:.p$}\" width=\"{:.p$}\" height=\"{:.p$}\" rx=\"{:.p$}\"/>",
                    rect.x, rect.y, rect.width, rect.height, border_radius[0],
                    p = p
                ));
            } else {
                self.output.push_str(&format!(
                    "<path d=\"{}\"/>",
                    rounded_rect_path(rect, border_radius, p)
                ));
            }
        } else {
            self.output.push_str(&format!(
                "<rect x=\"{:.p$}\" y=\"{:.p$}\" width=\"{:.p$}\" height=\"{:.p$}\"/>",
                rect.x, rect.y, rect.width, rect.height,
                p = p
            ));
        }

        self.output.push_str("</clipPath></defs>");
        self.output
            .push_str(&format!("<g clip-path=\"url(#clip-{})\">", clip_id));
    }

    fn pop_clip(&mut self) {
        self.output.push_str("</g>");
    }

    fn push_opacity(&mut self, opacity: f32) {
        self.output
            .push_str(&format!("<g opacity=\"{:.2}\">", opacity));
    }

    fn pop_opacity(&mut self) {
        self.output.push_str("</g>");
    }

    fn finish(mut self) -> String {
        self.output.push_str("</svg>");
        self.output
    }
}

/// Generate SVG path for rounded rectangle.
fn rounded_rect_path(rect: &Rect, radii: &[f32; 4], precision: usize) -> String {
    let [tl, tr, br, bl] = *radii;
    let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);

    // Clamp radii to fit in rect
    let max_radius = (w / 2.0).min(h / 2.0);
    let tl = tl.min(max_radius);
    let tr = tr.min(max_radius);
    let br = br.min(max_radius);
    let bl = bl.min(max_radius);

    format!(
        "M {:.p$},{:.p$} \
         L {:.p$},{:.p$} \
         Q {:.p$},{:.p$} {:.p$},{:.p$} \
         L {:.p$},{:.p$} \
         Q {:.p$},{:.p$} {:.p$},{:.p$} \
         L {:.p$},{:.p$} \
         Q {:.p$},{:.p$} {:.p$},{:.p$} \
         L {:.p$},{:.p$} \
         Q {:.p$},{:.p$} {:.p$},{:.p$} \
         Z",
        // Start after top-left corner
        x + tl,
        y,
        // Top edge to top-right corner
        x + w - tr,
        y,
        // Top-right corner
        x + w,
        y,
        x + w,
        y + tr,
        // Right edge to bottom-right corner
        x + w,
        y + h - br,
        // Bottom-right corner
        x + w,
        y + h,
        x + w - br,
        y + h,
        // Bottom edge to bottom-left corner
        x + bl,
        y + h,
        // Bottom-left corner
        x,
        y + h,
        x,
        y + h - bl,
        // Left edge to top-left corner
        x,
        y + tl,
        // Top-left corner
        x,
        y,
        x + tl,
        y,
        p = precision
    )
}

fn all_same(arr: &[f32; 4]) -> bool {
    arr.iter().all(|&x| (x - arr[0]).abs() < 0.001)
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("<test>"), "&lt;test&gt;");
        assert_eq!(escape_xml("a & b"), "a &amp; b");
    }

    #[test]
    fn test_all_same() {
        assert!(all_same(&[5.0, 5.0, 5.0, 5.0]));
        assert!(!all_same(&[5.0, 5.0, 0.0, 5.0]));
    }
}
