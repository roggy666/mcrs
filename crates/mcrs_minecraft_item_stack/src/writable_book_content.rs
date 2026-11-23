use alloc::borrow::Cow;
use alloc::string::String;

use crate::Filterable;
use crate::entity_form::EntityForm;

pub(crate) const MAX_PAGES: usize = 100;
pub(crate) const MAX_PAGE_UNITS: usize = 1024;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub struct WritableBookContent {
    pub pages: Cow<'static, [Filterable<String>]>,
}

impl WritableBookContent {
    pub const EMPTY: Self = Self {
        pages: Cow::Borrowed(&[]),
    };

    #[must_use]
    pub fn new(pages: impl Into<Cow<'static, [Filterable<String>]>>) -> Option<Self> {
        let pages: Cow<'static, [Filterable<String>]> = pages.into();
        let fits = pages.len() <= MAX_PAGES && pages.iter().all(is_writable_page);
        fits.then_some(Self { pages })
    }
}

impl EntityForm for WritableBookContent {
    fn borrow_static(&'static self) -> Self {
        Self {
            pages: Cow::Borrowed(&self.pages),
        }
    }
}

pub(crate) fn is_writable_page(page: &Filterable<String>) -> bool {
    fits(&page.raw) && page.filtered.as_deref().is_none_or(fits)
}

fn fits(text: &str) -> bool {
    text.encode_utf16().count() <= MAX_PAGE_UNITS
}
