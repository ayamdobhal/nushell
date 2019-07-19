use crate::Text;
use derive_new::new;
use getset::Getters;
use serde::Serialize;
use serde_derive::Deserialize;
use uuid::Uuid;

#[derive(
    new, Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Hash, Getters,
)]
#[get = "crate"]
pub struct Spanned<T> {
    pub span: Span,
    pub item: T,
}

impl<T> Spanned<T> {
    pub fn spanned(self, span: impl Into<Span>) -> Spanned<T> {
        Spanned::from_item(self.item, span.into())
    }
}

pub trait SpannedItem: Sized {
    fn spanned(self, span: impl Into<Span>) -> Spanned<Self> {
        Spanned::from_item(self, span.into())
    }

    // For now, this is a temporary facility. In many cases, there are other useful spans that we
    // could be using, such as the original source spans of JSON or Toml files, but we don't yet
    // have the infrastructure to make that work.
    fn spanned_unknown(self) -> Spanned<Self> {
        Spanned::from_item(self, (0, 0))
    }
}

impl<T> SpannedItem for T {}

impl<T> std::ops::Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.item
    }
}

impl<T> Spanned<T> {
    crate fn from_item(item: T, span: impl Into<Span>) -> Spanned<T> {
        Spanned {
            span: span.into(),
            item,
        }
    }

    pub fn map<U>(self, input: impl FnOnce(T) -> U) -> Spanned<U> {
        let Spanned { span, item } = self;

        let mapped = input(item);
        Spanned { span, item: mapped }
    }

    crate fn copy_span<U>(&self, output: U) -> Spanned<U> {
        let Spanned { span, .. } = self;

        Spanned {
            span: *span,
            item: output,
        }
    }

    pub fn source(&self, source: &Text) -> Text {
        Text::from(self.span().slice(source))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize, Hash)]
pub struct Span {
    crate start: usize,
    crate end: usize,
    pub source: Option<Uuid>,
}

impl From<Option<Span>> for Span {
    fn from(input: Option<Span>) -> Span {
        match input {
            None => Span {
                start: 0,
                end: 0,
                source: None,
            },
            Some(span) => span,
        }
    }
}

impl<T> From<&Spanned<T>> for Span {
    fn from(input: &Spanned<T>) -> Span {
        input.span
    }
}

impl From<&Span> for Span {
    fn from(input: &Span) -> Span {
        *input
    }
}

impl From<nom5_locate::LocatedSpan<&str>> for Span {
    fn from(input: nom5_locate::LocatedSpan<&str>) -> Span {
        Span {
            start: input.offset,
            end: input.offset + input.fragment.len(),
            source: None,
        }
    }
}

impl<T> From<(nom5_locate::LocatedSpan<T>, nom5_locate::LocatedSpan<T>)> for Span {
    fn from(input: (nom5_locate::LocatedSpan<T>, nom5_locate::LocatedSpan<T>)) -> Span {
        Span {
            start: input.0.offset,
            end: input.1.offset,
            source: None,
        }
    }
}

impl From<(usize, usize)> for Span {
    fn from(input: (usize, usize)) -> Span {
        Span {
            start: input.0,
            end: input.1,
            source: None,
        }
    }
}

impl From<&std::ops::Range<usize>> for Span {
    fn from(input: &std::ops::Range<usize>) -> Span {
        Span {
            start: input.start,
            end: input.end,
            source: None,
        }
    }
}

impl Span {
    pub fn unknown() -> Span {
        Span {
            start: 0,
            end: 0,
            source: None,
        }
    }

    pub fn unknown_with_uuid(uuid: Uuid) -> Span {
        Span {
            start: 0,
            end: 0,
            source: Some(uuid),
        }
    }

    pub fn is_unknown(&self) -> bool {
        self.start == 0 && self.end == 0
    }

    pub fn slice(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end]
    }
}

impl language_reporting::ReportingSpan for Span {
    fn with_start(&self, start: usize) -> Self {
        Span {
            start,
            end: self.end,
            source: None,
        }
    }

    fn with_end(&self, end: usize) -> Self {
        Span {
            start: self.start,
            end,
            source: None,
        }
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}
