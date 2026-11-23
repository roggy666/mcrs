use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;

use mcrs_minecraft_wire::Bounded;
use mcrs_minecraft_wire::Decode;
use mcrs_minecraft_wire::Encode;
use mcrs_minecraft_wire::ErrorKind;

use crate::Filterable;
use crate::Generation;
use crate::NameLookup;
use crate::WireError;
use crate::WritableBookContent;
use crate::WrittenBookContent;
use crate::wire_text::read_text;
use crate::wire_text::write_text;
use crate::wire_value::Reader;
use crate::wire_value::Writer;
use crate::wire_value::put_boolean;
use crate::wire_value::put_var_int;
use crate::writable_book_content::MAX_PAGE_UNITS;
use crate::writable_book_content::MAX_PAGES;
use crate::writable_book_content::is_writable_page;
use crate::written_book_content::MAX_TITLE_UNITS;
use crate::written_book_content::is_written_page;

impl<L: NameLookup> Reader<'_, L> {
    pub(crate) fn writable_book(
        &self,
        input: &mut &[u8],
    ) -> Result<WritableBookContent, WireError> {
        let pages = self.limited_list(input, MAX_PAGES, |reader, input| {
            reader.filterable(input, Reader::bounded_string::<MAX_PAGE_UNITS>)
        })?;
        Ok(WritableBookContent {
            pages: Cow::Owned(pages),
        })
    }

    pub(crate) fn written_book(&self, input: &mut &[u8]) -> Result<WrittenBookContent, WireError> {
        let title = self.filterable(input, Reader::bounded_string::<MAX_TITLE_UNITS>)?;
        let author = self.string(input)?;
        let generation = Generation::from_id(self.var_int(input)?)
            .ok_or(WireError::OutOfRange(self.component))?;
        let pages = self.list(input, |reader, input| {
            reader.filterable(input, |reader, input| read_text(input, reader.component))
        })?;
        Ok(WrittenBookContent {
            title,
            author,
            generation,
            pages: Cow::Owned(pages),
            resolved: self.boolean(input)?,
        })
    }

    fn filterable<T>(
        &self,
        input: &mut &[u8],
        read: impl Fn(&Self, &mut &[u8]) -> Result<T, WireError>,
    ) -> Result<Filterable<T>, WireError> {
        let raw = read(self, input)?;
        let filtered = self.optional(input, &read)?;
        Ok(Filterable { raw, filtered })
    }

    fn bounded_string<const MAX_CHARS: usize>(
        &self,
        input: &mut &[u8],
    ) -> Result<String, WireError> {
        match Bounded::<&str, MAX_CHARS>::decode(input) {
            Ok(text) => Ok(text.0.into()),
            Err(error) => Err(match error.kind() {
                ErrorKind::UnexpectedEnd => WireError::UnexpectedEnd,
                ErrorKind::TooLong { .. } => WireError::OutOfRange(self.component),
                _ => self.bad(),
            }),
        }
    }
}

impl<L: NameLookup> Writer<'_, L> {
    pub(crate) fn writable_book(
        &self,
        book: &WritableBookContent,
        output: &mut Vec<u8>,
    ) -> Result<(), WireError> {
        if !book.pages.iter().all(is_writable_page) {
            return Err(WireError::OutOfRange(self.component));
        }
        self.limited_list(&book.pages, MAX_PAGES, output, |writer, page, output| {
            writer.filterable(page, output, |writer, text, output| {
                writer.bounded_string::<MAX_PAGE_UNITS>(text, output)
            })
        })
    }

    pub(crate) fn written_book(
        &self,
        book: &WrittenBookContent,
        output: &mut Vec<u8>,
    ) -> Result<(), WireError> {
        if !book.pages.iter().all(is_written_page) {
            return Err(WireError::OutOfRange(self.component));
        }
        self.filterable(&book.title, output, |writer, text, output| {
            writer.bounded_string::<MAX_TITLE_UNITS>(text, output)
        })?;
        self.string(&book.author, output)?;
        put_var_int(output, book.generation.id());
        self.list(&book.pages, output, |writer, page, output| {
            writer.filterable(page, output, |writer, text, output| {
                write_text(text, writer.component, output)
            })
        })?;
        put_boolean(output, book.resolved);
        Ok(())
    }

    fn filterable<T>(
        &self,
        value: &Filterable<T>,
        output: &mut Vec<u8>,
        write: impl Fn(&Self, &T, &mut Vec<u8>) -> Result<(), WireError>,
    ) -> Result<(), WireError> {
        write(self, &value.raw, output)?;
        self.optional(value.filtered.as_ref(), output, write)
    }

    fn bounded_string<const MAX_CHARS: usize>(
        &self,
        text: &str,
        output: &mut Vec<u8>,
    ) -> Result<(), WireError> {
        Bounded::<&str, MAX_CHARS>(text)
            .encode(&mut *output)
            .map_err(|_| WireError::OutOfRange(self.component))
    }
}
