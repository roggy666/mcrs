use alloc::borrow::Cow;
use alloc::string::String;
use core::fmt;
use core::fmt::Write;
use core::hash::Hash;
use core::hash::Hasher;

use mcrs_minecraft_text::Text;

use crate::Filterable;
use crate::Generation;
use crate::entity_form::EntityForm;

pub(crate) const MAX_TITLE_UNITS: usize = 32;
pub(crate) const MAX_PAGE_UNITS: usize = 32767;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub struct WrittenBookContent {
    pub title: Filterable<String>,
    pub author: String,
    pub generation: Generation,
    pub pages: Cow<'static, [Filterable<Text>]>,
    pub resolved: bool,
}

impl WrittenBookContent {
    #[must_use]
    pub const fn new(title: Filterable<String>, author: String) -> Self {
        Self {
            title,
            author,
            generation: Generation::Original,
            pages: Cow::Borrowed(&[]),
            resolved: false,
        }
    }
}

impl Hash for WrittenBookContent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.author.hash(state);
        self.generation.hash(state);
        state.write_usize(self.pages.len());
        self.resolved.hash(state);
    }
}

impl EntityForm for WrittenBookContent {
    fn borrow_static(&'static self) -> Self {
        Self {
            title: self.title.clone(),
            author: self.author.clone(),
            generation: self.generation,
            pages: Cow::Borrowed(&self.pages),
            resolved: self.resolved,
        }
    }
}

pub(crate) fn is_title(title: &Filterable<String>) -> bool {
    fits_title(&title.raw) && title.filtered.as_deref().is_none_or(fits_title)
}

fn fits_title(text: &str) -> bool {
    text.encode_utf16().count() <= MAX_TITLE_UNITS
}

pub(crate) fn is_written_page(page: &Filterable<Text>) -> bool {
    fits_page(&page.raw) && page.filtered.as_ref().is_none_or(fits_page)
}

fn fits_page(text: &Text) -> bool {
    let mut units = Utf16Units(0);
    write!(units, "{text}").is_ok() && units.0 <= MAX_PAGE_UNITS
}

struct Utf16Units(usize);

impl fmt::Write for Utf16Units {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.0 = self.0.saturating_add(text.encode_utf16().count());
        Ok(())
    }
}
