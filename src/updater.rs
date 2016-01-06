use std::io::Read;
use std::path::{Path, PathBuf};
use std::fs;
use std::env;

use flate2::read::GzDecoder;
use tar::Archive;
use curl::http;
use tempdir::TempDir;
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

    /// Extract archive, return pages and license
    fn extract<R: Read>(&self, archive: &mut Archive<R>, path: &Path) -> Result<(PathBuf, PathBuf), TldrError> {
        try!(archive.unpack(path).map_err(|e| {
            UpdateError(format!("Could not unpack compressed data: {}", e))
        }));
        let repodir = path.join("tldr-master");
        Ok((repodir.join("pages"), repodir.join("LICENSE.md")))
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
    pub fn update(&self) -> Result<u64, TldrError> {
        // First, download the compressed data
        let response = try!(self.download());

        // Decompress the response body into an `Archive`
        let mut archive = try!(self.decompress(response.get_body()));

        // Create temporary directory
        let dir = try!(TempDir::new("tldr").map_err(|e| {
            UpdateError(format!("Could not create temporary directory: {}", e))
        }));

        // Extract archive and get paths to pages and license
        let (pages_src, license_src) = try!(self.extract(&mut archive, dir.path()));

        // Determine paths
        let home_dir = try!(env::home_dir().ok_or(UpdateError("Could not determine home directory".into())));
        let cache_dir = home_dir.join(".tldr").join("cache");
        let pages_dir = &cache_dir.join("pages");
        let license_dst = &cache_dir.join("LICENSE.md");
        let index_dst = &pages_dir.join("index.json");

        // Make sure that cache and pages directories exist
        debug!("Ensure cache directory {:?} exists", &cache_dir);
        try!(fs::create_dir_all(&cache_dir).map_err(|e| {
            UpdateError(format!("Could not create cache directory: {}", e))
        }));
        debug!("Ensure pages directory {:?} exists", &pages_dir);
        try!(fs::create_dir_all(&pages_dir).map_err(|e| {
            UpdateError(format!("Could not create pages directory: {}", e))
        }));

        // Copy license file
        debug!("Copy license from {:?} to {:?}", &license_src, &license_dst);
        try!(fs::copy(&license_src, &license_dst).map_err(|e| {
            UpdateError(format!("Could not extract license file: {}", e))
        }));

        // Copy index file
        let index_src = &pages_src.join("index.json");
        debug!("Copy index from {:?} to {:?}", &index_src, &index_dst);
        try!(fs::copy(&index_src, &index_dst).map_err(|e| {
            UpdateError(format!("Could not extract index file: {}", e))
        }));

        // Copy pages
        let index = try!(self.get_index(&pages_src));
        let copied = try!(self.copy_pages(&pages_src, &pages_dir, &index));
        
        Ok(copied)
    }

}
