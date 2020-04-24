//! Types and traits for implementing syntax highlighting.

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]

/// This trait is to be implemented by any type that syntax highlights source code for a particular
/// language. This is done by taking in a string slice and outputting a vector of
/// [`Span`](struct.Span.html)s.
pub trait Highlight {
    /// Ensure that all input text is also contained in the `text` fields of the outputted `Span`s
    /// – in other words, this function must be lossless.
    fn highlight<'input>(&self, input: &'input str) -> Vec<Span<'input>>;
}

/// An individual fragment of highlighted text.
#[derive(Clone, Copy, Debug)]
pub struct Span<'text> {
    /// the text being highlighted
    pub text: &'text str,
    /// the highlight group it may have been assigned
    pub group: Option<HighlightGroup>,
}

/// The set of possible syntactical forms text can be assigned.
///
/// As it is certain that more variants will be added in future, this enum has been marked as
/// non-exhaustive.
#[non_exhaustive]
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, strum_macros::EnumIter)]
pub enum HighlightGroup {
    Keyword,
}

/// An RGB color.
#[derive(Clone, Copy, Debug)]
pub struct Rgb {
    /// red
    pub r: u8,
    /// green
    pub g: u8,
    /// blue
    pub b: u8,
}

/// The styling applied to a given [`HighlightGroup`](enum.HighlightGroup.html).
#[derive(Clone, Copy, Debug, Default)]
pub struct Style {
    /// its (optional) foreground color
    pub fg_color: Option<Rgb>,
    /// its (optional) background color
    pub bg_color: Option<Rgb>,
}

impl Style {
    /// Creates a new Style without a foreground or background colour.
    pub fn new() -> Self {
        Self {
            fg_color: None,
            bg_color: None,
        }
    }

    fn resolve(self, resolved: ResolvedStyle) -> ResolvedStyle {
        ResolvedStyle {
            fg_color: self.fg_color.unwrap_or(resolved.fg_color),
            bg_color: self.bg_color.unwrap_or(resolved.bg_color),
        }
    }
}

/// Identical to a [`Style`](struct.Style.html), except that it must have a background color. This
/// is outputted by (`render`)(fn.render.html), which resolves the background colour of every
/// [`Style`](struct.Style.html) it encounters.
#[derive(Clone, Copy, Debug)]
pub struct ResolvedStyle {
    /// its foreground color
    pub fg_color: Rgb,
    /// its background color
    pub bg_color: Rgb,
}

/// A trait for defining syntax highlighting themes.
pub trait Theme {
    /// The style for unhighlighted text. To understand why this must be a fully resolved style,
    /// consider the following example:
    ///
    /// - `default_style` returns a [`Style`](struct.Style.html) which omits a foreground color
    /// - at some point a [highlighter](trait.Highlight.html) returns a [`Span`](struct.Span.html)
    ///   without a highlight group
    /// - when [`render`](fn.render.html) is called, what is the foreground color of this
    ///   unhighlighted span?
    ///
    /// To prevent situations like this, `default_style` acts as a fallback for all cases by
    /// forcing the implementor to define all of the style’s fields.
    fn default_style(&self) -> ResolvedStyle;

    /// Provides a mapping from `HighlightGroup`s to `Style`s. As `HighlightGroup`s contain a
    /// variant for unhighlighted text, this thereby defines the appearance of the whole text
    /// field.
    fn style(&self, group: HighlightGroup) -> Style;
}

/// A convenience function that renders a given input text using a given highlighter and theme,
/// returning a vector of string slices and the (fully resolved) styles to apply to them.
pub fn render<'input, H, T>(
    input: &'input str,
    highlighter: H,
    theme: T,
) -> Vec<(&'input str, ResolvedStyle)>
where
    H: Highlight,
    T: Theme,
{
    use {std::collections::HashMap, strum::IntoEnumIterator};

    let styles: HashMap<_, _> = HighlightGroup::iter()
        .map(|group| (group, theme.style(group)))
        .collect();

    highlighter
        .highlight(input)
        .into_iter()
        .map(|span| {
            let resolved_style = if let Some(group) = span.group {
                styles[&group].resolve(theme.default_style())
            } else {
                theme.default_style()
            };

            (span.text, resolved_style)
        })
        .collect()
}
