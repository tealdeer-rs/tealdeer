use std::io::Read;
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};
use std::fs;
use std::env;

use flate2::read::GzDecoder;
use tar::Archive;
use curl::http;
use tempdir::TempDir;

use error::TldrError::{self, UpdateError};


#[derive(Debug)]
pub struct Updater {
    url: String,
}

impl Updater {

    pub fn new(url: String) -> Updater {
        Updater {
            url: url,
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

    /// Extract archive, return pages and license
    fn extract<R: Read>(&self, archive: &mut Archive<R>, path: &Path) -> Result<(PathBuf, PathBuf), TldrError> {
        try!(archive.unpack(path).map_err(|e| {
            UpdateError(format!("Could not unpack compressed data: {}", e))
        }));
        let repodir = path.join("tldr-master");
        Ok((repodir.join("pages"), repodir.join("LICENSE.md")))
    }

    fn get_pages(&self, path: &Path) -> Vec<PathBuf> {
    }

    /// Update the pages cache.
    pub fn update(&self) -> Result<(), TldrError> {
        // First, download the compressed data
        let response = try!(self.download());

        // Decompress the response body into an `Archive`
        let mut archive = try!(self.decompress(response.get_body()));

        // Create temporary directory
        let dir = try!(TempDir::new("tldr").map_err(|e| {
            UpdateError(format!("Could not create temporary directory: {}", e))
        }));

        // Get paths to pages and license
        let (pages_src, license_src) = try!(self.extract(&mut archive, dir.path()));

        // Determine path to cache
        let home_dir = try!(env::home_dir().ok_or(UpdateError("Could not determine home directory".into())));
        let cache_dir = home_dir.join(".tldr").join("cache");

        // Make sure that cache dir exists
        try!(fs::create_dir_all(&cache_dir).map_err(|e| {
            UpdateError(format!("Could not create cache directory: {}", e))
        }));

        // Copy license file
        let license_dst = &cache_dir.join("LICENSE.md");
        debug!("Copy license from {:?} to {:?}", &license_src, &license_dst);
        try!(fs::copy(&license_src, &license_dst).map_err(|e| {
            UpdateError(format!("Could not extract license file: {}", e))
        }));
        
        Ok(())
    }

}
