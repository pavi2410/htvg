// ============================================================================
// TypeScript types mirroring Rust element types (src/element.rs)
// These are the JSON schema types for HTVG documents.
// ============================================================================

/** A dimension value: pixels (number) or percentage (string like "50%"). */
export type Dimension = number | string;

/** Spacing: uniform (number) or multi-value string ("10 20" or "10 20 30 40"). */
export type Spacing = number | string;

/** Border radius: uniform (number) or per-corner string ("8 8 0 0"). */
export type BorderRadius = number | string;

/** CSS color string: hex (#RGB, #RRGGBB, #RRGGBBAA), rgb(), rgba(), or named. */
export type Color = string;

/** Font weight: number (100-900) or keyword. */
export type FontWeight = number | "normal" | "bold";

// ============================================================================
// Enum types
// ============================================================================

export type FlexDirection = "row" | "column" | "row-reverse" | "column-reverse";
export type JustifyContent =
  | "flex-start"
  | "flex-end"
  | "center"
  | "space-between"
  | "space-around"
  | "space-evenly";
export type AlignItems =
  | "flex-start"
  | "flex-end"
  | "center"
  | "stretch"
  | "baseline";
export type FlexWrap = "nowrap" | "wrap";
export type TextAlign = "left" | "center" | "right" | "justify";
export type ObjectFit = "contain" | "cover" | "fill";

// ============================================================================
// Style types
// ============================================================================

/** Style for Box elements (block container). */
export interface BoxStyle {
  display?: "block" | "flex" | "none";

  width?: Dimension;
  height?: Dimension;
  minWidth?: Dimension;
  maxWidth?: Dimension;
  minHeight?: Dimension;
  maxHeight?: Dimension;

  margin?: Spacing;
  padding?: Spacing;

  backgroundColor?: Color;
  borderWidth?: number;
  borderColor?: Color;
  borderRadius?: BorderRadius;
  opacity?: number;
}

/** Style for Flex elements (flex container). */
export interface FlexStyle {
  display?: "block" | "flex" | "none";

  width?: Dimension;
  height?: Dimension;
  minWidth?: Dimension;
  maxWidth?: Dimension;
  minHeight?: Dimension;
  maxHeight?: Dimension;

  margin?: Spacing;
  padding?: Spacing;

  flexDirection?: FlexDirection;
  justifyContent?: JustifyContent;
  alignItems?: AlignItems;
  gap?: number;
  flexWrap?: FlexWrap;

  backgroundColor?: Color;
  borderWidth?: number;
  borderColor?: Color;
  borderRadius?: BorderRadius;
  opacity?: number;
}

/** Style for Text elements. */
export interface TextStyle {
  fontFamily?: string;
  fontSize?: number;
  fontWeight?: FontWeight;
  lineHeight?: number;
  textAlign?: TextAlign;
  color?: Color;
  letterSpacing?: number;
  textRendering?: "text" | "vector";

  flexGrow?: number;
  flexShrink?: number;
}

/** Style for Image elements. */
export interface ImageStyle {
  width?: Dimension;
  height?: Dimension;
  minWidth?: Dimension;
  maxWidth?: Dimension;
  minHeight?: Dimension;
  maxHeight?: Dimension;

  margin?: Spacing;

  objectFit?: ObjectFit;

  borderRadius?: BorderRadius;
  opacity?: number;

  flexGrow?: number;
  flexShrink?: number;
}

// ============================================================================
// Element types
// ============================================================================

/** Block container element. */
export interface BoxElement {
  type: "box";
  style?: BoxStyle;
  children?: Element[];
}

/** Flex container element. */
export interface FlexElement {
  type: "flex";
  style?: FlexStyle;
  children?: Element[];
}

/** Text leaf element. */
export interface TextElement {
  type: "text";
  content: string;
  style?: TextStyle;
}

/** Image element with intrinsic dimensions. */
export interface ImageElement {
  type: "image";
  src: string;
  width: number;
  height: number;
  style?: ImageStyle;
}

/** Any HTVG element. */
export type Element = BoxElement | FlexElement | TextElement | ImageElement;

// ============================================================================
// Document & compilation types
// ============================================================================

/** Compilation options (mirrors Rust CompileOptions). */
export interface CompileOptions {
  /** Output width in pixels. */
  width: number;
  /** Output height in pixels (auto-computed if omitted). */
  height?: number;
  /** Base font size for relative units (default: 16). */
  baseFontSize?: number;
}

/** Self-contained HTVG document. */
export interface HtvgDocument {
  /** Compilation options. */
  meta?: CompileOptions;
  /** The element tree to render. */
  content: Element;
}

/** Compilation result (mirrors Rust CompileResult). */
export interface CompileResult {
  /** Generated SVG string. */
  svg: string;
  /** Computed width. */
  width: number;
  /** Computed height. */
  height: number;
  /** Any warnings during compilation. */
  warnings: string[];
}
