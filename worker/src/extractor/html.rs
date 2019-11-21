use std::marker::PhantomData;

use scraper::{Html, Selector};
use url::Url;

use crate::errors::{ExtractError, ExtractResult};
use crate::errors::ExtractErrorKind::ParsingError;
use crate::traits::Extractor;

/// The HTMLExtractorBase is an Extractor that converts a page of bytes (u8) to HTML and
/// uses a HTMLExtractor to extract target data.
pub struct HTMLExtractorBase<D, H: HTMLExtractor<D>> {
    _marker: PhantomData<D>,
    html_extractor: H,
}

impl<D, H> Extractor<Vec<u8>, D> for HTMLExtractorBase<D, H>
where
    H: HTMLExtractor<D>,
{
    fn extract_content(&self, content: Vec<u8>, url: &Url) -> ExtractResult<(Vec<Url>, Vec<D>)> {
        let html = String::from_utf8(content).map_err(|e| {
            ExtractError::new(ParsingError, "Failed to parse html", Some(Box::new(e)))
        })?;
        let document = Html::parse_document(html.as_str());

        self.html_extractor.extract_from_html(document, url)
    }
}

impl<D, H: HTMLExtractor<D>> HTMLExtractorBase<D, H> {
    /// Construct a new HTMLExtractorBase that extracts Urls and target data with the given
    /// HTMLExtractor
    pub fn new(html_extractor: H) -> HTMLExtractorBase<D, H> {
        HTMLExtractorBase {
            _marker: PhantomData,
            html_extractor,
        }
    }
}

/// An HTMLExtractor extracts Urls and data from HTML.
pub trait HTMLExtractor<D> {
    fn extract_from_html(&self, content: Html, url: &Url) -> ExtractResult<(Vec<Url>, Vec<D>)>;
}

/// The HTMLLinkExtractor is a HTMLExtractor that only extracts links and no data. It finds
/// Urls by taking the href attributes of the anchor tags. 
pub struct HTMLLinkExtractor {
    link_selector: Selector,
}

impl HTMLLinkExtractor {
    /// Construct a new HTMLLinkExtractor
    pub fn new() -> HTMLLinkExtractor {
        HTMLLinkExtractor {
            link_selector: Selector::parse("a").expect("anchor tag selector"),
        }
    }
}

impl HTMLExtractor<()> for HTMLLinkExtractor {
    fn extract_from_html(
        &self,
        content: Html,
        reference_url: &Url,
    ) -> ExtractResult<(Vec<Url>, Vec<()>)> {
        // Extract no data
        // Urls are found in the href attributes of anchor tags
        // and check if they are either https or http
        let tasks: Vec<Url> = content
            .select(&self.link_selector)
            .filter_map(|element| element.value().attr("href"))
            .filter_map(|url| {
                Url::options()
                    .base_url(Some(&reference_url))
                    .parse(url)
                    .ok()
            })
            .filter_map(|url| {
                if "https" == url.scheme() || url.scheme() == "http" {
                    Some(url)
                } else {
                    None
                }
            })
            .collect();

        Ok((tasks, vec![]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_extractor() {
        let html_extractor = HTMLLinkExtractor::new();
        let extractor = HTMLExtractorBase::new(html_extractor);

        let test_string = "<!DOCTYPE html>
            <html>
            <body>
            <a>one</a>
            <a href=\"http://example.com/\">two</a>
            </body>
            </html>";
        let url = Url::parse("http://ref.ref").unwrap();

        let result = extractor.extract_content(test_string.as_bytes().to_vec(), &url);

        match result {
            Ok((urls, _)) => {
                assert_eq!(urls.len(), 1);
                assert_eq!(urls[0].as_str(), "http://example.com/");
            }
            Err(_) => panic!(),
        }
    }

    #[test]
    fn test_link_extractor_ref() {
        let html_extractor = HTMLLinkExtractor::new();
        let extractor = HTMLExtractorBase::new(html_extractor);

        let test_string = "<!DOCTYPE html>
            <html>
            <body>
            <a href=\"/test\">two</a>
            </body>
            </html>";
        let url = Url::parse("http://ref.ref").unwrap();

        let result = extractor.extract_content(test_string.as_bytes().to_vec(), &url);

        match result {
            Ok((url, _)) => {
                assert_eq!(url.len(), 1);
                assert_eq!(url[0].as_str(), "http://ref.ref/test");
            }
            Err(_) => panic!(),
        }
    }

    #[test]
    fn test_link_extractor_http_only() {
        let html_extractor = HTMLLinkExtractor::new();
        let extractor = HTMLExtractorBase::new(html_extractor);

        let test_string = "<!DOCTYPE html>
            <html>
            <body>
            <a>one</a>
            <a href=\"http://example.com/\">two</a>
            <a href=\"mailto:example.com/\">two</a>
            <a href=\"urn:example.com/\">two</a>
            </body>
            </html>";
        let url = Url::parse("http://ref.ref").unwrap();

        let result = extractor.extract_content(test_string.as_bytes().to_vec(), &url);

        match result {
            Ok((urls, _)) => {
                assert_eq!(urls.len(), 1);
                assert_eq!(urls[0].as_str(), "http://example.com/");
            }
            Err(_) => panic!(),
        }
    }
}
