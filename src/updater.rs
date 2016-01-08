use std::io::Read;
use std::fs;
use std::env;
use std::path::PathBuf;

#[cfg(unix)] use std::os::unix::fs::MetadataExt;

use flate2::read::GzDecoder;
use tar::Archive;
use curl::http;
use time;

use error::TldrError::{self, UpdateError};


#[derive(Debug)]
pub struct Updater {
    url: String,
}

impl Updater {

    pub fn new<S>(url: S) -> Updater where S: Into<String> {
        Updater {
            url: url.into(),
        }
    }

    /// Return the path to the cache directory.
    fn get_cache_dir(&self) -> Result<PathBuf, TldrError> {
        let home_dir = try!(env::home_dir().ok_or(UpdateError("Could not determine home directory".into())));
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

    /// Return the number of seconds since the cache directory was last modified.
    #[cfg(unix)]
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

}
