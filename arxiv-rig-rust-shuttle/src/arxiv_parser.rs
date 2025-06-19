use crate::error::ArxivError;
use crate::model::Paper;
use crate::util;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Reader;

#[derive(Default)]
pub struct ArxivParser<'a> {
    papers: Vec<Paper>,
    current_paper: Option<Paper>,
    current_authors: Vec<String>,
    current_categories: Vec<String>,
    in_entry: bool,
    current_field: Option<&'a str>,
}

impl<'a> ArxivParser<'a> {
    pub fn new() -> Self {
        Self {
            papers: Vec::new(),
            current_paper: None,
            current_authors: Vec::new(),
            current_categories: Vec::new(),
            in_entry: false,
            current_field: None,
        }
    }
    fn parse_start_event(&mut self, event: &BytesStart) {
        match event.name().as_ref() {
            // if the tag is "entry", this means we're at the start of a new xml block
            // so we can clear related variables and start anew
            b"entry" => {
                self.in_entry = true;
                self.current_paper = Some(Paper::new());
                self.current_authors.clear();
                self.current_categories.clear();
            }
            // otherwise, change the parsing state
            b"title" if self.in_entry => self.current_field = Some("title"),
            b"author" if self.in_entry => self.current_field = Some("author"),
            b"summary" if self.in_entry => self.current_field = Some("abstract"),
            b"link" if self.in_entry => self.current_field = Some("link"),
            b"category" if self.in_entry => self.current_field = Some("category"),
            _ => (),
        };
    }

    fn parse_text_event(&mut self, event: &BytesText) -> Result<(), ArxivError> {
        // if there's no current paper, just don't return anything
        let Some(paper) = self.current_paper.as_mut() else {
            return Ok(());
        };
        // otherwise, attempt to get the text and fill in the relevant field
        let text = str::from_utf8(event.as_ref())?.to_owned();
        match self.current_field {
            Some("title") => paper.title = text,
            Some("author") => self.current_authors.push(text),
            Some("abstract") => paper.abstract_text = text,
            _ => (),
        }
        Ok(())
    }

    fn parse_empty_event(&mut self, event: &BytesStart) -> Result<(), ArxivError> {
        // if we're not in an entry, just don't do anything
        if !self.in_entry {
            return Ok(());
        }
        // if the element is a link, convert the URL to the relevant format
        // and add the URL to the paper
        if event.name().as_ref() == b"link" {
            if let Some(paper) = self.current_paper.as_mut() {
                for attr in event.attributes().flatten() {
                    if attr.key.as_ref() == b"href" {
                        let url = str::from_utf8(&attr.value)?;
                        // Convert to HTTPS and ensure PDF URL
                        let secure_url = util::convert_pdf_url(url);
                        secure_url.clone_into(&mut paper.url);
                    }
                }
            }
        }
        // if the element is a Category, push the category terms
        // into the parser's list of current categories
        if event.name().as_ref() == b"category" {
            for attr in event.attributes().flatten() {
                if attr.key.as_ref() == b"term" {
                    self.current_categories
                        .push(str::from_utf8(&attr.value)?.to_owned());
                }
            }
        }

        Ok(())
    }

    fn parse_end_event(&mut self, event: &BytesEnd) -> Result<(), ArxivError> {
        // this is an end event - if the end tag is for an entry
        // add the current paper to the list of papers
        match event.name().as_ref() {
            b"entry" => {
                if let Some(mut paper) = self.current_paper.take() {
                    paper.authors.clone_from(&self.current_authors);
                    paper.categories.clone_from(&self.current_categories);
                    self.papers.push(paper);
                }
                self.in_entry = false;
            }
            // else, just change the currently parsed field to None
            // as there is now nothing to parse
            b"title" | b"author" | b"summary" | b"link" | b"category" => {
                self.current_field = None;
            }
            _ => (),
        }
        Ok(())
    }

    pub fn parse_response(&mut self, input: &str) -> Result<Vec<Paper>, ArxivError> {
        let mut reader = Reader::from_str(input);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => self.parse_start_event(e),
                Ok(Event::Text(ref e)) => self.parse_text_event(e)?,
                Ok(Event::Empty(ref e)) => self.parse_empty_event(e)?,
                Ok(Event::End(ref e)) => self.parse_end_event(e)?,
                // EoF means end of file - we can stop trying to parse here
                Ok(Event::Eof) => break,
                Err(e) => return Err(ArxivError::XmlParsing(e)),
                _ => (),
            }
        }

        if self.papers.is_empty() {
            return Err(ArxivError::NoResults);
        }

        Ok(self.papers.clone())
    }
}
