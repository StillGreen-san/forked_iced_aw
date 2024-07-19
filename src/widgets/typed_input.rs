//! Display fields that can only be filled with a specific type.
//!
//! *This API requires the following crate features to be activated: `typed_input`*

use iced::mouse::{self, Cursor};
use iced::{widget::text_input::{self, TextInput}, Size, Event, event};
use iced::advanced::layout::{Node, Limits, Layout};
use iced::advanced::widget::{Tree, Widget, tree::{State, Tag}, Operation};
use iced::advanced::{Clipboard, Shell};
use iced::{Length, Rectangle, Element};

use std::{
    fmt::Display,
    str::FromStr,
};

/// The default padding
const DEFAULT_PADDING: f32 = 5.0;

/// A field that can only be filled with a specific type.
///
/// # Example
/// ```ignore
/// # use iced_aw::TypedInput;
/// #
/// #[derive(Debug, Clone)]
/// enum Message {
///     TypedInputChanged(u32),
/// }
///
/// let value = 12;
/// let max = 1275;
///
/// let input = TypedInput::new(
///     value,
///     Message::TypedInputChanged,
/// );
/// ```
pub struct TypedInput<'a, T, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: iced::advanced::text::Renderer<Font = iced::Font>,
    Theme: text_input::Catalog,
{
    /// The current value of the [`TypedInput`].
    value: T,
    /// The underlying element of the [`TypeInput`].
    text_input: text_input::TextInput<'a, InternalMessage, Theme, Renderer>,
    text: String,
    /// The ``on_change`` event of the [`TextInput`].
    on_change: Box<dyn Fn(T) -> Message>,
    /// The ``on_change`` event of the [`TextInput`].
    on_submit: Option<Message>,
    /// The font text of the [`TextInput`].
    font: Renderer::Font,
}

#[derive(Debug, Clone, PartialEq)]
enum InternalMessage {
    OnChange(String),
    OnSubmit,
}

impl<'a, T, Message, Theme, Renderer> TypedInput<'a, T, Message, Theme, Renderer>
where
    T: Display + FromStr,
    Message: Clone,
    Renderer: iced::advanced::text::Renderer<Font = iced::Font>,
    Theme: text_input::Catalog,
{
    /// Creates a new [`TypedInput`].
    ///
    /// It expects:
    /// - the current value
    /// - a function that produces a message when the [`TypedInput`] changes
    pub fn new<F>(value: T, on_changed: F) -> Self
    where
        F: 'static + Fn(T) -> Message + Copy,
        T: 'a + Clone,
    {
        let padding = DEFAULT_PADDING;
        // let move_value = value.clone();
        // let convert_to_t = move |s: String| on_changed(T::from_str(&s).unwrap_or(move_value.clone()));

        Self { 
            value: value.clone(), 
            text_input: text_input::TextInput::new("", format!("{value}").as_str())
                .on_input(InternalMessage::OnChange)
                .on_submit(InternalMessage::OnSubmit)
                .padding(padding)
                .width(Length::Fixed(127.0))
                .class(<Theme as text_input::Catalog>::default()), 
            text: value.to_string(),
            on_change: Box::new(on_changed),
            on_submit: None, 
            font: Renderer::Font::default(),
        }
    }

    /// Gets the text value of the [`TypedInput`].
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Sets the width of the [`TypedInput`].
    #[must_use]
    pub fn width(mut self, width: Length) -> Self {
        self.text_input = self.text_input.width(width);
        self
    }

    /// Sets the [`Font`] of the [`Text`].
    ///
    /// [`Font`]: core::Font
    /// [`Text`]: core::widget::Text
    #[allow(clippy::needless_pass_by_value)]
    #[must_use]
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self.text_input = self.text_input.font(font);
        self
    }

    /// Sets the message that should be produced when the [`TextInput`] is
    /// focused and the enter key is pressed.
    #[must_use]
    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }

    /// Sets the padding of the [`TypedInput`].
    #[must_use]
    pub fn padding(mut self, units: f32) -> Self {
        self.text_input = self.text_input.padding(units);
        self
    }

    /// Sets the text size of the [`TypedInput`].
    #[must_use]
    pub fn size(mut self, size: f32) -> Self {
        self.text_input = self.text_input.size(size);
        self
    }

    /// Sets the style of the input of the [`TypedInput`].
    #[must_use]
    pub fn style(
        mut self,
        style: impl Fn(&Theme, text_input::Status) -> text_input::Style + 'a,
    ) -> Self
    where
        <Theme as text_input::Catalog>::Class<'a>: From<text_input::StyleFn<'a, Theme>>,
    {
        self.text_input = self.text_input.style(style);
        self
    }

    /// Sets the class of the input of the [`TypedInput`].
    #[must_use]
    pub fn class(
        mut self,
        class: impl Into<<Theme as text_input::Catalog>::Class<'a>>,
    ) -> Self
    {
        self.text_input = self.text_input.class(class);
        self
    }
}

impl<'a, T, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TypedInput<'a, T, Message, Theme, Renderer>
where
    T: Display + FromStr + Clone + PartialEq,
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer<Font = iced::Font>,
    Theme: text_input::Catalog,
{
    fn tag(&self) -> Tag {
        Tag::of::<()>()
    }
    fn state(&self) -> State {
        State::new(())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree {
            tag: self.text_input.tag(),
            state: self.text_input.state(),
            children: self.text_input.children(),
        }]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children_custom(
            &[&self.text_input],
            |state, content| content.diff(state),
            |&content| Tree {
                tag: content.tag(),
                state: content.state(),
                children: content.children(),
            },
        );
    }

    fn size(&self) -> Size<Length> {
        <TextInput<_, _, _> as Widget<_, _, _>>::size(&self.text_input)
    }

    fn layout(&self, tree: &mut Tree,renderer: &Renderer, limits: &Limits) -> Node {
        let content = <TextInput<_, _, _> as Widget<_, _, _>>::layout(&self.text_input, &mut tree.children[0], renderer, limits);
        let size = limits.resolve(Length::Shrink, Length::Shrink, content.size());
        Node::with_children(
            size,
            vec![ content ]
        )
    }

    fn draw(&self, tree: &Tree, renderer: &mut Renderer, theme: &Theme, style: &iced::advanced::renderer::Style, layout: Layout<'_>, cursor: Cursor, viewport: &Rectangle) {
        let mut children = layout.children();
        let text_input_layout = children.next().expect("fail to get TextInput layout");
        <TextInput<_, _, _> as Widget<_, _, _>>::draw(&self.text_input, &tree.children[0], renderer, theme, style, text_input_layout, cursor, viewport);
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        <TextInput<_, _, _> as Widget<_, _, _>>::mouse_interaction(&self.text_input, &state.children[0], layout.children().next().expect("TypedInput inner child Textbox was not created."), cursor, viewport, renderer)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<()>,
    ) {
        <TextInput<_, _, _> as Widget<_, _, _>>::operate(&self.text_input, &mut tree.children[0], layout.children().next().expect("TypedInput inner child Textbox was not created."), renderer, operation)
    }

    #[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        let text_input_layout = layout.children().next().expect("fail to get text_input layout");

        let child = &mut state.children[0];

        let mut messages = Vec::new();
        let mut sub_shell = Shell::new(&mut messages);
        let status = self.text_input.on_event(
                child, event, text_input_layout, cursor, renderer, clipboard, &mut sub_shell, viewport,
            );
        // todo!()
        // println!("shell: {:?}", shell);
        if let Some(redraw) = sub_shell.redraw_request() {
            shell.request_redraw(redraw);
        }
        if sub_shell.is_layout_invalid() {
            shell.invalidate_layout();
        }
        if sub_shell.are_widgets_invalid() {
            shell.invalidate_widgets();
        }

        for message in messages {
            match message {
                InternalMessage::OnChange(value) => {
                    self.text = value;
                    if let Ok(val) = T::from_str(&self.text) {
                        if self.value != val {
                            self.value = val.clone();
                            shell.publish((self.on_change)(val));
                        }
                    }
                    shell.invalidate_layout();
                }
                InternalMessage::OnSubmit => {
                    if let Some(on_submit) = &self.on_submit {
                        shell.publish(on_submit.clone());
                    }
                }
            }
        }
        status
    }
        
}

impl<'a, T, Message, Theme, Renderer> From<TypedInput<'a, T, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    T: 'a + Display + FromStr + Clone + PartialEq,
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer<Font = iced::Font>,
    Theme: 'a + text_input::Catalog,
{
    fn from(typed_input: TypedInput<'a, T, Message, Theme, Renderer>) -> Self {
        Element::new(typed_input)
    }
}
