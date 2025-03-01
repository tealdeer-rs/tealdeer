use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{BufReader, Cursor, Read},
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use anyhow::{ensure, Context, Result};
use log::debug;
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};
use ureq::Agent;
use walkdir::{DirEntry, WalkDir};
use zip::ZipArchive;

use crate::{config::TlsBackend, types::PlatformType, utils::print_warning};

pub static TLDR_PAGES_DIR: &str = "tldr-pages";
static TLDR_OLD_PAGES_DIR: &str = "tldr-master";

#[derive(Debug)]
pub struct Cache {
    cache_dir: PathBuf,
    enable_styles: bool,
    tls_backend: TlsBackend,
}

#[derive(Debug)]
pub struct PageLookupResult {
    pub page_path: PathBuf,
    pub patch_path: Option<PathBuf>,
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

impl Cache {
    pub fn new<P>(cache_dir: P, enable_styles: bool, tls_backend: TlsBackend) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            cache_dir: cache_dir.into(),
            enable_styles,
            tls_backend,
        }
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Make sure that the cache directory exists and is a directory.
    /// If necessary, create the directory.
    fn ensure_cache_dir_exists(&self) -> Result<()> {
        // Check whether `cache_dir` exists and is a directory
        let (cache_dir_exists, cache_dir_is_dir) = self
            .cache_dir
            .metadata()
            .map_or((false, false), |md| (true, md.is_dir()));
        ensure!(
            !cache_dir_exists || cache_dir_is_dir,
            "Cache directory path `{}` is not a directory",
            self.cache_dir.display(),
        );

        if !cache_dir_exists {
            // If missing, try to create the complete directory path
            fs::create_dir_all(&self.cache_dir).with_context(|| {
                format!(
                    "Cache directory path `{}` cannot be created",
                    self.cache_dir.display(),
                )
            })?;
            eprintln!(
                "Successfully created cache directory path `{}`.",
                self.cache_dir.display(),
            );
        }

        Ok(())
    }

    fn pages_dir(&self) -> PathBuf {
        self.cache_dir.join(TLDR_PAGES_DIR)
    }

    /// Update the pages cache from the specified URL.
    pub fn update(&self, archive_source: &str) -> Result<()> {
        self.ensure_cache_dir_exists()?;

        let archive_url = format!("{archive_source}/tldr.zip");

        let client = Self::build_client(self.tls_backend)?;
        // First, download the compressed data
        let bytes: Vec<u8> = Self::download(&client, &archive_url)?;

        // Decompress the response body into an `Archive`
        let mut archive = ZipArchive::new(Cursor::new(bytes))
            .context("Could not decompress downloaded ZIP archive")?;

        // Clear cache directory
        // Note: This is not the best solution. Ideally we would download the
        // archive to a temporary directory and then swap the two directories.
        // But renaming a directory doesn't work across filesystems and Rust
        // does not yet offer a recursive directory copying function. So for
        // now, we'll use this approach.
        self.clear()
            .context("Could not clear the cache directory")?;

        // Extract archive into pages dir
        archive
            .extract(self.pages_dir())
            .context("Could not unpack compressed data")?;

        Ok(())
    }

    /// Return the duration since the cache directory was last modified.
    pub fn last_update(&self) -> Option<Duration> {
        if let Ok(metadata) = fs::metadata(self.pages_dir()) {
            if let Ok(mtime) = metadata.modified() {
                let now = SystemTime::now();
                return now.duration_since(mtime).ok();
            };
        };
        None
    }

    /// Return the freshness of the cache (fresh, stale or missing).
    pub fn freshness(&self) -> CacheFreshness {
        match self.last_update() {
            Some(ago) if ago > crate::config::MAX_CACHE_AGE => CacheFreshness::Stale(ago),
            Some(_) => CacheFreshness::Fresh,
            None => CacheFreshness::Missing,
        }
    }

    /// Return the platform directory.
    fn get_platform_dir(platform: PlatformType) -> &'static str {
        match platform {
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

    /// Check for pages for a given platform in one of the given languages.
    fn find_page_for_platform(
        page_name: &str,
        pages_dir: &Path,
        platform: &str,
        language_dirs: &[String],
    ) -> Option<PathBuf> {
        language_dirs
            .iter()
            .map(|lang_dir| pages_dir.join(lang_dir).join(platform).join(page_name))
            .find(|path| path.exists() && path.is_file())
    }

    /// Look up custom patch (<name>.patch.md). If it exists, store it in a variable.
    fn find_patch(patch_name: &str, custom_pages_dir: Option<&Path>) -> Option<PathBuf> {
        custom_pages_dir
            .map(|custom_dir| custom_dir.join(patch_name))
            .filter(|path| path.exists() && path.is_file())
    }

    /// Search for a page and return the path to it.
    pub fn find_page(
        &self,
        name: &str,
        languages: &[String],
        custom_pages_dir: Option<&Path>,
        platforms: &[PlatformType],
    ) -> Option<PageLookupResult> {
        let page_filename = format!("{name}.md");
        let patch_filename = format!("{name}.patch.md");
        let custom_filename = format!("{name}.page.md");

        // Determine directory paths
        let pages_dir = self.pages_dir();
        let lang_dirs: Vec<String> = languages
            .iter()
            .map(|lang| {
                if lang == "en" {
                    String::from("pages")
                } else {
                    format!("pages.{lang}")
                }
            })
            .collect();

        // Look up custom page (<name>.page.md). If it exists, return it directly
        if let Some(config_dir) = custom_pages_dir {
            // TODO: Remove this check 1 year after version 1.7.0 was released
            self.check_for_old_custom_pages(config_dir);

            let custom_page = config_dir.join(custom_filename);
            if custom_page.exists() && custom_page.is_file() {
                return Some(PageLookupResult::with_page(custom_page));
            }
        }

        let patch_path = Self::find_patch(&patch_filename, custom_pages_dir);

        // Try to find a platform specific path next, in the order supplied by the user, and append custom patch to it.
        for &platform in platforms {
            let platform_dir = Cache::get_platform_dir(platform);
            if let Some(page) =
                Self::find_page_for_platform(&page_filename, &pages_dir, platform_dir, &lang_dirs)
            {
                return Some(PageLookupResult::with_page(page).with_optional_patch(patch_path));
            }
        }

        None
    }

    /// Return the available pages.
    pub fn list_pages(
        &self,
        custom_pages_dir: Option<&Path>,
        platforms: &[PlatformType],
    ) -> Vec<String> {
        // Determine platforms directory and platform
        let platforms_dir = self.pages_dir().join("pages");
        let platform_dirs: Vec<&'static str> = platforms
            .iter()
            .map(|&p| Self::get_platform_dir(p))
            .collect();

        // Closure that allows the WalkDir instance to traverse platform
        // relevant page directories, but not others.
        let should_walk = |entry: &DirEntry| -> bool {
            let file_type = entry.file_type();
            let Some(file_name) = entry.file_name().to_str() else {
                return false;
            };
            if file_type.is_dir() {
                return platform_dirs.contains(&file_name);
            } else if file_type.is_file() {
                return true;
            }
            false
        };

        let to_stem = |entry: DirEntry| -> Option<String> {
            entry
                .path()
                .file_stem()
                .and_then(OsStr::to_str)
                .map(str::to_string)
        };

        let to_stem_custom = |entry: DirEntry| -> Option<String> {
            entry
                .path()
                .file_name()
                .and_then(OsStr::to_str)
                .and_then(|s| s.strip_suffix(".page.md"))
                .map(str::to_string)
        };

        // Recursively walk through platform specific directory
        let mut pages = WalkDir::new(platforms_dir)
            .min_depth(1) // Skip root directory
            .into_iter()
            .filter_entry(should_walk) // Filter out pages for other architectures
            .filter_map(Result::ok) // Convert results to options, filter out errors
            .filter_map(|e| {
                let extension = e.path().extension().unwrap_or_default();
                if e.file_type().is_file() && extension == "md" {
                    to_stem(e)
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();

        if let Some(custom_pages_dir) = custom_pages_dir {
            let is_page = |entry: &DirEntry| -> bool {
                entry.file_type().is_file()
                    && entry
                        .path()
                        .file_name()
                        .and_then(OsStr::to_str)
                        .is_some_and(|file_name| file_name.ends_with(".page.md"))
            };

            let custom_pages = WalkDir::new(custom_pages_dir)
                .min_depth(1)
                .max_depth(1)
                .into_iter()
                .filter_entry(is_page)
                .filter_map(Result::ok)
                .filter_map(to_stem_custom);

            pages.extend(custom_pages);
        }

        pages.sort();
        pages.dedup();
        pages
    }

    /// Delete the cache directory
    ///
    /// Returns true if the cache was deleted and false if the cache dir did
    /// not exist.
    pub fn clear(&self) -> Result<bool> {
        if !self.cache_dir.exists() {
            return Ok(false);
        }
        ensure!(
            self.cache_dir.is_dir(),
            "Cache path ({}) is not a directory.",
            self.cache_dir.display(),
        );

        // Delete old tldr-pages cache location as well if present
        // TODO: To be removed in the future
        for pages_dir_name in [TLDR_PAGES_DIR, TLDR_OLD_PAGES_DIR] {
            let pages_dir = self.cache_dir.join(pages_dir_name);

            if pages_dir.exists() {
                fs::remove_dir_all(&pages_dir).with_context(|| {
                    format!(
                        "Could not remove the cache directory at {}",
                        pages_dir.display()
                    )
                })?;
            }
        }

        Ok(true)
    }

    /// Check for old custom pages (without .md suffix) and print a warning.
    fn check_for_old_custom_pages(&self, custom_pages_dir: &Path) {
        let old_custom_pages_exist = WalkDir::new(custom_pages_dir)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_entry(|entry| entry.file_type().is_file())
            .any(|entry| {
                if let Ok(entry) = entry {
                    let extension = entry.path().extension();
                    if let Some(extension) = extension {
                        extension == "page" || extension == "patch"
                    } else {
                        false
                    }
                } else {
                    false
                }
            });
        if old_custom_pages_exist {
            print_warning(
                self.enable_styles,
                &format!(
                    "Custom pages using the old naming convention were found in {}.\n\
                     Please rename them to follow the new convention:\n\
                     - `<name>.page` → `<name>.page.md`\n\
                     - `<name>.patch` → `<name>.patch.md`",
                    custom_pages_dir.display()
                ),
            );
        }
    }
}

impl Cache {
    fn build_client(tls_backend: TlsBackend) -> Result<Agent> {
        let tls_builder = match tls_backend {
            #[cfg(feature = "native-tls")]
            TlsBackend::NativeTls => TlsConfig::builder()
                .provider(TlsProvider::NativeTls)
                .root_certs(RootCerts::PlatformVerifier),
            #[cfg(feature = "rustls-with-webpki-roots")]
            TlsBackend::RustlsWithWebpkiRoots => TlsConfig::builder()
                .provider(TlsProvider::Rustls)
                .root_certs(RootCerts::WebPki),
            #[cfg(feature = "rustls-with-native-roots")]
            TlsBackend::RustlsWithNativeRoots => TlsConfig::builder()
                .provider(TlsProvider::Rustls)
                .root_certs(RootCerts::PlatformVerifier),
        };
        let config = Agent::config_builder()
            .tls_config(tls_builder.build())
            .build();

        Ok(config.into())
    }

    /// Download the archive from the specified URL.
    fn download(client: &Agent, archive_url: &str) -> Result<Vec<u8>> {
        let response = client
            .get(archive_url)
            .call()
            .with_context(|| format!("Could not download tldr pages from {archive_url}"))?;
        let mut buf: Vec<u8> = Vec::new();
        response.into_body().into_reader().read_to_end(&mut buf)?;
        debug!("{} bytes downloaded", buf.len());
        Ok(buf)
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
