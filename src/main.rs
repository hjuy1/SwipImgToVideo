pub mod error;
mod prelude;
pub mod swiping_img;

use error::{Kind, Result};
use prelude::debug_print;
use std::{
    fs::{self, File},
    path::Path,
    time::Instant,
};
use swiping_img::BigImg;

fn read_json<P, T>(file: P) -> Result<Vec<T>>
where
    P: AsRef<Path>,
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let file = File::open(file.as_ref()).map_err(|e| err_new_io!(e))?;
    Ok(serde_json::from_reader(file).map_err(|e| err_new!(Kind::Other, &e.to_string()))?)
}

fn main() -> Result<()> {
    let t = Instant::now();
    let data_file = Path::new("./data").join("Birth.json");

    let work_dir = Path::new("E:/pictures/arknights/0birth");
    fs::create_dir_all(work_dir).map_err(|e| err_new_io!(e))?;

    let data_use = &read_json(data_file)?[..60];

    let si = BigImg::new(work_dir, &data_use);
    // let si = BigImg::builder(work_dir, &data_use)
    //     .step(5)
    //     .video_swip_speed(3)
    //     .build()?;
    debug_print(&si);
    si.run("result.mp4")?;

    println!("cost {} ms", t.elapsed().as_millis());
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test() {}
}
