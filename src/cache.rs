use std::{
    env,
    ffi::OsStr,
    fs::{self, File},
    io::{BufReader, Cursor, Read},
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use anyhow::{ensure, Context, Result};
use log::debug;
use reqwest::{blocking::Client, Proxy};
use walkdir::{DirEntry, WalkDir};
use zip::ZipArchive;

use crate::{config::TlsBackend, types::PlatformType, utils::print_warning};

pub static TLDR_PAGES_DIR: &str = "tldr-pages";
static TLDR_OLD_PAGES_DIR: &str = "tldr-master";

#[derive(Debug, PartialEq, Eq)]
pub struct Language<'a>(pub(crate) &'a str);

pub struct CacheConfig<'a> {
    pub pages_directory: &'a Path,
    pub custom_pages_directory: Option<&'a Path>,
    pub platforms: &'a [PlatformType],
    pub languages: &'a [Language<'a>],
}

/// The directory backing this cache is checked to be populated at construction.
pub struct Cache<'a> {
    config: CacheConfig<'a>,
}

#[derive(Debug)]
pub struct PageLookupResult {
    pub page_path: PathBuf,
    pub patch_path: Option<PathBuf>,
}

impl<'a> Cache<'a> {
    pub fn open(config: CacheConfig<'a>) -> Result<Option<Self>> {
        todo!()
    }

    pub fn open_or_create(config: CacheConfig<'a>) -> Result<Self> {
        todo!()
    }

    pub fn age(&self) -> Result<Duration> {
        todo!()
    }

    pub fn list_pages(&self) -> impl IntoIterator<Item = String> {
        []
    }

    pub fn find_page(&self, command: &str) -> Option<PageLookupResult> {
        todo!()
    }

    pub fn clear(self) -> Result<()> {
        todo!()
    }

    pub fn update(&mut self, archive_url: &str) -> Result<()> {
        todo!()
    }

    fn build_client(tls_backend: TlsBackend) -> Result<reqwest::Client> {
        todo!()
    }

    pub fn config(&self) -> &CacheConfig<'a> {
        &self.config
    }
}

impl PageLookupResult {
    pub fn with_page(page_path: PathBuf) -> Self {
        Self {
            page_path,
            patch_path: None,
        }
    }

    pub fn with_optional_patch(mut self, patch_path: Option<PathBuf>) -> Self {
        self.patch_path = patch_path;
        self
    }

    /// Create a buffered reader that sequentially reads from the page and the
    /// patch, as if they were concatenated.
    ///
    /// This will return an error if either the page file or the patch file
    /// cannot be opened.
    pub fn reader(&self) -> Result<BufReader<Box<dyn Read>>> {
        // Open page file
        let page_file = File::open(&self.page_path)
            .with_context(|| format!("Could not open page file at {}", self.page_path.display()))?;

        // Open patch file
        let patch_file_opt = match &self.patch_path {
            Some(path) => Some(
                File::open(path)
                    .with_context(|| format!("Could not open patch file at {}", path.display()))?,
            ),
            None => None,
        };

        // Create chained reader from file(s)
        //
        // Note: It might be worthwhile to create our own struct that accepts
        // the page and patch files and that will read them sequentially,
        // because it avoids the boxing below. However, the performance impact
        // would first need to be shown to be significant using a benchmark.
        Ok(BufReader::new(if let Some(patch_file) = patch_file_opt {
            Box::new(page_file.chain(&b"\n"[..]).chain(patch_file)) as Box<dyn Read>
        } else {
            Box::new(page_file) as Box<dyn Read>
        }))
    }
}

pub enum CacheFreshness {
    /// The cache is still fresh (less than `MAX_CACHE_AGE` old)
    Fresh,
    /// The cache is stale and should be updated
    Stale(Duration),
    /// The cache is missing
    Missing,
}

/// Unit Tests for cache module
#[cfg(test)]
mod tests {
    use super::*;

    use std::{
        fs::File,
        io::{Read, Write},
    };

    #[test]
    fn test_reader_with_patch() {
        // Write test files
        let dir = tempfile::tempdir().unwrap();
        let page_path = dir.path().join("test.page.md");
        let patch_path = dir.path().join("test.patch.md");
        {
            let mut f1 = File::create(&page_path).unwrap();
            f1.write_all(b"Hello\n").unwrap();
            let mut f2 = File::create(&patch_path).unwrap();
            f2.write_all(b"World").unwrap();
        }

        // Create chained reader from lookup result
        let lr = PageLookupResult::with_page(page_path).with_optional_patch(Some(patch_path));
        let mut reader = lr.reader().unwrap();

        // Read into a Vec
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).unwrap();

        assert_eq!(&buf, b"Hello\n\nWorld");
    }

    #[test]
    fn test_reader_without_patch() {
        // Write test file
        let dir = tempfile::tempdir().unwrap();
        let page_path = dir.path().join("test.page.md");
        {
            let mut f = File::create(&page_path).unwrap();
            f.write_all(b"Hello\n").unwrap();
        }

        // Create chained reader from lookup result
        let lr = PageLookupResult::with_page(page_path);
        let mut reader = lr.reader().unwrap();

        // Read into a Vec
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).unwrap();

        assert_eq!(&buf, b"Hello\n");
    }

    #[test]
    #[cfg(feature = "native-tls")]
    fn test_create_https_client_with_native_tls() {
        Cache::build_client(TlsBackend::NativeTls).expect("fails to build a client.");
    }

    #[test]
    #[cfg(feature = "rustls-with-webpki-roots")]
    fn test_create_https_client_with_rustls() {
        Cache::build_client(TlsBackend::RustlsWithWebpkiRoots).expect("fails to build a client.");
    }

    #[test]
    #[cfg(feature = "rustls-with-native-roots")]
    fn test_create_https_client_with_rustls_with_native_roots() {
        Cache::build_client(TlsBackend::RustlsWithNativeRoots).expect("fails to build a client.");
    }
}
