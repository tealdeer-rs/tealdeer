use std::io::Read;
use std::fs;
use std::env;

use flate2::read::GzDecoder;
use tar::Archive;
use curl::http;

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
        let home_dir = try!(env::home_dir().ok_or(UpdateError("Could not determine home directory".into())));
        let cache_dir = home_dir.join(".cache").join("tldr-rs");

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

}
