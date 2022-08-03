pub mod display;
pub mod enzymepresence;
pub mod imzml_types;
pub mod parsers;
use crate::display::*;
use crate::enzymepresence::*;
use crate::imzml_types::*;
use crate::parsers::*;

use image::io::Reader;
use image::DynamicImage;
use image::{GenericImage, GenericImageView, ImageBuffer, Pixel, Pixels, RgbImage};
use std::fs::write;
use std::fs::OpenOptions;
use std::io::Cursor;
use std::path::Path;
use std::path::PathBuf;
use opengl_graphics::GlyphCache;
use opengl_graphics::*;

use std::*;
fn main() {
    //let mut omswriterpath: String = "tempIMZ files/the.oms".to_string();
    //
    //                    FOR READING FROM IBD:
    //
    // let optimizedmapfromibd: OptimizedMSMap = readoptimizedMSmap(
    //     Path::new("tempIMZ files/raw/RAW/mb.ibd").to_path_buf(),
    //     Path::new("tempIMZ files/raw/RAW/mb.imzML").to_path_buf(),
    // );
    // write_oms_file(Path::new(&omswriterpath).to_path_buf(), optimizedmapfromibd);
    //
    //              Reading a OMS file
    //
    //let optimizedmap = read_oms_file(Path::new(&omswriterpath).to_path_buf());
    start();
}
// Test Enzymes
pub fn circularcorimageoutput(optimizedmap: OptimizedMSMap) {
    let mut vecofranges: Vec<(f64, f64)> = Vec::new();
    vecofranges.push((798.47, 798.67));
    vecofranges.push((794.47, 794.67));
    vecofranges.push((792.96, 793.16));
    vecofranges.push((820.56, 820.76));
    vecofranges.push((778.85, 779.05));
    vecofranges.push((600.87, 601.07));
    let mut imgs: Vec<CircularCorrilationImage> =
        circularcalculatecorrilations(optimizedmap, vecofranges);
    for img in imgs {
        let mut data: String = "Expected Ratio (1 over 2): ".to_string();
        if (img.avrratio1over2 > 1.) {
            data += &img.avrratio1over2.to_string();
            data += " R1 Per 1.0 R2";
        } else {
            data += "1.0 R1 Per ";
            data += &(1.0 / img.avrratio1over2).to_string();
            data += " R2";
        }
        data += ", Average Consistancy As Radius Increases, 1 Rad: ";
        data += &(img.corrilationscore1 * 100.).to_string();
        data += ", 2 Rad: ";
        data += &(img.corrilationscore2 * 100.).to_string();
        data += ", 3 Rad: ";
        data += &(img.corrilationscore3 * 100.).to_string();
        data += ", 4 Rad: ";
        data += &(img.corrilationscore4 * 100.).to_string();
        data += ", 5 Rad: ";
        data += &(img.corrilationscore5 * 100.).to_string();
        data += ", 6 Rad: ";
        data += &(img.corrilationscore6 * 100.).to_string();
        let mut dir: String = "tempIMZ files/video/vid/(".to_string();
        dir += &img.ratioimg.minmz1.to_string();
        dir += "to";
        dir += &img.ratioimg.maxmz1.to_string();
        dir += ") OVER (";
        dir += &img.ratioimg.minmz2.to_string();
        dir += "to";
        dir += &img.ratioimg.maxmz2.to_string();
        let mut dircons1: String = dir.clone();
        let mut dircons2: String = dir.clone();
        let mut dircons3: String = dir.clone();
        let mut dircons4: String = dir.clone();
        let mut dircons5: String = dir.clone();
        let mut dircons6: String = dir.clone();
        let mut dir3: String = dir.clone();
        dir += ").png";
        dircons1 += ") CORRILATIONMAP RADIUS 1 per pixel.png";
        dircons2 += ") CORRILATIONMAP RADIUS 2 per pixel.png";
        dircons3 += ") CORRILATIONMAP RADIUS 3 per pixel.png";
        dircons4 += ") CORRILATIONMAP RADIUS 4 per pixel.png";
        dircons5 += ") CORRILATIONMAP RADIUS 5 per pixel.png";
        dircons6 += ") CORRILATIONMAP RADIUS 6 per pixel.png";
        dir3 += ") DATA.txt";
        use std::fs::File;
        let path: PathBuf = Path::new(&dir).to_path_buf();
        let path3: PathBuf = Path::new(&dir3).to_path_buf();
        fs::write(path3, (data.as_bytes()));
        img.ratioimg.imagem.save(path).unwrap();
        img.corcirc1.imagem.save(dircons1).unwrap();
        img.corcirc2.imagem.save(dircons2).unwrap();
        img.corcirc3.imagem.save(dircons3).unwrap();
        img.corcirc4.imagem.save(dircons4).unwrap();
        img.corcirc5.imagem.save(dircons5).unwrap();
        img.corcirc6.imagem.save(dircons6).unwrap();
    }
}
