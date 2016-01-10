use std::io::Read;
use std::fs;
use std::env;
use std::path::PathBuf;

#[cfg(unix)] use std::os::unix::fs::MetadataExt;

use flate2::read::GzDecoder;
use tar::Archive;
use curl::http;
use time;

use error::TldrError::{self, CacheError, UpdateError};


#[derive(Debug)]
pub struct Cache {
    url: String,
}

impl Cache {

    pub fn new<S>(url: S) -> Cache where S: Into<String> {
        Cache {
            url: url.into(),
        }
    }

    /// Return the path to the cache directory.
    fn get_cache_dir(&self) -> Result<PathBuf, TldrError> {
        let home_dir = try!(env::home_dir().ok_or(CacheError("Could not determine home directory".into())));
        Ok(home_dir.join(".cache").join("tldr-rs"))
    }

    /// Download the archive
    fn download(&self) -> Result<http::Response, TldrError> {
        let resp = try!(
            http::handle()
                 .follow_location(1)
                 .get(&self.url[..])
                 .exec()
        );
        Ok(resp)
    }

    /// Decompress and open the archive
    fn decompress<R: Read>(&self, reader: R) -> Result<Archive<GzDecoder<R>>, TldrError> {
        let decoder = try!(GzDecoder::new(reader).map_err(|_| UpdateError("Could not decode gzip data".into())));
        Ok(Archive::new(decoder))
    }

    /// Update the pages cache. Return the number of cached pages.
    pub fn update(&self) -> Result<(), TldrError> {
        // First, download the compressed data
        let response = try!(self.download());

        // Decompress the response body into an `Archive`
        let mut archive = try!(self.decompress(response.get_body()));

        // Determine paths
        let cache_dir = try!(self.get_cache_dir());

        // Extract archive
        try!(archive.unpack(&cache_dir).map_err(|e| {
            UpdateError(format!("Could not unpack compressed data: {}", e))
        }));

        // Make sure that cache directory exists
        debug!("Ensure cache directory {:?} exists", &cache_dir);
        try!(fs::create_dir_all(&cache_dir).map_err(|e| {
            UpdateError(format!("Could not create cache directory: {}", e))
        }));

        Ok(())
    }

    #[cfg(unix)]
    /// Return the number of seconds since the cache directory was last modified.
    pub fn last_update(&self) -> Option<i64> {
        if let Ok(cache_dir) = self.get_cache_dir() {
            if let Ok(metadata) = fs::metadata(cache_dir) {
                let mtime = metadata.mtime();
                let now = time::now_utc().to_timespec();
                return Some(now.sec - mtime)
            };
        };
        None
    }

    pub fn find_page(&self, name: &str) -> Option<PathBuf> {
        // Build page file name
        let page_filename = format!("{}.md", name);

        // Get platform dir
        let cache_dir = match self.get_cache_dir() {
            Ok(dir) => dir,
            Err(_) => return None,
        };
        let platforms_dir = cache_dir.join("tldr-master").join("pages");

        // Determine platform
        let platform = if cfg!(target_os = "linux") {
            Some("linux")
        } else if cfg!(target_os = "macos") {
            Some("osx")
        } else {
            None // TODO: Does rust support Sun OS?
        };

        // Search for the page in the platform specific directory
        if let Some(pf) = platform {
            let path = platforms_dir.join(&pf).join(&page_filename);
            if path.exists() && path.is_file() {
                return Some(path);
            }
        }

        // If platform is not supported or if platform specific page does not exist,
        // look up the page in the "common" directory.
        let path = platforms_dir.join("common").join(&page_filename);

        // Return it if it exists, otherwise give up and return `None`
        if path.exists() && path.is_file() {
            Some(path)
        } else {
            None
        }
    }

    /// Delete the cache directory.
    pub fn clear(&self) -> Result<(), TldrError> {
        let path = try!(self.get_cache_dir());
        if path.exists() && path.is_dir() {
            try!(fs::remove_dir_all(&path).map_err(|_| {
                CacheError(format!("Could not remove cache directory ({}).", path.display()))
            }));
        } else if path.exists() {
            return Err(CacheError(format!("Cache path ({}) is not a directory.", path.display())));
        } else {
            return Err(CacheError(format!("Cache path ({}) does not exist.", path.display())));
        };
        Ok(())
    }

}
