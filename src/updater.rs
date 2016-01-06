use std::io::Read;
use std::path::Path;
use std::fs;
use std::env;

use flate2::read::GzDecoder;
use tar::Archive;
use curl::http;
use rustc_serialize::json;

use error::TldrError::{self, UpdateError};


#[derive(Debug, RustcEncodable, RustcDecodable)]
struct TldrIndex {
    commands: Vec<TldrCommand>,
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
struct TldrCommand {
    name: String,
    platform: Vec<String>,
}

impl TldrIndex {
    /// Return vec of all occuring platforms, without duplicates.
    fn unique_platforms(&self) -> Vec<String> {
        let mut flattened: Vec<String> = self.commands
                                             .iter()
                                             .flat_map(|cmd| cmd.platform.clone())
                                             .collect();
        flattened.sort();
        flattened.dedup();
        flattened
    }
}

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

    /// Given the path to the `pages` directory, return `TldrIndex` instances.
    fn get_index(&self, path: &Path) -> Result<TldrIndex, TldrError> {
        let mut buffer = String::new();
        let mut index = try!(fs::File::open(path.join("index.json"))
            .map_err(|_| UpdateError("Could not open index.json".into())));
        try!(index.read_to_string(&mut buffer)
            .map_err(|_| UpdateError("Could not read index.json".into())));
        let deserialized: TldrIndex = try!(json::decode(&buffer)
            .map_err(|e| UpdateError(format!("Could not deserialize index.json: {}", e))));
        Ok(deserialized)
    }

    /// Copy all pages to the cache. Return the number of copied pages.
    fn copy_pages(&self, src_dir: &Path, dst_dir: &Path, index: &TldrIndex) -> Result<u64, TldrError> {
        // Make sure all platform directories exist
        for platform in &index.unique_platforms() {
            debug!("Ensure platform directory {:?} exists", &dst_dir.join(platform));
            try!(fs::create_dir_all(&dst_dir.join(platform)).map_err(|e| {
                UpdateError(format!("Could not create platform directory: {}", e))
            }));
        }

        let mut copied = 0u64;
        for page in &index.commands {
            for platform in &page.platform {
                let relpath = Path::new(&platform).join(format!("{}.md", &page.name));
                let bytes = fs::copy(&src_dir.join(&relpath), &dst_dir.join(&relpath)).unwrap_or_else(|e| {
                    debug!("Could not copy {:?} to {:?}: {}",
                           &src_dir.join(&relpath), &dst_dir.join(&relpath), e);
                    println!("Warning: Could not copy the tldr page for the \"{}\" command.", &page.name);
                    0u64
                });
                if bytes > 0 {
                    copied += 1;
                };
            }
        };

        debug!("Copied {} pages to {:?}", copied, &dst_dir);
        Ok(copied)
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
