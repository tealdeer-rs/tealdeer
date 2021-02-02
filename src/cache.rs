use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::iter;
use std::path::{Path, PathBuf};

use app_dirs::{get_app_root, AppDataType};
use flate2::read::GzDecoder;
use log::debug;
use reqwest::{blocking::Client, Proxy};
use std::time::{Duration, SystemTime};
use tar::Archive;
use walkdir::{DirEntry, WalkDir};

use crate::config::Config;
use crate::error::TealdeerError::{self, CacheError, UpdateError};
use crate::types::OsType;

#[derive(Debug)]
pub struct Cache {
    url: String,
    os: OsType,
}

pub struct PageLookupResult {
    page_path: PathBuf,
    patch_path: Option<PathBuf>,
}

impl PageLookupResult {
    fn with_page(page_path: PathBuf) -> Self {
        Self {
            page_path,
            patch_path: None,
        }
    }

    fn with_patch(mut self, patch_path: PathBuf) -> Self {
        self.patch_path = Some(patch_path);
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = &Path> {
        iter::once(self.page_path.as_path()).chain(self.patch_path.as_deref().into_iter())
    }
}

impl Cache {
    pub fn new<S>(url: S, os: OsType) -> Self
    where
        S: Into<String>,
    {
        Self {
            url: url.into(),
            os,
        }
    }

    /// Return the path to the cache directory.
    fn get_cache_dir() -> Result<PathBuf, TealdeerError> {
        // Allow overriding the cache directory by setting the
        // $TEALDEER_CACHE_DIR env variable.
        if let Ok(value) = env::var("TEALDEER_CACHE_DIR") {
            let path = PathBuf::from(value);

            if path.exists() && path.is_dir() {
                return Ok(path);
            } else {
                return Err(CacheError(
                    "Path specified by $TEALDEER_CACHE_DIR \
                     does not exist or is not a directory."
                        .into(),
                ));
            }
        };

        // Otherwise, fall back to user cache directory.
        match get_app_root(AppDataType::UserCache, &crate::APP_INFO) {
            Ok(dirs) => Ok(dirs),
            Err(_) => Err(CacheError(
                "Could not determine user cache directory.".into(),
            )),
        }
    }

    /// Download the archive
    fn download(&self) -> Result<Vec<u8>, TealdeerError> {
        let mut builder = Client::builder();
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
        let client = builder.build().unwrap_or_else(|_| Client::new());
        let mut resp = client.get(&self.url).send()?;
        let mut buf: Vec<u8> = vec![];
        let bytes_downloaded = resp.copy_to(&mut buf)?;
        debug!("{} bytes downloaded", bytes_downloaded);
        Ok(buf)
    }

    /// Decompress and open the archive
    fn decompress<R: Read>(reader: R) -> Archive<GzDecoder<R>> {
        Archive::new(GzDecoder::new(reader))
    }

    /// Update the pages cache.
    pub fn update(&self) -> Result<(), TealdeerError> {
        // First, download the compressed data
        let bytes: Vec<u8> = self.download()?;

        // Decompress the response body into an `Archive`
        let mut archive = Self::decompress(&bytes[..]);

        // Determine paths
        let cache_dir = Self::get_cache_dir()?;

        // Make sure that cache directory exists
        debug!("Ensure cache directory {:?} exists", &cache_dir);
        fs::create_dir_all(&cache_dir)
            .map_err(|e| UpdateError(format!("Could not create cache directory: {}", e)))?;

        // Clear cache directory
        // Note: This is not the best solution. Ideally we would download the
        // archive to a temporary directory and then swap the two directories.
        // But renaming a directory doesn't work across filesystems and Rust
        // does not yet offer a recursive directory copying function. So for
        // now, we'll use this approach.
        Self::clear()?;

        // Extract archive
        archive
            .unpack(&cache_dir)
            .map_err(|e| UpdateError(format!("Could not unpack compressed data: {}", e)))?;

        Ok(())
    }

    /// Return the duration since the cache directory was last modified.
    pub fn last_update() -> Option<Duration> {
        if let Ok(cache_dir) = Self::get_cache_dir() {
            if let Ok(metadata) = fs::metadata(cache_dir.join("tldr-master")) {
                if let Ok(mtime) = metadata.modified() {
                    let now = SystemTime::now();
                    return now.duration_since(mtime).ok();
                };
            };
        };
        None
    }

    /// Return the platform directory.
    #[allow(clippy::match_same_arms)]
    fn get_platform_dir(&self) -> Option<&'static str> {
        match self.os {
            OsType::Linux => Some("linux"),
            OsType::OsX => Some("osx"),
            OsType::SunOs => None, // TODO: Does Rust support SunOS?
            OsType::Windows => Some("windows"),
            OsType::Other => None,
        }
    }

    /// Search for a page and return the path to it.
    pub fn find_pages(&self, name: &str, config: &Config) -> Option<PageLookupResult> {
        // Get platform dir
        let platforms_dir = match Self::get_cache_dir() {
            Ok(cache_dir) => cache_dir.join("tldr-master").join("pages"),
            Err(e) => {
                log::error!("Could not get cache directory: {}", e);
                return None;
            }
        };

        // Get custom pages directory
        let custom_pages_directory = &config.directories.custom_pages_dir;

        // Look up custom page (<name>.page). If it exists, return it directly,
        let custom_page = platforms_dir
            .join(&custom_pages_directory)
            .join(&format!("{}.page", name));
        if custom_page.exists() && custom_page.is_file() {
            return Some(PageLookupResult::with_page(custom_page));
        }

        // Look up custom patch (<name>.patch). If it exists, store it in a variable.
        let custom_patch = platforms_dir
            .join(&custom_pages_directory)
            .join(&format!("{}.patch", name));
        let custom_patch = if custom_patch.exists() && custom_patch.is_file() {
            Some(custom_patch)
        } else {
            None
        };

        let page_filename = format!("{}.md", name);

        // Look up platform-specific page. If it exists, return page plus the patch above (if set).
        if let Some(platform_directory) = self.get_platform_dir() {
            let page = platforms_dir.join(&platform_directory).join(&page_filename);
            if page.exists() && page.is_file() {
                match custom_patch {
                    Some(patch) => {
                        return Some(PageLookupResult::with_page(page).with_patch(patch))
                    }
                    None => return Some(PageLookupResult::with_page(page)),
                }
            }
        }

        // Look up page from the "common" directory. If it exists, return page plus the patch above (if set).
        let page = platforms_dir.join("common").join(&page_filename);
        if page.exists() && page.is_file() {
            match custom_patch {
                Some(patch) => return Some(PageLookupResult::with_page(page).with_patch(patch)),
                None => return Some(PageLookupResult::with_page(page)),
            }
        }

        // None of the above were resolved, return an empty vec.
        None
    }

    /// Return the available pages.
    pub fn list_pages(&self) -> Result<Vec<String>, TealdeerError> {
        // Determine platforms directory and platform
        let cache_dir = Self::get_cache_dir()?;
        let platforms_dir = cache_dir.join("tldr-master").join("pages");
        let platform_dir = self.get_platform_dir();

        // Closure that allows the WalkDir instance to traverse platform
        // specific and common page directories, but not others.
        let should_walk = |entry: &DirEntry| -> bool {
            let file_type = entry.file_type();
            let file_name = match entry.file_name().to_str() {
                Some(name) => name,
                None => return false,
            };
            if file_type.is_dir() {
                if file_name == "common" {
                    return true;
                }
                if let Some(platform) = platform_dir {
                    return file_name == platform;
                }
            } else if file_type.is_file() {
                return true;
            }
            false
        };

        // Recursively walk through common and (if applicable) platform specific directory
        let mut pages = WalkDir::new(platforms_dir)
            .min_depth(1) // Skip root directory
            .into_iter()
            .filter_entry(|e| should_walk(e)) // Filter out pages for other architectures
            .filter_map(Result::ok) // Convert results to options, filter out errors
            .filter_map(|e| {
                let path = e.path();
                let extension = &path.extension().and_then(OsStr::to_str).unwrap_or("");
                if e.file_type().is_file() && extension == &"md" {
                    path.file_stem()
                        .and_then(|stem| stem.to_str().map(|s| s.into()))
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();
        pages.sort();
        pages.dedup();
        Ok(pages)
    }

    /// Delete the cache directory.
    pub fn clear() -> Result<(), TealdeerError> {
        let path = Self::get_cache_dir()?;
        if path.exists() && path.is_dir() {
            fs::remove_dir_all(&path).map_err(|_| {
                CacheError(format!(
                    "Could not remove cache directory ({}).",
                    path.display()
                ))
            })?;
        } else if path.exists() {
            return Err(CacheError(format!(
                "Cache path ({}) is not a directory.",
                path.display()
            )));
        } else {
            return Err(CacheError(format!(
                "Cache path ({}) does not exist.",
                path.display()
            )));
        };
        Ok(())
    }
}
