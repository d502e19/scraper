use crate::traits::Extractor;
use crate::task::Task;
use std::error::Error;
use scraper::{Html, Selector};
use std::marker::PhantomData;
use url::Url;

struct HTMLExtractorBase<D, H: HTMLExtractor<D>> {
    _marker: PhantomData<D>,
    html_extractor: H,
}

impl <D, H>Extractor<Vec<u8>, D> for HTMLExtractorBase<D, H> where H: HTMLExtractor<D> {
    fn extract_content(&self, content: Vec<u8>, url: Url) -> Result<(Vec<Task>, Vec<D>), Box<dyn Error>> {
        let html = String::from_utf8(content)?;
        let document = Html::parse_document(html.as_str());

        self.html_extractor.extract_from_html(document, url)
    }
}

impl <D, H: HTMLExtractor<D>>HTMLExtractorBase<D, H> {
    fn new(html_extractor: H) -> HTMLExtractorBase<D, H> {
        HTMLExtractorBase {
            _marker: PhantomData,
            html_extractor
        }
    }
}

pub trait HTMLExtractor<D> {
    fn extract_from_html(&self, content: Html, url: Url) -> Result<(Vec<Task>, Vec<D>), Box<dyn Error>>;
}

pub struct HTMLLinkExtractor {
    link_selector: Selector,
}

impl HTMLLinkExtractor {
    fn new() -> HTMLLinkExtractor {
        HTMLLinkExtractor {
            link_selector: Selector::parse("a").expect("anchor tag selector")
        }
    }
}

impl HTMLExtractor<()> for HTMLLinkExtractor {
    fn extract_from_html(&self, content: Html, reference_url: Url) -> Result<(Vec<Task>, Vec<()>), Box<dyn Error>> {
        let tasks: Vec<Task> = content.select(&self.link_selector)
            .filter_map(|element| {
                element.value().attr("href")
            })
            .filter_map(|url| {
                Url::options()
                    .base_url(Some(&reference_url))
                    .parse(url)
                    .ok()
            })
            .map(|url| {
                Task {
                    url
                }
            })
            .collect();

        Ok((tasks, vec!()))
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

        let result = extractor.extract_content(test_string.as_bytes().to_vec(), url);

        match result {
            Ok((tasks, _)) => {
                assert_eq!(tasks.len(), 1);
                assert_eq!(tasks[0].url.as_str(), "http://example.com/");
            },
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

        let result = extractor.extract_content(test_string.as_bytes().to_vec(), url);

        match result {
            Ok((tasks, _)) => {
                assert_eq!(tasks.len(), 1);
                assert_eq!(tasks[0].url.as_str(), "http://ref.ref/test");
            },
            Err(_) => panic!(),
        }
    }
}
