use std::str::FromStr;

use super::{LocalName, Numbering, NumberingPattern};
use crate::layout::{BlockNode, TableNode, VNode};
use crate::prelude::*;
use crate::text::TextNode;

/// A figure with an optional caption.
///
/// ## Example
/// ```example
/// = Pipeline
/// @fig-lab shows the central step of
/// our molecular testing pipeline.
///
/// #figure(
///   image("molecular.jpg", width: 80%),
///   caption: [
///     The molecular testing pipeline.
///   ],
/// ) <fig-lab>
/// ```
///
/// Display: Figure
/// Category: meta
#[node(Locatable, Synthesize, Show, LocalName)]
pub struct FigureNode {
    /// The content of the figure. Often, an [image]($func/image).
    #[required]
    pub body: Content,

    /// The figure's caption.
    pub caption: Option<Content>,

    /// How to number the figure. Accepts a
    /// [numbering pattern or function]($func/numbering).
    #[default(Some(Numbering::Pattern(NumberingPattern::from_str("1").unwrap())))]
    pub numbering: Option<Numbering>,

    /// The vertical gap between the body and caption.
    #[default(Em::new(0.65).into())]
    pub gap: Length,

    /// The figure's number.
    #[synthesized]
    pub number: Option<NonZeroUsize>,
}

impl FigureNode {
    fn element(&self) -> NodeId {
        let mut id = self.body().id();
        if id != NodeId::of::<TableNode>() {
            id = NodeId::of::<Self>();
        }
        id
    }
}

impl Synthesize for FigureNode {
    fn synthesize(&mut self, vt: &Vt, styles: StyleChain) {
        let my_id = self.0.stable_id();
        let element = self.element();

        let mut number = None;
        let numbering = self.numbering(styles);
        if numbering.is_some() {
            number = NonZeroUsize::new(
                1 + vt
                    .locate_node::<Self>()
                    .take_while(|figure| figure.0.stable_id() != my_id)
                    .filter(|figure| figure.element() == element)
                    .count(),
            );
        }

        self.push_number(number);
        self.push_numbering(numbering);
    }
}

impl Show for FigureNode {
    fn show(&self, vt: &mut Vt, styles: StyleChain) -> SourceResult<Content> {
        let mut realized = self.body();

        if let Some(mut caption) = self.caption(styles) {
            if let Some(numbering) = self.numbering(styles) {
                let number = self.number().unwrap();
                let name = self.local_name(TextNode::lang_in(styles));
                caption = TextNode::packed(eco_format!("{name}\u{a0}"))
                    + numbering.apply(vt.world(), &[number])?.display()
                    + TextNode::packed(": ")
                    + caption;
            }

            realized += VNode::weak(self.gap(styles).into()).pack();
            realized += caption;
        }

        Ok(BlockNode::new()
            .with_body(Some(realized))
            .with_breakable(false)
            .pack()
            .aligned(Axes::with_x(Some(Align::Center.into()))))
    }
}

impl LocalName for FigureNode {
    fn local_name(&self, lang: Lang) -> &'static str {
        let body = self.body();
        if body.is::<TableNode>() {
            return body.with::<dyn LocalName>().unwrap().local_name(lang);
        }

        match lang {
            Lang::GERMAN => "Abbildung",
            Lang::ENGLISH | _ => "Figure",
        }
    }
}