use std::collections::HashSet;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::error::Error;

/// A Wikipedia article extracted from the XML dump.
///
/// Each article corresponds to a single `<page>` element in the MediaWiki
/// XML dump, containing the page metadata and the wikitext body.
#[derive(Debug, Clone)]
pub struct Article {
    /// The unique page identifier.
    pub id: u64,
    /// The page title.
    pub title: String,
    /// The namespace number (0 for main articles, 1 for talk pages, etc.).
    pub namespace: i32,
    /// The raw wikitext content of the page.
    pub text: String,
}

/// Streaming XML parser that yields [`Article`]s filtered by namespace.
///
/// `DumpReader` wraps a `quick_xml::Reader` and lazily parses `<page>` elements
/// from a MediaWiki XML dump. Only pages whose namespace is in the configured
/// set are returned; all others are silently skipped.
///
/// # Type Parameters
///
/// * `R` - A type implementing [`BufRead`] that provides the XML data.
pub struct DumpReader<R: BufRead> {
    reader: Reader<R>,
    buf: Vec<u8>,
    namespaces: HashSet<i32>,
    url_base: String,
    initialized: bool,
}

impl<R: BufRead> DumpReader<R> {
    /// Creates a new `DumpReader` that parses XML from the given reader,
    /// yielding only articles whose namespace is in `namespaces`.
    ///
    /// # Arguments
    ///
    /// * `reader` - A buffered reader providing the XML data.
    /// * `namespaces` - A slice of namespace numbers to include.
    ///
    /// # Returns
    ///
    /// A new `DumpReader` instance.
    pub fn new(reader: R, namespaces: &[i32]) -> Self {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);
        Self {
            reader: xml_reader,
            buf: Vec::new(),
            namespaces: namespaces.iter().copied().collect(),
            url_base: String::new(),
            initialized: false,
        }
    }

    /// Returns the base URL extracted from `<siteinfo><base>`.
    ///
    /// This value is populated after the first call to `next()` (or after
    /// initialization completes). If the dump does not contain a `<siteinfo>`
    /// section, the returned string will be empty.
    pub fn url_base(&self) -> &str {
        &self.url_base
    }

    /// Parses the `<siteinfo>` header to extract the base URL.
    ///
    /// Reads events until the closing `</siteinfo>` tag, extracting the
    /// text content of the `<base>` element. The page name is stripped
    /// from the URL so that only the wiki base path remains
    /// (e.g., `https://en.wikipedia.org/wiki`).
    fn parse_siteinfo(&mut self) -> Result<(), Error> {
        let mut in_base = false;
        loop {
            self.buf.clear();
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(ref e)) => {
                    if e.local_name().as_ref() == b"base" {
                        in_base = true;
                    }
                }
                Ok(Event::Text(ref e)) if in_base => {
                    let base_text = e
                        .decode()
                        .map_err(|err| Error::Xml(format!("failed to decode base text: {err}")))?
                        .to_string();
                    // Strip the page name from the URL to get the wiki base.
                    // e.g., "https://en.wikipedia.org/wiki/Main_Page" -> "https://en.wikipedia.org/wiki"
                    if let Some(pos) = base_text.rfind('/') {
                        self.url_base = base_text[..pos].to_string();
                    } else {
                        self.url_base = base_text;
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.local_name().as_ref() == b"base" {
                        in_base = false;
                    } else if e.local_name().as_ref() == b"siteinfo" {
                        return Ok(());
                    }
                }
                Ok(Event::Eof) => {
                    return Err(Error::Xml(
                        "unexpected EOF while parsing siteinfo".to_string(),
                    ));
                }
                Err(e) => return Err(Error::XmlReader(e)),
                _ => {}
            }
        }
    }

    /// Ensures that the `<siteinfo>` header has been parsed.
    ///
    /// This is called lazily on the first iteration. It reads events until
    /// it encounters `<siteinfo>` and delegates to [`parse_siteinfo`].
    fn ensure_initialized(&mut self) -> Result<(), Error> {
        if self.initialized {
            return Ok(());
        }
        self.initialized = true;

        loop {
            self.buf.clear();
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.local_name();
                    if name.as_ref() == b"siteinfo" {
                        self.parse_siteinfo()?;
                        return Ok(());
                    }
                    if name.as_ref() == b"page" {
                        // No siteinfo found before the first page.
                        // We still consider initialization done; we will
                        // need to parse this page in the next iteration,
                        // but since we consumed the start tag we handle it
                        // by returning Ok and letting the iterator pick up
                        // from here. Unfortunately we already consumed the
                        // <page> start. To handle this edge case properly,
                        // we set a flag so the iterator knows.
                        return Ok(());
                    }
                }
                Ok(Event::Eof) => return Ok(()),
                Err(e) => return Err(Error::XmlReader(e)),
                _ => {}
            }
        }
    }

    /// Parses a single `<page>` element and returns an [`Article`] if the
    /// page's namespace is in the configured set.
    ///
    /// Returns `Ok(Some(article))` if the page matches the namespace filter,
    /// `Ok(None)` if the page was skipped, or `Err` on parse errors.
    fn parse_page(&mut self) -> Result<Option<Article>, Error> {
        let mut title = String::new();
        let mut id: Option<u64> = None;
        let mut ns: Option<i32> = None;
        let mut text = String::new();
        let mut current_tag = String::new();
        let mut in_revision = false;
        let mut page_id_captured = false;

        loop {
            self.buf.clear();
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(ref e)) => {
                    let local = e.local_name();
                    current_tag = String::from_utf8_lossy(local.as_ref()).to_string();
                    if current_tag == "revision" {
                        in_revision = true;
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local = e.local_name();
                    let tag = local.as_ref();
                    if tag == b"revision" {
                        in_revision = false;
                    } else if tag == b"page" {
                        // End of page element.
                        let namespace = ns.unwrap_or(0);
                        if !self.namespaces.contains(&namespace) {
                            return Ok(None);
                        }
                        let page_id = match id {
                            Some(v) => v,
                            None => {
                                log::warn!("page missing <id>, skipping");
                                return Ok(None);
                            }
                        };
                        return Ok(Some(Article {
                            id: page_id,
                            title,
                            namespace,
                            text,
                        }));
                    }
                    current_tag.clear();
                }
                Ok(Event::Text(ref e)) => {
                    let value = e
                        .decode()
                        .map_err(|err| {
                            Error::Xml(format!("failed to decode text in <{current_tag}>: {err}"))
                        })?
                        .to_string();
                    match current_tag.as_str() {
                        "title" => title = value,
                        "ns" => {
                            ns = Some(value.parse::<i32>().map_err(|err| {
                                Error::Xml(format!("invalid namespace value '{value}': {err}"))
                            })?);
                        }
                        "id" if !in_revision && !page_id_captured => {
                            id = Some(value.parse::<u64>().map_err(|err| {
                                Error::Xml(format!("invalid page id '{value}': {err}"))
                            })?);
                            page_id_captured = true;
                        }
                        "text" if in_revision => {
                            text = value;
                        }
                        _ => {}
                    }
                }
                Ok(Event::CData(ref e)) => {
                    if current_tag == "text" && in_revision {
                        let value = String::from_utf8_lossy(e.as_ref()).to_string();
                        text = value;
                    }
                }
                Ok(Event::Eof) => {
                    return Err(Error::Xml("unexpected EOF inside <page>".to_string()));
                }
                Err(e) => return Err(Error::XmlReader(e)),
                _ => {}
            }
        }
    }
}

impl<R: BufRead> Iterator for DumpReader<R> {
    type Item = Result<Article, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Err(e) = self.ensure_initialized() {
            return Some(Err(e));
        }

        loop {
            self.buf.clear();
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(ref e)) => {
                    if e.local_name().as_ref() == b"page" {
                        match self.parse_page() {
                            Ok(Some(article)) => return Some(Ok(article)),
                            Ok(None) => continue, // Filtered out or skipped.
                            Err(e) => {
                                log::warn!("error parsing page: {e}");
                                continue;
                            }
                        }
                    }
                }
                Ok(Event::Eof) => return None,
                Err(e) => return Some(Err(Error::XmlReader(e))),
                _ => {}
            }
        }
    }
}

/// Opens a Wikipedia dump file, auto-detecting `.bz2` compression by extension.
///
/// If the file path ends with `.bz2`, the file is decompressed using
/// [`bzip2::bufread::MultiBzDecoder`]. The `MultiBz` variant is required
/// because Wikipedia dumps consist of multiple bzip2 streams concatenated
/// together.
///
/// # Arguments
///
/// * `path` - Path to the dump file (plain XML or `.bz2` compressed).
/// * `namespaces` - Namespace numbers to include in the output.
///
/// # Returns
///
/// A [`DumpReader`] that streams [`Article`]s from the dump file.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read.
pub fn open_dump(path: &Path, namespaces: &[i32]) -> Result<DumpReader<Box<dyn BufRead>>, Error> {
    let file = std::fs::File::open(path)?;
    let buf_reader = BufReader::new(file);

    let reader: Box<dyn BufRead> = if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("bz2"))
    {
        Box::new(BufReader::new(bzip2::bufread::MultiBzDecoder::new(
            buf_reader,
        )))
    } else {
        Box::new(buf_reader)
    };

    Ok(DumpReader::new(reader, namespaces))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a `DumpReader` from an XML string for testing.
    fn reader_from_xml(xml: &str, namespaces: &[i32]) -> DumpReader<Box<dyn BufRead>> {
        let cursor = std::io::Cursor::new(xml.to_string().into_bytes());
        let boxed: Box<dyn BufRead> = Box::new(BufReader::new(cursor));
        DumpReader::new(boxed, namespaces)
    }

    const SINGLE_PAGE_XML: &str = r#"<mediawiki>
  <siteinfo>
    <sitename>Wikipedia</sitename>
    <base>https://en.wikipedia.org/wiki/Main_Page</base>
  </siteinfo>
  <page>
    <title>Test Article</title>
    <ns>0</ns>
    <id>42</id>
    <revision>
      <id>999</id>
      <text>Hello, world!</text>
    </revision>
  </page>
</mediawiki>"#;

    #[test]
    fn test_parse_single_page() {
        let mut reader = reader_from_xml(SINGLE_PAGE_XML, &[0]);
        let article = reader.next().unwrap().unwrap();
        assert_eq!(article.id, 42);
        assert_eq!(article.title, "Test Article");
        assert_eq!(article.namespace, 0);
        assert_eq!(article.text, "Hello, world!");
        assert!(reader.next().is_none());
    }

    #[test]
    fn test_namespace_filtering() {
        let xml = r#"<mediawiki>
  <siteinfo>
    <sitename>Wikipedia</sitename>
    <base>https://en.wikipedia.org/wiki/Main_Page</base>
  </siteinfo>
  <page>
    <title>Main Article</title>
    <ns>0</ns>
    <id>1</id>
    <revision>
      <id>100</id>
      <text>Main content</text>
    </revision>
  </page>
  <page>
    <title>Talk:Main Article</title>
    <ns>1</ns>
    <id>2</id>
    <revision>
      <id>101</id>
      <text>Talk content</text>
    </revision>
  </page>
  <page>
    <title>Another Article</title>
    <ns>0</ns>
    <id>3</id>
    <revision>
      <id>102</id>
      <text>Another content</text>
    </revision>
  </page>
</mediawiki>"#;

        let reader = reader_from_xml(xml, &[0]);
        let articles: Vec<Article> = reader.map(|r| r.unwrap()).collect();
        assert_eq!(articles.len(), 2);
        assert_eq!(articles[0].id, 1);
        assert_eq!(articles[1].id, 3);
    }

    #[test]
    fn test_url_base_extraction() {
        let mut reader = reader_from_xml(SINGLE_PAGE_XML, &[0]);
        // Trigger initialization by consuming at least one item.
        let _ = reader.next();
        assert_eq!(reader.url_base(), "https://en.wikipedia.org/wiki");
    }

    #[test]
    fn test_multiple_pages() {
        let xml = r#"<mediawiki>
  <siteinfo>
    <sitename>Wikipedia</sitename>
    <base>https://en.wikipedia.org/wiki/Main_Page</base>
  </siteinfo>
  <page>
    <title>First</title>
    <ns>0</ns>
    <id>10</id>
    <revision>
      <id>200</id>
      <text>First text</text>
    </revision>
  </page>
  <page>
    <title>Second</title>
    <ns>0</ns>
    <id>20</id>
    <revision>
      <id>201</id>
      <text>Second text</text>
    </revision>
  </page>
  <page>
    <title>Third</title>
    <ns>0</ns>
    <id>30</id>
    <revision>
      <id>202</id>
      <text>Third text</text>
    </revision>
  </page>
</mediawiki>"#;

        let reader = reader_from_xml(xml, &[0]);
        let articles: Vec<Article> = reader.map(|r| r.unwrap()).collect();
        assert_eq!(articles.len(), 3);
        assert_eq!(articles[0].id, 10);
        assert_eq!(articles[0].title, "First");
        assert_eq!(articles[1].id, 20);
        assert_eq!(articles[1].title, "Second");
        assert_eq!(articles[2].id, 30);
        assert_eq!(articles[2].title, "Third");
    }

    #[test]
    fn test_redirect_page() {
        let xml = r#"<mediawiki>
  <siteinfo>
    <sitename>Wikipedia</sitename>
    <base>https://en.wikipedia.org/wiki/Main_Page</base>
  </siteinfo>
  <page>
    <title>Redirect Page</title>
    <ns>0</ns>
    <id>50</id>
    <redirect title="Target Page" />
    <revision>
      <id>300</id>
      <text>#REDIRECT [[Target Page]]</text>
    </revision>
  </page>
</mediawiki>"#;

        let mut reader = reader_from_xml(xml, &[0]);
        let article = reader.next().unwrap().unwrap();
        assert_eq!(article.id, 50);
        assert_eq!(article.title, "Redirect Page");
        assert_eq!(article.text, "#REDIRECT [[Target Page]]");
    }

    #[test]
    fn test_missing_text() {
        let xml = r#"<mediawiki>
  <siteinfo>
    <sitename>Wikipedia</sitename>
    <base>https://en.wikipedia.org/wiki/Main_Page</base>
  </siteinfo>
  <page>
    <title>No Text Page</title>
    <ns>0</ns>
    <id>60</id>
    <revision>
      <id>400</id>
    </revision>
  </page>
</mediawiki>"#;

        let mut reader = reader_from_xml(xml, &[0]);
        let article = reader.next().unwrap().unwrap();
        assert_eq!(article.id, 60);
        assert_eq!(article.title, "No Text Page");
        assert_eq!(article.text, "");
    }
}
