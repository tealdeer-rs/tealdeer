use std::{
    env,
    ffi::OsStr,
    fs::{self, File},
    io::{BufReader, Cursor, Read},
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use anyhow::{anyhow, ensure, Context, Result};
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
        let (cache_dir_exists, cache_dir_is_dir) = config
            .pages_directory
            .metadata()
            .map_or((false, false), |md| (true, md.is_dir()));
        ensure!(
            !cache_dir_exists || cache_dir_is_dir,
            "{} exists, but is not a directory.",
            config.pages_directory.display(),
        );

        Ok(cache_dir_is_dir.then_some(Cache { config }))
    }

    pub fn open_or_create(config: CacheConfig<'a>) -> Result<Self> {
        let (cache_dir_exists, cache_dir_is_dir) = config
            .pages_directory
            .metadata()
            .map_or((false, false), |md| (true, md.is_dir()));
        ensure!(
            !cache_dir_exists || cache_dir_is_dir,
            "{} exists, but is not a directory.",
            config.pages_directory.display(),
        );

        if !cache_dir_is_dir {
            fs::create_dir_all(&config.pages_directory).with_context(|| {
                format!(
                    "Cache directory `{}` cannot be created",
                    config.pages_directory.display(),
                )
            })?;
            eprintln!(
                "Successfully created cache directory `{}`.",
                config.pages_directory.display(),
            );
        }

        Ok(Cache { config })
    }

    pub fn age(&self) -> Result<Duration> {
        let mtime = self.config.pages_directory.metadata()?.modified()?;
        SystemTime::now()
            .duration_since(mtime)
            .context("Error comparing cache mtime with current time")
    }

    pub fn find_page(&self, command: &str) -> Option<PageLookupResult> {
        let page_filename = format!("{command}.md");
        let patch_filename = format!("{command}.patch.md");
        let custom_filename = format!("{command}.page.md");

        if let Some(custom_pages_dir) = self.config.custom_pages_directory {
            let custom_page = custom_pages_dir.join(custom_filename);
            if custom_page.is_file() {
                return Some(PageLookupResult::with_page(custom_page));
            }
        }

        let patch_path = self
            .config
            .custom_pages_directory
            .map(|dir| dir.join(&patch_filename))
            .filter(|path| path.is_file());

        let mut search_path = self.config.pages_directory.to_path_buf();
        for &platform in self.config.platforms {
            for language in self.config.languages {
                search_path.push(language.directory_name());
                search_path.push(platform.directory_name());
                search_path.push(&page_filename);

                if search_path.is_file() {
                    return Some(
                        PageLookupResult::with_page(search_path).with_optional_patch(patch_path),
                    );
                }

                search_path.pop();
                search_path.pop();
                search_path.pop();
            }
        }

        None
    }

    pub fn list_pages(&self) -> Result<impl IntoIterator<Item = String>> {
        let mut pages = Vec::new();

        let mut append_all = |directory: &Path, suffix: &str| -> Result<()> {
            let Ok(file_iter) = fs::read_dir(&directory) else {
                return Ok(());
            };

            for entry in file_iter {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    let mut page_path = entry
                        .file_name()
                        .into_string()
                        .map_err(|_| anyhow!("Found invalid filename: {:?}", entry.path()))?;

                    if page_path.ends_with(suffix) {
                        page_path.truncate(page_path.len() - suffix.len());
                        pages.push(page_path);
                    } else {
                        debug!(
                            "Skipping page entry not ending in \".md\": {:?}",
                            entry.path(),
                        );
                    }
                }
            }

            Ok(())
        };

        let mut search_path = self.config.pages_directory.to_path_buf();
        for language in self.config.languages {
            search_path.push(language.directory_name());
            for platform in self.config.platforms {
                search_path.push(platform.directory_name());
                append_all(&search_path, ".md")?;
                search_path.pop();
            }
            search_path.pop();
        }

        if let Some(custom_pages_dir) = self.config.custom_pages_directory {
            append_all(&custom_pages_dir, ".page.md")?;
        }

        pages.sort_unstable();
        pages.dedup();
        Ok(pages)
    }

    pub fn check_for_old_custom_pages(&self) -> Result<bool> {
        // TODO
        Ok(false)
    }

    pub fn clear(self) -> Result<()> {
        fs::remove_dir_all(self.config.pages_directory).with_context(|| {
            format!(
                "Could not remove pages directory at {}",
                self.config.pages_directory.display(),
            )
        })
    }

    pub fn update(&mut self, archive_url: &str) -> Result<()> {
        todo!()
    }

    fn build_client(tls_backend: TlsBackend) -> Result<reqwest::blocking::Client> {
        let mut builder = Client::builder();
        builder = match tls_backend {
            #[cfg(feature = "native-tls")]
            TlsBackend::NativeTls => builder
                .use_native_tls()
                .tls_built_in_root_certs(true)
                .tls_built_in_webpki_certs(false)
                .tls_built_in_native_certs(false),
            #[cfg(feature = "rustls-with-webpki-roots")]
            TlsBackend::RustlsWithWebpkiRoots => builder
                .use_rustls_tls()
                .tls_built_in_root_certs(false)
                .tls_built_in_webpki_certs(true)
                .tls_built_in_native_certs(false),
            #[cfg(feature = "rustls-with-native-roots")]
            TlsBackend::RustlsWithNativeRoots => builder
                .use_rustls_tls()
                .tls_built_in_root_certs(false)
                .tls_built_in_webpki_certs(false)
                .tls_built_in_native_certs(true),
        };
        if let Ok(ref host) = env::var("HTTP_PROXY") {
            if let Ok(proxy) = Proxy::http(host) {
                builder = builder.proxy(proxy);
            }
        }
        if let Ok(ref host) = env::var("HTTPS_PROXY") {
            if let Ok(proxy) = Proxy::https(host) {
                builder = builder.proxy(proxy);
            }
        }
        builder.build().context("Could not instantiate HTTP client")
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

trait DirectoryName {
    type S;
    fn directory_name(&self) -> Self::S;
}

impl DirectoryName for Language<'_> {
    type S = String;

    fn directory_name(&self) -> Self::S {
        format!("pages.{}", self.0)
    }
}

impl DirectoryName for PlatformType {
    type S = &'static str;

    fn directory_name(&self) -> Self::S {
        match self {
            PlatformType::Linux => "linux",
            PlatformType::OsX => "osx",
            PlatformType::SunOs => "sunos",
            PlatformType::Windows => "windows",
            PlatformType::Android => "android",
            PlatformType::FreeBsd => "freebsd",
            PlatformType::NetBsd => "netbsd",
            PlatformType::OpenBsd => "openbsd",
            PlatformType::Common => "common",
        }
    }
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
