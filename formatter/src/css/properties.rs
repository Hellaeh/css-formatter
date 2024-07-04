pub use trie::Trie;

/// Groups of CSS properties. Will be used for grouping CSS properties separated by newline
#[repr(u8)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Group {
	// FIXME: Should be used for grids
	ParentLayout = 0,

	Positioning = 1,
	Layout = 2,
	BoxModel = 3,
	Display = 4,
	Typography = 5,
	Animation = 6,
	Transition = 7,
	Special = 8,

	/// Variables like: `--some-var: rgb(0, 1, 2);`
	Variable = u8::MAX - 1,

	/// Not yet defined in [`Properties`] or vendor prefixes
	/// `-webkit-line-clamp`
	Unknown = u8::MAX,
}

/// A CSS property descriptor
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub struct Descriptor {
	name: Name,
	group: Group,
	order: u16,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Name {
	inner: *const str,
}

impl std::ops::Deref for Name {
	type Target = str;

	#[inline(always)]
	fn deref(&self) -> &Self::Target {
		unsafe { &*self.inner }
	}
}

impl From<&str> for Name {
	#[inline]
	fn from(value: &str) -> Self {
		Self {
			inner: value as *const str,
		}
	}
}

macro_rules! group_css_props {
	($($group: expr => ($lexicographical_order: expr, $variant: ident, $repr: expr)$(,)?)*) => {
		/// Contains CSS properties
		/// Provided a handy method that returns property [`descriptor`](Descriptor)
		#[repr(u16)]
		#[allow(dead_code)]
		#[allow(clippy::enum_variant_names)]
		#[derive(Clone, Copy, Debug)]
		pub enum Property {
			$($variant),*
		}

		impl From<Property> for Descriptor {
			#[inline]
			fn from(value: Property) -> Self {
				match value {
					$(Property::$variant => Descriptor { name: $repr.into(), group: $group, order: value as u16 }),*
				}
			}
		}
	};
}

// We have Groups of properties and each group have a custom "lexicographical" order
group_css_props!(
	Group::Animation => (1, Animation, "animation"), // Creates an animating element
	Group::Animation => (1, AnimationName, "animation-name"), // Defines a name for the animation
	Group::Animation => (2, AnimationDelay, "animation-delay"), // Sets a delay before an animation begins
	Group::Animation => (2, AnimationDuration, "animation-duration"), // Defines the duration of an animation cycle
	Group::Animation => (2, AnimationTimingFunction, "animation-timing-function"), // Specifies the animation speed curve
	Group::Animation => (3, AnimationDirection, "animation-direction"), // Sets how, in which direction, an animation is played
	Group::Animation => (3, AnimationFillMode, "animation-fill-mode"), // Defines how styles are applied before and after animation
	Group::Animation => (3, AnimationIterationCount, "animation-iteration-count"), // Sets the number of times an animation is played
	Group::Animation => (3, AnimationPlayState, "animation-play-state"), // Sets the animation play state to running or paused

	Group::BoxModel => (100, AspectRatio, "aspect-ratio"),
	Group::BoxModel => (100, BoxSizing, "box-sizing"), // Sets how element height and width are calculated
	Group::BoxModel => (110, Height, "height"), // Sets the height of an element
	Group::BoxModel => (110, Width, "width"), // Sets the width of an element
	Group::BoxModel => (120, MaxHeight, "max-height"), // Sets the maximumn height for an element
	Group::BoxModel => (120, MaxWidth, "max-width"), // Sets the maximum width for an element
	Group::BoxModel => (120, MinHeight, "min-height"), // Sets the minimum height for an element
	Group::BoxModel => (120, MinWidth, "min-width"), // Sets the minimum width for an element
	Group::BoxModel => (200, Margin, "margin"), // Sets the margin (outside spacing), for an element
	Group::BoxModel => (210, MarginTop, "margin-top"), // Sets the top margin (outside spacing), for an element
	Group::BoxModel => (220, MarginRight, "margin-right"), // Sets the right margin (outside spacing), for an element
	Group::BoxModel => (230, MarginBottom, "margin-bottom"), // Sets the bottom margin (outside spacing), for an element
	Group::BoxModel => (240, MarginLeft, "margin-left"), // Sets the left margin (outside spacing), for an element
	Group::BoxModel => (250, MarginBlock, "margin-block"), // Defines the logical block start margin of an element
	Group::BoxModel => (250, MarginBlockStart, "margin-block-start"), // Defines the logical block start margin of an element
	Group::BoxModel => (251, MarginBlockEnd, "margin-block-end"), // Defines the logical block start margin of an element
	Group::BoxModel => (300, Border, "border"), // Specifies a border for an element
	Group::BoxModel => (300, BorderWidth, "border-width"), // Sets the border width of the element
	Group::BoxModel => (310, BorderTop, "border-top"), // Sets the top border of the element
	Group::BoxModel => (310, BorderTopWidth, "border-top-width"), // Sets the width of the top border
	Group::BoxModel => (320, BorderRight, "border-right"), // Sets the right border of the element
	Group::BoxModel => (320, BorderRightWidth, "border-right-width"), // Sets the width of the right border
	Group::BoxModel => (330, BorderBottom, "border-bottom"),
	Group::BoxModel => (330, BorderBottomWidth, "border-bottom-width"), // Sets the width of the bottom border
	Group::BoxModel => (340, BorderLeft, "border-left"), // Sets the left border of the element
	Group::BoxModel => (340, BorderLeftWidth, "border-left-width"), // Sets the width of the left border
	Group::BoxModel => (400, Padding, "padding"), // Sets the spacing between content and element border
	Group::BoxModel => (410, PaddingTop, "padding-top"), // Sets the spacing between content and top element border
	Group::BoxModel => (420, PaddingRight, "padding-right"), // Sets the spacing between content and right element border
	Group::BoxModel => (430, PaddingBottom, "padding-bottom"), // Sets the spacing between content and bottom element border
	Group::BoxModel => (440, PaddingLeft, "padding-left"), // Sets the spacing between content and left element border
	Group::BoxModel => (450, PaddingInlineStart, "padding-inline-start"),

	Group::Display => (1000, Visibility, "visibility"), // Specifies the visibility of an element
	Group::Display => (1010, Opacity, "opacity"), // Sets the opacity (transparency), of the element
	Group::Display => (1010, Overflow, "overflow"), // Specifies the flow of content that exceeds the container
	Group::Display => (1010, OverflowX, "overflow-x"), // Specifies the flow of content that exceeds the container width
	Group::Display => (1010, OverflowY, "overflow-y"), // Specifies the flow of content that exceeds the container height
	Group::Display => (1010, Transform, "transform"), // Applies a 2D or 3D transformation to an element
	Group::Display => (1010, TransformOrigin, "transform-origin"), // Sets the origin for the transformation of the element
	Group::Display => (1010, TransformStyle, "transform-style"), // Specifies the display behavior of 3D space nested elements
	Group::Display => (1011, Rotate, "rotate"), // Allows you to specify rotation transforms individually and independently of the transform property
	Group::Display => (1011, Scale, "scale"), // Allows you to specify scale transforms individually and independently of the transform property
	Group::Display => (1011, Translate, "translate"), // Allows you to specify translation transforms individually and independently of the transform property
	Group::Display => (1020, BoxShadow, "box-shadow"), // Adds a shadow effect to an element
	Group::Display => (1020, CaretColor, "caret-color"), // Sets the color of the blinking mouse caret
	Group::Display => (1020, ClipPath, "clip-path"), // Clips an element inside a specific shape or SVG
	Group::Display => (1020, Cursor, "cursor"), // Specifies the shape of the mouse cursor
	Group::Display => (1020, Filter, "filter"), // Defines effects (e.g. blurring or color shifting) on an element before the element is displayed
	Group::Display => (1100, Perspective, "perspective"), // Adds perspective to a 3DPositioned element
	Group::Display => (1100, PerspectiveOrigin, "perspective-origin"), // Sets the origin of the perspective for a 3DPositioned element
	Group::Display => (1200, AccentColor, "accent-color"), // Specifies the color to be used as the accent color.
	Group::Display => (2000, Background, "background"), // Sets the background of an element
	Group::Display => (2000, BackgroundColor, "background-color"), // Sets the background color of the element
	Group::Display => (2010, BackgroundAttachment, "background-attachment"), // Defines how the background is attached to an element
	Group::Display => (2010, BackgroundBlendMode, "background-blend-mode"), // Defines the background layer blending mode
	Group::Display => (2010, BackgroundClip, "background-clip"), // Defines how background extends beyond the element
	Group::Display => (2010, BackgroundImage, "background-image"), // Specifies a background image for an element
	Group::Display => (2010, BackgroundOrigin, "background-origin"), // Specifies the background image origin position
	Group::Display => (2010, BackgroundPosition, "background-position"), // Sets the position of a background image
	Group::Display => (2010, BackgroundRepeat, "background-repeat"), // Specifies how the background image is repeated
	Group::Display => (2010, BackgroundSize, "background-size"), // Sets the size of the background image
	Group::Display => (2020, BackdropFilter, "backdrop-filter"), // Defines a graphical effect to the area behind an element
	Group::Display => (2020, BackfaceVisibility, "backface-visibility"), // Shows or hides the backface visibility of an element
	Group::Display => (3000, BorderColor, "border-color"), // Sets the color of the border
	Group::Display => (3000, BorderImage, "border-image"), // Defines an image as border, instead of a color
	Group::Display => (3000, BorderImageOutset, "border-image-outset"), // Sets how far a border image extends beyond the border
	Group::Display => (3000, BorderImageRepeat, "border-image-repeat"), // Defines if and how the border image is repeated
	Group::Display => (3000, BorderImageSlice, "border-image-slice"), // Defines how the border image will be sliced
	Group::Display => (3000, BorderImageSource, "border-image-source"), // Specifies the url of the border image file
	Group::Display => (3000, BorderImageWidth, "border-image-width"), // Sets the width of the image border
	Group::Display => (3000, BorderRadius, "border-radius"), // Sets the radius of the border
	Group::Display => (3000, BorderStyle, "border-style"), // Defines the style of the border
	Group::Display => (3010, BorderTopColor, "border-top-color"), // Sets the color of the top border
	Group::Display => (3010, BorderTopLeftRadius, "border-top-left-radius"), // Sets the border radius of the top left corner
	Group::Display => (3010, BorderTopRightRadius, "border-top-right-radius"), // Sets the border radius of the top right corner
	Group::Display => (3010, BorderTopStyle, "border-top-style"), // Sets the style of the top border
	Group::Display => (3020, BorderRightColor, "border-right-color"), // Sets the color of the right border
	Group::Display => (3020, BorderRightStyle, "border-right-style"), // Sets the style of the right border
	Group::Display => (3030, BorderBottomColor, "border-bottom-color"), // Sets the color of a bottom border
	Group::Display => (3030, BorderBottomLeftRadius, "border-bottom-left-radius"), // Sets the border radius of the bottom left corner
	Group::Display => (3030, BorderBottomRightRadius, "border-bottom-right-radius"), // Sets the border radius of the bottom right corner
	Group::Display => (3030, BorderBottomStyle, "border-bottom-style"), // Sets the style of the bottom border
	Group::Display => (3040, BorderLeftColor, "border-left-color"), // Sets the color of the left border
	Group::Display => (3040, BorderLeftStyle, "border-left-style"), // Sets the style of the left border
	Group::Display => (4000, Outline , "outline"), // Adds an outline (highlighted border), to an element
	Group::Display => (4000, OutlineColor, "outline-color"), // Sets the color of an outline
	Group::Display => (4000, OutlineOffset, "outline-offset"), // Sets the space between the outline and border
	Group::Display => (4000, OutlineStyle, "outline-style"), // Sets the style of an outline
	Group::Display => (4000, OutlineWidth, "outline-width"), // Sets the width of an outline
	Group::Display => (5000, ListStyle, "list-style"), // Defines the markers (bullet points), for items in a list
	Group::Display => (5000, ListStyleImage, "list-style-image"), // Defines an image markers (bullet points), for items in a list
	Group::Display => (5000, ListStylePosition, "list-style-position"), // Sets the marker (bullet point), positions for items in a list
	Group::Display => (5000, ListStyleType , "list-style-type"), // Defines the marker types (bullet points), for items in a list

	Group::Layout => (100, Display, "display"), // Specify an element's display behavior
	Group::Layout => (101, Clear, "clear"), // Sets the element side that does not allow floating elements
	Group::Layout => (101, Float, "float"), // Sets how an element is positioned relative to other elements
	Group::Layout => (101, Gap, "gap"), // Sets the gaps (gutters) between rows and columns
	Group::Layout => (110, FlexDirection, "flex-direction"), // Specifies the direction for the flex item to align
	Group::Layout => (111, AlignContent, "align-content"), // Aligns items in a flex container along flex lines
	Group::Layout => (111, AlignItems, "align-items"), // Aligns evenly spaced items in a flex container
	Group::Layout => (111, AlignSelf, "align-self"), // Aligns an item inside a flex container
	Group::Layout => (111, JustifyContent, "justify-content"), // Specifies the alignment between the items inside a flexible container when the items do not use all available space
	Group::Layout => (111, Order, "order"), // Specifies the order of an item in a flex container
	Group::Layout => (112, Flex, "flex"), // Specifies the width of the flexible items
	Group::Layout => (112, FlexBasis, "flex-basis"), // Specifies the initial width of a flex item
	Group::Layout => (112, FlexFlow, "flex-flow"), // Controls the direction and wrapping of flexible items
	Group::Layout => (112, FlexGrow, "flex-grow"), // Specifies how a flex item can grow inside the container
	Group::Layout => (112, FlexShrink, "flex-shrink"), // Specifies how a flex item can shrink inside the container
	Group::Layout => (112, FlexWrap, "flex-wrap"), // Specifies how flexible items wrap inside the container
	Group::Layout => (120, Grid, "grid"), // Defines a grid layout with responsive rows and columns
	Group::Layout => (120, JustifyItems, "justify-items"), // Is set on the grid container. Specifies the alignment of grid items in the inline direction
	Group::Layout => (120, JustifySelf, "justify-self"), // Is set on the grid item. Specifies the alignment of the grid item in the inline direction
	Group::Layout => (121, GridTemplate, "grid-template"), // Divides a page into sections with a size, position, and layer
	Group::Layout => (121, GridTemplateAreas, "grid-template-areas"), // Specifies area in a grid container
	Group::Layout => (121, GridTemplateColumns, "grid-template-columns"), // Sets the number and width of columns in a grid container
	Group::Layout => (121, GridTemplateRows, "grid-template-rows"), // Sets the number and height of rows in a grid container
	Group::Layout => (122, GridArea, "grid-area"), // Sets the size and location of grid items in a grid container
	Group::Layout => (122, GridAutoColumns, "grid-auto-columns"), // Specifies the size of the columns in a grid container
	Group::Layout => (122, GridAutoFlow, "grid-auto-flow"), // Specifies the initial placement of items in a grid container
	Group::Layout => (122, GridAutoRows, "grid-auto-rows"), // Specifies the initial size of the items in a grid container
	Group::Layout => (122, GridColumn, "grid-column"), // Specifies the size and location of a grid item in a grid container
	Group::Layout => (122, GridColumnEnd, "grid-column-end"), // Specifies in which columnLine the grid item will end
	Group::Layout => (122, GridColumnGap, "grid-column-gap"), // Specifies the gap size between columns in a grid container
	Group::Layout => (122, GridColumnStart, "grid-column-start"), // Specifies in which column line the grid item will start
	Group::Layout => (122, GridGap, "grid-gap"), // Specifies the gap size between grid rows and columns
	Group::Layout => (122, GridRow, "grid-row"), // Specifies the grid item size and location in a grid container
	Group::Layout => (122, GridRowEnd, "grid-row-end"), // Specifies in which rowLine the grid item will end
	Group::Layout => (122, GridRowGap, "grid-row-gap"), // Specifies the gap size between rows in a grid container
	Group::Layout => (122, GridRowStart, "grid-row-start"), // Specifies in which row line the grid item will start
	Group::Layout => (130, BorderCollapse, "border-collapse"), // Sets table borders to single collapsed line or separated
	Group::Layout => (130, BorderSpacing, "border-spacing"), // Sets the adjacent table cell distance
	Group::Layout => (130, CaptionSide, "caption-side"), // Defines on which side of the table a caption is placed
	Group::Layout => (130, EmptyCells, "empty-cells"), // Specifies whether empty table cell borders will be displayed
	Group::Layout => (130, TableLayout, "table-layout"), // Aligns elements according to a table with rows and columns
	Group::Layout => (200, ObjectFit, "object-fit"), // Specifies how an image or video fits inside a container
	Group::Layout => (200, ObjectPosition, "object-position"), // Specifies the image or video position inside a container

	Group::Positioning => (1, Position, "position"), // Sets the element's positioning method
	Group::Positioning => (1, ZIndex, "z-index"), // Sets the vertical stacking order relative to other elements
	Group::Positioning => (2, Top, "top"), // Positions the element from the top of the relative container
	Group::Positioning => (3, Right, "right"), // Positions the element from the right of the relative container
	Group::Positioning => (4, Bottom, "bottom"), // Positions the element from the bottom of the relative container
	Group::Positioning => (5, Left, "left"), // Positions the element from the left of the relative container

	Group::Special => (1, All, "all"), // Resets all element properties to its default or inherited values
	Group::Special => (1, BreakAfter, "break-after"), // Adds a print pageBreak after an element
	Group::Special => (1, BreakBefore, "break-before"), // Adds a print pageBreak before an element
	Group::Special => (1, BreakInside, "break-inside"), // Specifies if print pageBreak is allowed inside an element
	Group::Special => (1, CounterIncrement, "counter-increment"), // Increase or decrease a CSS counter
	Group::Special => (1, CounterReset, "counter-reset"), // Initialize or reset CSS counter
	Group::Special => (1, OverscrollBehavior, "overscroll-behavior") // Sets what a browser does when reaching the boundary of a scrolling area
	Group::Special => (1, OverscrollBehaviorBlock, "overscroll-behavior-block") // Sets the browser's behavior when the block direction boundary of a scrolling area is reached
	Group::Special => (1, OverscrollBehaviorInline, "overscroll-behavior-inline") // Sets the browser's behavior when the inline direction boundary of a scrolling area is reached
	Group::Special => (1, OverscrollBehaviorX, "overscroll-behavior-x") // Sets the browser's behavior when the horizontal boundary of a scrolling area is reached
	Group::Special => (1, OverscrollBehaviorY, "overscroll-behavior-y") // Sets the browser's behavior when the vertical boundary of a scrolling area is reached
	Group::Special => (1, PointerEvents, "pointer-events"), // Specifies whether element reacts to pointer events or not
	Group::Special => (1, Resize, "resize"), // Sets whether an element is resizable, and if so, in which directions.
	Group::Special => (1, ScrollBehavior, "scroll-behavior"), // Specifies the scrolling behavior of an element
	Group::Special => (1, TouchAction, "touch-action") // Sets how an element's region can be manipulated by a touchscreen user
	Group::Special => (1, UserSelect, "user-select"), // Controls whether the user can select text
	Group::Special => (1, WillChange, "will-change"), // Hints to browsers how an element is expected to change

	Group::Transition => (1, Transition, "transition"), // Creates transitions from one property value to another
	Group::Transition => (1, TransitionDelay, "transition-delay"), // Creates a delay before the transition effect starts
	Group::Transition => (1, TransitionDuration, "transition-duration"), // Specifies the time the transition will take
	Group::Transition => (1, TransitionProperty, "transition-property"), // Specifies the CSS property that will transition
	Group::Transition => (1, TransitionTimingFunction, "transition-timing-function"), // Defines the speed curve function of the transition

	Group::Typography => (10, Color, "color"), // Specifies the color of text in an element
	Group::Typography => (10, Content, "content"), // Used to insert content before or after an element
	Group::Typography => (10, Direction, "direction"), // Specifies the text writing direction of a blockLevel element
	Group::Typography => (10, Font, "font"), // Sets font family, variant, weight, height, and size for an element
	Group::Typography => (10, FontFamily, "font-family"), // Sets the font family for an element
	Group::Typography => (10, FontSize, "font-size"), // Sets the size of the font for an element
	Group::Typography => (10, FontStyle, "font-style"), // Set the font style to normal, italic, or oblique
	Group::Typography => (10, FontVariationSettings, "font-variation-settings"), // Provides low-level control over variable font characteristics
	Group::Typography => (10, FontWeight, "font-weight"), // Sets the weight or thickness of the font
	Group::Typography => (10, LineHeight, "line-height"), // Sets the vertical spacing between lines of text
	Group::Typography => (20, FontFeatureSettings, "font-feature-settings"), // Allows control over advanced typographic features in OpenType fonts
	Group::Typography => (20, FontKerning, "font-kerning"), // Sets the spacing between the font's characters
	Group::Typography => (20, FontSizeAdjust, "font-size-adjust"), // Specifies a fallBack font size
	Group::Typography => (20, FontStretch, "font-stretch"), // Sets the text characters to a wider or narrower variant
	Group::Typography => (20, FontVariant, "font-variant"), // Specifies that text is displayed in a smallCaps font
	Group::Typography => (20, LetterSpacing, "letter-spacing"), // Sets the spacing between characters
	Group::Typography => (20, TabSize, "tab-size"), // Is used to customize the width of tab characters (U+0009)
	Group::Typography => (20, TextAlign, "text-align"), // Sets the alignment of text inside an element
	Group::Typography => (20, TextAlignLast, "text-align-last"), // Sets the alignment for the last line of text
	Group::Typography => (20, TextIndent, "text-indent"), // Sets the indentation to the beginning of text
	Group::Typography => (20, TextJustify, "text-justify"), // Defines the text justification inside a container
	Group::Typography => (20, TextOverflow, "text-overflow"), // Sets the display behavior of text that overflows a container
	Group::Typography => (20, VerticalAlign, "vertical-align"), // Specifies vertical alignment of an element
	Group::Typography => (20, WhiteSpace, "white-space"), // Specifies how whiteSpace is handled inside an element
	Group::Typography => (20, WordBreak, "word-break"), // Specifies how line breaks take place
	Group::Typography => (20, WordSpacing, "word-spacing"), // Sets the spacing between words
	Group::Typography => (20, WordWrap, "word-wrap"), // Specifies how long words can be wrapped
	Group::Typography => (20, WritingMode, "writing-mode"), // Sets the text reading orientation: top to bottom, etc
	Group::Typography => (30, Columns, "columns"), // Divide an element into columns of a certain width
	Group::Typography => (30, Widows, "widows"), // Sets the minimum number of lines in a block container that must be shown at the top of a page, region, or column
	Group::Typography => (31, ColumnCount, "column-count"), // Divides an element into the specified number of columns
	Group::Typography => (31, ColumnFill, "column-fill"), // Specifies how divided columns are filled
	Group::Typography => (31, ColumnGap, "column-gap"), // Specifies the space between divided columns
	Group::Typography => (31, ColumnRule, "column-rule"), // Sets the style, width, and color of a column divider
	Group::Typography => (31, ColumnRuleColor, "column-rule-color"), // Sets the color of a column divider
	Group::Typography => (31, ColumnRuleStyle, "column-rule-style"), // Sets the style of a column divider
	Group::Typography => (31, ColumnRuleWidth, "column-rule-width"), // Sets the width of a column divider
	Group::Typography => (31, ColumnSpan, "column-span"), // Sets number of divided columns an element should span
	Group::Typography => (31, ColumnWidth, "column-width"), // Specifies the width of a divided column
	Group::Typography => (50, FontVariantLigatures, "font-variant-ligatures") // Controls which ligatures and contextual forms are used in textual content of the elements it applies to
	Group::Typography => (50, Hyphens, "hyphens"), // Specifies hyphenation with wrap opportunities in a line of text
	Group::Typography => (50, Quotes, "quotes"), // Defines the quotation marks to be used on text
	Group::Typography => (50, TextDecoration, "text-decoration"), // Defines the style and color of underlined text
	Group::Typography => (50, TextDecorationColor, "text-decoration-color"), // Defines the color of underlined text
	Group::Typography => (50, TextDecorationLine, "text-decoration-line"), // Defines the kind of line to use with text
	Group::Typography => (50, TextDecorationStyle, "text-decoration-style"), // Defines the style of underlined text
	Group::Typography => (50, TextDecorationThickness, "text-decoration-thickness"), // Sets the stroke thickness of the decoration line
	Group::Typography => (50, TextShadow, "text-shadow"), // Adds a shadow effect to text
	Group::Typography => (50, TextTransform, "text-transform"), // Defines text capitalization or casing
	Group::Typography => (50, TextUnderlineOffset, "text-underline-offset"), // Sets the offset distance of an underline text
	Group::Typography => (60, TextRendering, "text-rendering") // Provides information to the rendering engine about what to optimize for when rendering text
);

impl Property {
	#[inline(always)]
	pub fn descriptor(self) -> Descriptor {
		std::convert::Into::<Descriptor>::into(self)
	}
}

impl Descriptor {
	#[inline]
	fn new(name: &str, group: Group) -> Self {
		Self {
			name: name.into(),
			group,
			order: 0,
		}
	}

	#[inline(always)]
	pub fn variable(name: &str) -> Self {
		Self::new(name, Group::Variable)
	}

	#[inline(always)]
	pub fn unknown(name: &str) -> Self {
		Self::new(name, Group::Unknown)
	}

	#[inline(always)]
	pub fn name(&self) -> &str {
		&self.name
	}

	#[inline(always)]
	pub fn group(&self) -> Group {
		self.group
	}
}

impl Ord for Descriptor {
	#[inline]
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self
			.group
			.cmp(&other.group)
			.then(self.order.cmp(&other.order))
			// WARN: Keep this last
			.then_with(|| self.name.cmp(&*other.name))
	}
}

impl PartialOrd for Descriptor {
	#[inline(always)]
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Default for Name {
	#[inline]
	fn default() -> Self {
		Self { inner: "" }
	}
}

impl Default for Group {
	#[inline]
	fn default() -> Self {
		Self::Unknown
	}
}

impl std::fmt::Debug for Name {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "\"{}\"", unsafe { &*self.inner })
	}
}

mod trie;

#[cfg(test)]
mod tests {
	use super::Name;

	#[test]
	fn name() {
		let name = Name::default();
		let s = unsafe { &*name.inner };

		assert!(s.is_empty())
	}
}
