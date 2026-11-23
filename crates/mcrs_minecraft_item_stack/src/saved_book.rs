use alloc::borrow::Cow;
use alloc::string::String;
use alloc::string::ToString;

use mcrs_minecraft_nbt::tag::NbtTag;

use crate::Filterable;
use crate::Generation;
use crate::NameLookup;
use crate::SavedError;
use crate::WritableBookContent;
use crate::WrittenBookContent;
use crate::saved::Reader;
use crate::saved_form::Fault;
use crate::saved_form::Walk;
use crate::saved_form::listed;
use crate::saved_sink::Field;
use crate::saved_sink::Leaf;
use crate::saved_sink::SavedSink;
use crate::saved_text::read_text;
use crate::writable_book_content::MAX_PAGES;
use crate::writable_book_content::is_writable_page;
use crate::written_book_content::is_title;
use crate::written_book_content::is_written_page;

const RAW: &str = "raw";
const FILTERED: &str = "filtered";
const GENERATIONS: core::ops::RangeInclusive<i32> = 0..=3;

impl<L: NameLookup> Reader<'_, L> {
    pub(crate) fn writable_book(&self, tag: &NbtTag) -> Result<WritableBookContent, SavedError> {
        let map = self.map(tag)?;
        let pages = self.listed(map, "pages", |reader, tag| {
            reader.filterable(tag, |reader, tag| Ok(reader.string(tag)?.to_string()))
        })?;
        if pages.len() > MAX_PAGES || !pages.iter().all(is_writable_page) {
            return Err(SavedError::OutOfRange(self.component));
        }
        Ok(WritableBookContent {
            pages: Cow::Owned(pages),
        })
    }

    pub(crate) fn written_book(&self, tag: &NbtTag) -> Result<WrittenBookContent, SavedError> {
        let map = self.map(tag)?;
        let title = self.filterable(self.necessary(map, "title")?, |reader, tag| {
            Ok(reader.string(tag)?.to_string())
        })?;
        if !is_title(&title) {
            return Err(SavedError::OutOfRange(self.component));
        }
        let author = self.string(self.necessary(map, "author")?)?.to_string();
        let generation = match map.get("generation") {
            Some(tag) => Generation::from_id(self.int(tag, GENERATIONS)?)
                .ok_or(SavedError::OutOfRange(self.component))?,
            None => Generation::Original,
        };
        let pages = self.listed(map, "pages", |reader, tag| {
            reader.filterable(tag, |reader, tag| read_text(tag, reader.component))
        })?;
        if !pages.iter().all(is_written_page) {
            return Err(SavedError::OutOfRange(self.component));
        }
        Ok(WrittenBookContent {
            title,
            author,
            generation,
            pages: Cow::Owned(pages),
            resolved: self.flag(map, "resolved", false)?,
        })
    }

    fn filterable<T>(
        &self,
        tag: &NbtTag,
        read: impl Fn(&Self, &NbtTag) -> Result<T, SavedError>,
    ) -> Result<Filterable<T>, SavedError> {
        if let NbtTag::Compound(map) = tag
            && let Some(raw) = map.get(RAW)
        {
            return Ok(Filterable {
                raw: read(self, raw)?,
                filtered: self.optional(map, FILTERED, &read)?,
            });
        }
        Ok(Filterable::new(read(self, tag)?))
    }
}

impl<L: NameLookup> Walk<'_, L> {
    pub(crate) fn writable_book<S: SavedSink>(
        &self,
        sink: &mut S,
        book: &WritableBookContent,
    ) -> Result<S::Node, Fault> {
        if book.pages.len() > MAX_PAGES || !book.pages.iter().all(is_writable_page) {
            return Err(Fault::OutOfRange(self.component));
        }
        let pages = listed(sink, &book.pages, |sink, page| {
            filterable(sink, page, |sink, text: &String| {
                Ok(sink.leaf(Leaf::String(text)))
            })
        })?;
        Ok(sink.fields([Field::optional("pages", pages)]))
    }

    pub(crate) fn written_book<S: SavedSink>(
        &self,
        sink: &mut S,
        book: &WrittenBookContent,
    ) -> Result<S::Node, Fault> {
        if !is_title(&book.title) || !book.pages.iter().all(is_written_page) {
            return Err(Fault::OutOfRange(self.component));
        }
        let title = filterable(sink, &book.title, |sink, text: &String| {
            Ok(sink.leaf(Leaf::String(text)))
        })?;
        let pages = listed(sink, &book.pages, |sink, page| {
            filterable(sink, page, |sink, text| self.text(sink, text))
        })?;
        let generation = (book.generation != Generation::Original)
            .then(|| sink.leaf(Leaf::Int(book.generation.id())));
        let resolved = book.resolved.then(|| sink.leaf(Leaf::Boolean(true)));
        Ok(sink.fields([
            Field::new("title", title),
            Field::new("author", sink.leaf(Leaf::String(&book.author))),
            Field::optional("generation", generation),
            Field::optional("pages", pages),
            Field::optional("resolved", resolved),
        ]))
    }
}

fn filterable<S: SavedSink, T>(
    sink: &mut S,
    value: &Filterable<T>,
    mut node: impl FnMut(&mut S, &T) -> Result<S::Node, Fault>,
) -> Result<S::Node, Fault> {
    let raw = node(sink, &value.raw)?;
    let filtered = value
        .filtered
        .as_ref()
        .map(|filtered| node(sink, filtered))
        .transpose()?;
    Ok(sink.fields([Field::new(RAW, raw), Field::optional(FILTERED, filtered)]))
}
