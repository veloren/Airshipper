///! e.g. run cargo run -- ~/.local/share/airshipper/profiles/default https://github.com/veloren/veloren/releases/download/nightly/nightly-linux-x86_64-2024-06-09T16_04.zip
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use clap::Parser;
use reqwest::header::RANGE;
use tokio::runtime::Runtime;

mod partial_buffer;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// specify which directory to update with the fast update method
    veloren_airshipper_dir: PathBuf,

    /// specify source to update local dir from
    online_zip: String,
}

fn main() {
    println!("Hello, world!");
    //test_local();
    let cli = Cli::parse();
    let runtime = Runtime::new().unwrap();

    let local = runtime
        .block_on(fetch_local_info(&cli.veloren_airshipper_dir))
        .unwrap();
    let remote = runtime.block_on(fetch_server_info(cli.online_zip)).unwrap();
    runtime.block_on(compare_files(local, remote));
}

async fn fetch_server_info(
    online_zip: String,
) -> Result<Vec<zip_structs::zip_central_directory::ZipCDEntry>, Box<dyn std::error::Error>>
{
    use std::io::Cursor;
    let client = reqwest::Client::builder().http2_prior_knowledge().build()?;
    let document_size = client.get(&online_zip).send().await?;
    let content_length = document_size.content_length().unwrap_or(0);
    println!("content-length: {}", content_length);

    let approx_eocd_start = content_length - 60000;
    // Get EOCD
    let range = format!("bytes={}-{}", approx_eocd_start, content_length);
    let eocd_res = client.get(&online_zip).header(RANGE, range).send().await?;
    let eocd_bytes = eocd_res.bytes().await?;
    let eocd_bytes_len = eocd_bytes.len();
    let mut cursor = Cursor::new(eocd_bytes);

    let eocd = zip_structs::zip_eocd::ZipEOCD::from_reader(&mut cursor)?;
    println!("n_cd_entries_in_disk: {}", eocd.n_cd_entries_in_disk);
    println!("n_cd_entries: {}", eocd.n_cd_entries);
    println!("cd_size: {}", eocd.cd_size);
    println!("cd_starting_position: {}", eocd.cd_starting_position);
    println!("bytes_downloaded: {}", eocd_bytes_len);

    // List of all Central Directories
    let cds_start = eocd.cd_starting_position;
    let cds_end = eocd.cd_starting_position + eocd.cd_size;
    let range = format!("bytes={}-{}", cds_start, cds_end);

    let cd_res = client.get(&online_zip).header(RANGE, range).send().await?;
    let cd_bytes = cd_res.bytes().await?;
    let cd_bytes_len = cd_bytes.len();
    let mut cursor = Cursor::new(cd_bytes);

    let mut cds = Vec::new();
    while let Ok(cd) =
        zip_structs::zip_central_directory::ZipCDEntry::read_and_generate_from_signature(
            &mut cursor,
        )
    {
        let crc = cd.crc32;
        let file_name = std::str::from_utf8(&cd.file_name_raw);
        let download_size = cd.compressed_size;

        println!(
            "{:<60} [{}kb] crc32: {}",
            file_name.unwrap(),
            download_size / 1024,
            crc
        );

        cds.push(cd);
    }
    println!("entries: {}", cds.len());
    println!("bytes_downloaded: {}", cd_bytes_len);

    Ok(cds)
}

async fn fetch_local_info(
    veloren_airshipper_dir: &Path,
) -> Result<Vec<(String, u32)>, Box<dyn std::error::Error>> {
    use std::io::Read;
    fn visit_dir(
        dir: &Path,
        files: &mut Vec<PathBuf>,
        dirs: &mut Vec<PathBuf>,
    ) -> Result<(), std::io::Error> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path.to_owned());
                visit_dir(&path, files, dirs)?;
            } else {
                files.push(path.to_owned());
            }
        }
        Ok(())
    }
    let mut local_files = Vec::new();
    let mut local_dirs = Vec::new();
    visit_dir(veloren_airshipper_dir, &mut local_files, &mut local_dirs)?;
    let local_files = local_files
        .into_iter()
        .map(|path| {
            let mut f = std::fs::File::open(&path).unwrap();
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer).unwrap();

            let crc_inst = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
            let crc32 = crc_inst.checksum(&buffer);

            let file_name = path
                .strip_prefix(veloren_airshipper_dir)
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            println!("{:?} crc32: {}", &file_name, crc32);

            (file_name, crc32)
        })
        .collect();

    Ok(local_files)
}

async fn compare_files(
    local: Vec<(String, u32)>,
    remote: Vec<zip_structs::zip_central_directory::ZipCDEntry>,
) {
    let mut compare_map: HashMap<
        String,
        (
            Option<u32>,
            Option<zip_structs::zip_central_directory::ZipCDEntry>,
        ),
    > = HashMap::new();

    for l in local {
        let _ = compare_map.entry(l.0).or_insert((Some(l.1), None));
    }
    for r in remote {
        let file_name = std::str::from_utf8(&r.file_name_raw).unwrap().to_string();
        let e = compare_map.entry(file_name).or_insert((None, None));
        e.1 = Some(r);
    }

    for (n, d) in compare_map.iter() {
        let msg = match (&d.0, &d.1) {
            (Some(_), None) => "REMOVED",
            (None, Some(_)) => "NEW",
            (None, None) => unreachable!(),
            (Some(f), Some(r)) if *f == r.crc32 => "OK",
            _ => "OUTDATED",
        };
        println!("{:<60} : {msg}", n);
    }
}

#[allow(dead_code)]
fn test_local() {
    let filename = "/home/marcel/m2/entw/veloren/airshipper/zippy/end_of_zip.zip";
    let mut zipfile = std::fs::File::open(filename).unwrap();

    let eocd = zip_structs::zip_eocd::ZipEOCD::from_reader(&mut zipfile).unwrap();
    println!("n_cd_entries_in_disk: {}", eocd.n_cd_entries_in_disk);
    println!("n_cd_entries: {}", eocd.n_cd_entries);
    println!("cd_size: {}", eocd.cd_size);
    println!("cd_starting_position: {}", eocd.cd_starting_position);

    let filename = "/home/marcel/m2/entw/veloren/airshipper/zippy/central_dir.zip";
    let mut zipfile = std::fs::File::open(filename).unwrap();

    let mut i = 0;
    while let Ok(cd) =
        zip_structs::zip_central_directory::ZipCDEntry::read_and_generate_from_signature(
            &mut zipfile,
        )
    {
        i += 1;
        let crc = cd.crc32;
        let file_name = std::str::from_utf8(&cd.file_name_raw);
        let download_size = cd.compressed_size;

        println!(
            "{:<60} [{}kb] crc32: {}",
            file_name.unwrap(),
            download_size / 1024,
            crc
        );
    }
    println!("entries: {i}");
}
/*


*/
